/*{
  "DESCRIPTION": "Optical video feedback with VHS character: a virtual camera films the monitor showing its own picture, through a contrasty, inverting camera, so brightness edges multiply into black-and-white zebra bands that crawl outwards. The loop runs at a set number of generations per second.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "feedback", "glitch", "analog"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "genRate",    "LABEL": "Generations/s",         "TYPE": "float", "DEFAULT": 12.0,  "MIN": 1.0,   "MAX": 60.0 },
    { "NAME": "fbMix",      "LABEL": "Feedback mix",             "TYPE": "float", "DEFAULT": 0.85,  "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbGain",     "LABEL": "Feedback gain",            "TYPE": "float", "DEFAULT": 1.15,  "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "fbZoom",     "LABEL": "Camera zoom",              "TYPE": "float", "DEFAULT": 1.01,  "MIN": 0.97,  "MAX": 1.03 },
    { "NAME": "fbRotateDeg","LABEL": "Camera turn (degrees)",    "TYPE": "float", "DEFAULT": 0.4,   "MIN": -2.0,  "MAX": 2.0 },
    { "NAME": "fbShiftX",   "LABEL": "Camera shift X",         "TYPE": "float", "DEFAULT": 0.0,   "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbShiftY",   "LABEL": "Camera shift Y",         "TYPE": "float", "DEFAULT": 0.0,   "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbFocus",    "LABEL": "Camera focus",          "TYPE": "float", "DEFAULT": 0.8,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbBlack",    "LABEL": "Black level",              "TYPE": "float", "DEFAULT": 0.04,  "MIN": 0.0,   "MAX": 0.3 },
    { "NAME": "contrast",   "LABEL": "Camera contrast",          "TYPE": "float", "DEFAULT": 1.8,   "MIN": 0.5,   "MAX": 4.0 },
    { "NAME": "invert",     "LABEL": "Invert per generation","TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fold",       "LABEL": "Zebra fold",            "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "zebra",      "LABEL": "Camcorder zebra",          "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "sharpen",    "LABEL": "VHS sharpening",           "TYPE": "float", "DEFAULT": 0.6,   "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "color",      "LABEL": "Color",                    "TYPE": "float", "DEFAULT": 0.3,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "vhsNoise",   "LABEL": "VHS noise",             "TYPE": "float", "DEFAULT": 0.3,   "MIN": 0.0,   "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "loopBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// OPTICAL FEEDBACK V2 — optical-feedback.fs with the loop running at genRate generations per
// second: on frames where no new generation is due, pass 0 just passes the held frame on.
// v1: camera-at-the-monitor loop with VHS artefacts. Pass 0: new loop frame =
// mix(VHS(input), camera(previous loop frame), fbMix), with zebra overlay and VHS damage applied
// inside the loop so they feed back too. Pass 1: show it with a last bit of chroma bleed.

#define TAU 6.2831853

const float zebraLevel = 0.85;   // v1 had a slider for this; its slot went to genRate

float hash(vec2 p){
    return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453);
}
float hash1(float n){ return fract(sin(n*12.9898)*43758.5453); }

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 hueRotate(vec3 c, float h){
    float a = h*TAU;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}

// VHS line damage for screen row y (pixels): horizontal jitter (px), plus a tracking band that
// rolls up the frame and a head-switching tear at the bottom.
float vhsLineShift(float y){
    float v = clamp(vhsNoise, 0.0, 1.0);
    if (v <= 0.0) return 0.0;
    float H = RENDERSIZE.y;
    float frame = float(FRAMEINDEX);
    float jitter = (hash(vec2(floor(y), frame)) - 0.5)*v*3.0;
    float bandY = fract(TIME*0.07)*H*1.4 - H*0.2;
    float band = exp(-pow((y - bandY)/(H*0.03), 2.0));
    float track = band*(hash(vec2(floor(y*0.5), frame + 3.0)) - 0.3)*v*60.0;
    float head = (y < H*0.03) ? (1.0 - y/(H*0.03))*v*40.0 : 0.0;
    return jitter + track + head;
}

// Input with VHS treatment: line damage, chroma smeared sideways (and a bit late).
vec3 vhsInput(vec2 px){
    px.x += vhsLineShift(px.y);
    vec2 uv = px/RENDERSIZE;
    vec3 c = IMG_NORM_PIXEL(inputImage, uv).rgb;
    float bleed = (2.0 + vhsNoise*6.0)/RENDERSIZE.x;
    vec3 chroma = vec3(0.0);
    for (int i = 0; i < 5; i++)
        chroma += IMG_NORM_PIXEL(inputImage, uv + vec2((float(i) - 1.0)*bleed, 0.0)).rgb;
    chroma /= 5.0;
    return vec3(luma(c)) + (chroma - vec3(luma(chroma)));
}

vec3 loopAt(vec2 uv){
    if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) return vec3(0.0);  // off the monitor
    return IMG_NORM_PIXEL(loopBuf, uv).rgb;
}

// The camera: zoom/rotate about the centre, then shift; soft focus; VHS-style sharpening along
// the line (the overshoot rings are what the loop turns into extra zebra bands).
vec3 camera(vec2 px){
    vec2 c = RENDERSIZE*0.5;
    vec2 q = px - c;
    float a = radians(fbRotateDeg);
    q = vec2(cos(a)*q.x - sin(a)*q.y, sin(a)*q.x + cos(a)*q.y)/fbZoom;
    vec2 uv = (q + c)/RENDERSIZE - vec2(fbShiftX, fbShiftY);

    vec2 asp = vec2(RENDERSIZE.y/RENDERSIZE.x, 1.0);
    float r = (1.0 - clamp(fbFocus, 0.0, 1.0))*0.01;
    vec3 v = loopAt(uv)*0.4 + 0.15*(loopAt(uv + vec2(r, 0.0)*asp) + loopAt(uv - vec2(r, 0.0)*asp)
                                   + loopAt(uv + vec2(0.0, r)) + loopAt(uv - vec2(0.0, r)));

    if (sharpen > 0.0){
        vec2 sx = vec2(2.0/RENDERSIZE.x, 0.0);
        vec3 side = 0.5*(loopAt(uv + sx) + loopAt(uv - sx));
        v += sharpen*(v - side);
    }
    return v;
}

void main(){
    vec2 px = gl_FragCoord.xy;

    if (PASSINDEX == 0){
        // Generation clock: a new generation only when floor(TIME*genRate) ticks over since the
        // previous frame - otherwise hold the loop as it is (also holds the input and VHS noise).
        // Stateless, and independent of the host's frame rate.
        float rate = max(genRate, 0.1);
        if (FRAMEINDEX >= 2 && floor(TIME*rate) == floor((TIME - TIMEDELTA)*rate)){
            gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(loopBuf).rgb, 1.0);
            return;
        }

        vec3 src = vhsInput(px);
        if (FRAMEINDEX < 2){
            gl_FragColor = vec4(src, 1.0);
            return;
        }

        // Camera reads the previous frame through the same line damage, so VHS tears feed back.
        vec3 cam = camera(px + vec2(vhsLineShift(px.y)*0.5, 0.0));

        // Camera response: black level, gain, S-curve contrast around mid-grey, soft ceiling.
        cam = max(cam - fbBlack, 0.0)/max(1.0 - fbBlack, 0.05)*fbGain;
        cam = 0.5 + (cam - 0.5)*contrast;
        cam = 0.5 + 0.55*tanh((cam - 0.5)/0.55);

        // Inversion every generation: edges alternate black/white as they travel = zebra bands.
        cam = mix(cam, 1.0 - cam, clamp(invert, 0.0, 1.0));

        // Brightness folding: levels fold back like a triangle wave, adding contour stripes.
        if (fold > 0.0){
            float k = 1.0 + fold*7.0;
            vec3 tri = 1.0 - abs(fract(cam*k*0.5)*2.0 - 1.0);
            cam = mix(cam, tri, smoothstep(0.0, 0.3, fold));
        }

        // Colour: from black-and-white to full colour, with a hue drift each generation.
        cam = mix(vec3(luma(cam)), cam, color);
        if (color > 0.0)
            cam = clamp(hueRotate(cam, color*0.02), 0.0, 1.5);

        vec3 v = mix(src, cam, clamp(fbMix, 0.0, 1.0));

        // Camcorder zebra on overexposed areas - inside the loop, so it gets filmed again.
        if (zebra > 0.0){
            float over = smoothstep(zebraLevel - 0.03, zebraLevel + 0.03, luma(v));
            float stripes = step(0.5, fract((px.x + px.y)/(RENDERSIZE.y*0.02) - TIME*1.5));
            v = mix(v, v*stripes, over*zebra);
        }

        // VHS noise: grain plus occasional white dropout streaks.
        if (vhsNoise > 0.0){
            float frame = float(FRAMEINDEX);
            v += (hash(px + frame) - 0.5)*vhsNoise*0.12;
            float dropRow = floor(px.y/2.0);
            if (hash(vec2(dropRow, frame)) < vhsNoise*0.004){
                float x0 = hash(vec2(dropRow + 5.0, frame))*RENDERSIZE.x;
                float len = hash(vec2(dropRow + 9.0, frame))*RENDERSIZE.x*0.2;
                if (px.x > x0 && px.x < x0 + len) v = vec3(1.0);
            }
        }

        gl_FragColor = vec4(clamp(v, 0.0, 1.5), 1.0);
        return;
    }

    // Output: one last horizontal chroma smear, like playback on a VCR.
    vec2 uv = isf_FragNormCoord;
    vec3 c = IMG_NORM_PIXEL(loopBuf, uv).rgb;
    vec2 b = vec2(3.0/RENDERSIZE.x, 0.0);
    vec3 cs = (IMG_NORM_PIXEL(loopBuf, uv - b).rgb + c + IMG_NORM_PIXEL(loopBuf, uv + b).rgb)/3.0;
    vec3 outc = vec3(luma(c)) + (cs - vec3(luma(cs)));
    gl_FragColor = vec4(clamp(outc, 0.0, 1.0), 1.0);
}
