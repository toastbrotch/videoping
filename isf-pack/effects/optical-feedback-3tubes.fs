/*{
  "DESCRIPTION": "Optical feedback through an old three-tube color camera: red, green and blue each have their own misconverged pickup tube, so every generation the channels part a little further and black-and-white zebra bands turn into rainbow fringes and color spirals.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "feedback", "glitch", "analog"],
  "INPUTS": [
    { "NAME": "inputImage",  "TYPE": "image" },
    { "NAME": "genRate",     "LABEL": "Generations/s",          "TYPE": "float", "DEFAULT": 12.0,   "MIN": 1.0,   "MAX": 60.0 },
    { "NAME": "fbMix",       "LABEL": "Feedback mix",              "TYPE": "float", "DEFAULT": 0.85,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbGain",      "LABEL": "Feedback gain",             "TYPE": "float", "DEFAULT": 1.15,   "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "fbZoom",      "LABEL": "Camera zoom",               "TYPE": "float", "DEFAULT": 1.01,   "MIN": 0.97,  "MAX": 1.03 },
    { "NAME": "fbRotateDeg", "LABEL": "Camera turn (degrees)",     "TYPE": "float", "DEFAULT": 0.4,    "MIN": -2.0,  "MAX": 2.0 },
    { "NAME": "fbShiftX",    "LABEL": "Camera shift X",          "TYPE": "float", "DEFAULT": 0.0,    "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbShiftY",    "LABEL": "Camera shift Y",          "TYPE": "float", "DEFAULT": 0.0,    "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbFocus",     "LABEL": "Camera focus",           "TYPE": "float", "DEFAULT": 0.8,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbBlack",     "LABEL": "Black level",               "TYPE": "float", "DEFAULT": 0.04,   "MIN": 0.0,   "MAX": 0.3 },
    { "NAME": "contrast",    "LABEL": "Camera contrast",           "TYPE": "float", "DEFAULT": 1.8,    "MIN": 0.5,   "MAX": 4.0 },
    { "NAME": "invert",      "LABEL": "Invert per generation","TYPE": "float", "DEFAULT": 1.0,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fold",        "LABEL": "Zebra fold",             "TYPE": "float", "DEFAULT": 0.0,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "zebra",       "LABEL": "Camcorder zebra",           "TYPE": "float", "DEFAULT": 0.0,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "zebraLevel",  "LABEL": "Zebra level",            "TYPE": "float", "DEFAULT": 0.85,   "MIN": 0.4,   "MAX": 1.0 },
    { "NAME": "sharpen",     "LABEL": "VHS sharpening",            "TYPE": "float", "DEFAULT": 0.6,    "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "vhsNoise",    "LABEL": "VHS noise",              "TYPE": "float", "DEFAULT": 0.3,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "color",       "LABEL": "Input color",            "TYPE": "float", "DEFAULT": 0.0,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "hueDrift",    "LABEL": "Hue drift",             "TYPE": "float", "DEFAULT": 0.0,    "MIN": -0.1,  "MAX": 0.1 },
    { "NAME": "rZoom",       "LABEL": "Red tube zoom",           "TYPE": "float", "DEFAULT": 0.002,  "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "rRotDeg",     "LABEL": "Red tube turn",        "TYPE": "float", "DEFAULT": 0.15,   "MIN": -1.0,  "MAX": 1.0 },
    { "NAME": "rShiftX",     "LABEL": "Red tube shift X",      "TYPE": "float", "DEFAULT": 0.001,  "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "rShiftY",     "LABEL": "Red tube shift Y",      "TYPE": "float", "DEFAULT": 0.0,    "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "bZoom",       "LABEL": "Blue tube zoom",          "TYPE": "float", "DEFAULT": -0.002, "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "bRotDeg",     "LABEL": "Blue tube turn",       "TYPE": "float", "DEFAULT": -0.15,  "MIN": -1.0,  "MAX": 1.0 },
    { "NAME": "bShiftX",     "LABEL": "Blue tube shift X",     "TYPE": "float", "DEFAULT": -0.001, "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "bShiftY",     "LABEL": "Blue tube shift Y",     "TYPE": "float", "DEFAULT": 0.0,    "MIN": -0.01, "MAX": 0.01 },
    { "NAME": "tubeDrift",   "LABEL": "Tube drift",             "TYPE": "float", "DEFAULT": 0.3,    "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "tubeLag",     "LABEL": "Tube lag",        "TYPE": "float", "DEFAULT": 0.3,    "MIN": 0.0,   "MAX": 0.95 }
  ],
  "PASSES": [
    { "TARGET": "tubeBuf", "PERSISTENT": true, "FLOAT": true },
    { "TARGET": "loopBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// OPTICAL FEEDBACK V3 — the three-tube camera. Like v2, the loop advances genRate times per
// second and holds in between. Per generation:
//   pass 0 (tubeBuf): each colour tube films the monitor (previous loopBuf) with its own
//                     geometry - green = the camera's zoom/rotate/shift, red and blue add their
//                     misconvergence errors - and keeps some of its own last image (tube lag).
//   pass 1 (loopBuf): camera response (black, gain, contrast, inversion, folding, hue drift),
//                     mixed with the VHS'd input, then camcorder zebra and VHS noise - as in v2.
//   pass 2:           output with a last VCR chroma smear.

#define TAU 6.2831853

float hash(vec2 p){
    return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453);
}

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 hueRotate(vec3 c, float h){
    float a = h*TAU;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}

// True on frames where no new generation is due (both buffers just hold).
bool holding(){
    float rate = max(genRate, 0.1);
    return FRAMEINDEX >= 2 && floor(TIME*rate) == floor((TIME - TIMEDELTA)*rate);
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

// Input with VHS treatment: line damage, chroma smeared sideways, saturation set by Eingangs-Farbe.
vec3 vhsInput(vec2 px){
    px.x += vhsLineShift(px.y);
    vec2 uv = px/RENDERSIZE;
    vec3 c = IMG_NORM_PIXEL(inputImage, uv).rgb;
    float bleed = (2.0 + vhsNoise*6.0)/RENDERSIZE.x;
    vec3 chroma = vec3(0.0);
    for (int i = 0; i < 5; i++)
        chroma += IMG_NORM_PIXEL(inputImage, uv + vec2((float(i) - 1.0)*bleed, 0.0)).rgb;
    chroma /= 5.0;
    return vec3(luma(c)) + (chroma - vec3(luma(chroma)))*color;
}

vec3 loopAt(vec2 uv){
    if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) return vec3(0.0);  // off the monitor
    return IMG_NORM_PIXEL(loopBuf, uv).rgb;
}

// One tube filming the monitor: zoom/rotate about the centre, then shift; soft focus; VHS-style
// sharpening along the line. Returns all three channels; the caller keeps the tube's own one.
vec3 tubeView(vec2 px, float zoom, float rotDeg, vec2 shift){
    vec2 c = RENDERSIZE*0.5;
    vec2 q = px - c;
    float a = radians(rotDeg);
    q = vec2(cos(a)*q.x - sin(a)*q.y, sin(a)*q.x + cos(a)*q.y)/zoom;
    vec2 uv = (q + c)/RENDERSIZE - shift;

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

// Slow wandering of a tube's misconvergence (warm-up drift): incommensurate sines, -1..1.
float wander(float seed){
    return 0.6*sin(TIME*0.13 + seed*7.1) + 0.4*sin(TIME*0.31 + seed*3.7);
}

void main(){
    vec2 px = gl_FragCoord.xy;

    if (PASSINDEX == 0){
        if (holding() || FRAMEINDEX < 2){
            gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(tubeBuf).rgb, 1.0);
            return;
        }

        // The tubes read the monitor through the VHS line damage, so tears feed back.
        vec2 p = px + vec2(vhsLineShift(px.y)*0.5, 0.0);
        float d = clamp(tubeDrift, 0.0, 1.0);
        vec2 shift = vec2(fbShiftX, fbShiftY);

        float red = tubeView(p, fbZoom + rZoom + d*0.003*wander(1.0), fbRotateDeg + rRotDeg + d*0.4*wander(2.0),
                             shift + vec2(rShiftX, rShiftY) + d*0.003*vec2(wander(3.0), wander(4.0))).r;
        float grn = tubeView(p, fbZoom, fbRotateDeg, shift).g;
        float blu = tubeView(p, fbZoom + bZoom + d*0.003*wander(5.0), fbRotateDeg + bRotDeg + d*0.4*wander(6.0),
                             shift + vec2(bShiftX, bShiftY) + d*0.003*vec2(wander(7.0), wander(8.0))).b;
        vec3 now = vec3(red, grn, blu);

        // Tube lag: each tube keeps part of its last image - red longest, blue shortest.
        vec3 lag = clamp(tubeLag, 0.0, 0.95)*vec3(1.0, 0.75, 0.5);
        vec3 prev = IMG_THIS_NORM_PIXEL(tubeBuf).rgb;
        gl_FragColor = vec4(mix(now, prev, lag), 1.0);
        return;
    }

    if (PASSINDEX == 1){
        if (holding()){
            gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(loopBuf).rgb, 1.0);
            return;
        }

        vec3 src = vhsInput(px);
        if (FRAMEINDEX < 2){
            gl_FragColor = vec4(src, 1.0);
            return;
        }

        vec3 cam = IMG_THIS_NORM_PIXEL(tubeBuf).rgb;

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

        if (hueDrift != 0.0)
            cam = clamp(hueRotate(cam, hueDrift), 0.0, 1.5);

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
