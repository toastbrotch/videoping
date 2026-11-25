/*{
  "DESCRIPTION": "Keyed optical feedback after Dan Sandin's Image Processor: a camera-at-the-monitor feedback loop that only runs where a keyer lets it. The key comes from the input's brightness, from the loop itself or from motion; outside the key the live input shows (or black).",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "feedback", "keying", "analog"],
  "INPUTS": [
    { "NAME": "inputImage",   "TYPE": "image" },
    { "NAME": "genRate",      "LABEL": "Generations/s",          "TYPE": "float", "DEFAULT": 15.0,  "MIN": 1.0,   "MAX": 60.0 },
    { "NAME": "fbMix",        "LABEL": "Feedback mix (in the key)",     "TYPE": "float", "DEFAULT": 1.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbGain",       "LABEL": "Feedback gain",             "TYPE": "float", "DEFAULT": 1.3,   "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "fbZoom",       "LABEL": "Camera zoom",               "TYPE": "float", "DEFAULT": 1.028, "MIN": 0.97,  "MAX": 1.03 },
    { "NAME": "fbRotateDeg",  "LABEL": "Camera turn (degrees)",     "TYPE": "float", "DEFAULT": 0.52,   "MIN": -2.0,  "MAX": 2.0 },
    { "NAME": "fbShiftX",     "LABEL": "Camera shift X",          "TYPE": "float", "DEFAULT": 0.002,   "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbShiftY",     "LABEL": "Camera shift Y",          "TYPE": "float", "DEFAULT": 0.0,   "MIN": -0.05, "MAX": 0.05 },
    { "NAME": "fbFocus",      "LABEL": "Camera focus",           "TYPE": "float", "DEFAULT": 0.85,  "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "fbBlack",      "LABEL": "Black level",               "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,   "MAX": 0.3 },
    { "NAME": "contrast",     "LABEL": "Camera contrast",           "TYPE": "float", "DEFAULT": 4.0,   "MIN": 0.5,   "MAX": 4.0 },
    { "NAME": "invert",       "LABEL": "Invert per generation","TYPE": "float", "DEFAULT": 0.14,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "keySource",    "LABEL": "Key source (input / loop / motion)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "keyLow",       "LABEL": "Key low",                 "TYPE": "float", "DEFAULT": 0.37,  "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "keyHigh",      "LABEL": "Key high",                  "TYPE": "float", "DEFAULT": 1.1,   "MIN": 0.0,   "MAX": 1.1 },
    { "NAME": "keySoft",      "LABEL": "Key softness",             "TYPE": "float", "DEFAULT": 0.04,  "MIN": 0.0,   "MAX": 0.3 },
    { "NAME": "keyInvert",    "LABEL": "Invert key",           "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "keyMemory",    "LABEL": "Key memory",           "TYPE": "float", "DEFAULT": 0.059,   "MIN": 0.0,   "MAX": 0.99 },
    { "NAME": "outsideBlack", "LABEL": "Black outside",            "TYPE": "float", "DEFAULT": 0.02,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "keyEdge",      "LABEL": "Key edges",                "TYPE": "float", "DEFAULT": 0.4,   "MIN": 0.0,   "MAX": 2.0 },
    { "NAME": "keyEdgeHue",   "LABEL": "Key edge hue",        "TYPE": "float", "DEFAULT": 0.55,  "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "colorize",     "LABEL": "Colorizer",                 "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "colorBands",   "LABEL": "Colorizer steps",          "TYPE": "float", "DEFAULT": 4.6,   "MIN": 2.0,   "MAX": 16.0 },
    { "NAME": "colorHue",     "LABEL": "Colorizer hue",         "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "colorSpeed",   "LABEL": "Colorizer speed",           "TYPE": "float", "DEFAULT": 0.05,  "MIN": -1.0,  "MAX": 1.0 },
    { "NAME": "inputColor",   "LABEL": "Input color",            "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "tubeSpread",   "LABEL": "Tube errors",            "TYPE": "float", "DEFAULT": 0.2,   "MIN": 0.0,   "MAX": 1.0 },
    { "NAME": "sharpen",      "LABEL": "Sharpening",                "TYPE": "float", "DEFAULT": 0.5,   "MIN": 0.0,   "MAX": 3.0 },
    { "NAME": "noise",        "LABEL": "Noise",                  "TYPE": "float", "DEFAULT": 0.05,  "MIN": 0.0,   "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "loopBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// OPTICAL FEEDBACK KEYER — Sandin-style keyed feedback. Per generation (genRate per second, held
// in between as in optical-feedback-v2), pass 0 writes loopBuf:
//   rgb = mix(outside, inside, key) + key-edge glow
//         inside  = mix(input, camera(previous loopBuf), fbMix)
//         outside = input, faded to black by outsideBlack
//   a   = the key, remembered: max(key now, key before * keyMemory)
// Pass 1 shows loopBuf through the colorizer (brightness bands -> colours, output only).

#define TAU 6.2831853

float hash(vec2 p){
    return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453);
}

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 hsv(float h, float s, float v){
    vec3 c = clamp(abs(mod(h*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
    return v*mix(vec3(1.0), c, s);
}

bool holding(){
    float rate = max(genRate, 0.1);
    return FRAMEINDEX >= 2 && floor(TIME*rate) == floor((TIME - TIMEDELTA)*rate);
}

vec3 inputAt(vec2 px){
    vec3 c = IMG_NORM_PIXEL(inputImage, px/RENDERSIZE).rgb;
    return mix(vec3(luma(c)), c, inputColor);
}

vec3 loopAt(vec2 uv){
    if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) return vec3(0.0);  // off the monitor
    return IMG_NORM_PIXEL(loopBuf, uv).rgb;
}

// The camera filming the monitor with the given geometry: soft focus and light sharpening.
vec3 view(vec2 px, float zoom, float rotDeg){
    vec2 c = RENDERSIZE*0.5;
    vec2 q = px - c;
    float a = radians(rotDeg);
    q = vec2(cos(a)*q.x - sin(a)*q.y, sin(a)*q.x + cos(a)*q.y)/zoom;
    vec2 uv = (q + c)/RENDERSIZE - vec2(fbShiftX, fbShiftY);

    vec2 asp = vec2(RENDERSIZE.y/RENDERSIZE.x, 1.0);
    float r = (1.0 - clamp(fbFocus, 0.0, 1.0))*0.01;
    vec3 v = loopAt(uv)*0.4 + 0.15*(loopAt(uv + vec2(r, 0.0)*asp) + loopAt(uv - vec2(r, 0.0)*asp)
                                   + loopAt(uv + vec2(0.0, r)) + loopAt(uv - vec2(0.0, r)));
    if (sharpen > 0.0){
        vec2 sx = vec2(2.0/RENDERSIZE.x, 0.0);
        v += sharpen*(v - 0.5*(loopAt(uv + sx) + loopAt(uv - sx)));
    }
    return v;
}

// Camera with a symmetric three-tube misconvergence: red zooms/turns a little more than green,
// blue a little less, so the channels part over generations.
vec3 camera(vec2 px){
    float s = clamp(tubeSpread, 0.0, 1.0);
    if (s <= 0.0) return view(px, fbZoom, fbRotateDeg);
    float r = view(px, fbZoom + 0.004*s, fbRotateDeg + 0.3*s).r;
    float g = view(px, fbZoom, fbRotateDeg).g;
    float b = view(px, fbZoom - 0.004*s, fbRotateDeg - 0.3*s).b;
    return vec3(r, g, b);
}

void main(){
    vec2 px = gl_FragCoord.xy;

    if (PASSINDEX == 0){
        if (holding()){
            gl_FragColor = IMG_THIS_NORM_PIXEL(loopBuf);
            return;
        }

        vec3 src = inputAt(px);
        if (FRAMEINDEX < 2){
            gl_FragColor = vec4(src, 0.0);
            return;
        }

        // Camera response: black level, gain, contrast around mid-grey, soft ceiling, inversion.
        vec3 cam = camera(px);
        cam = max(cam - fbBlack, 0.0)/max(1.0 - fbBlack, 0.05)*fbGain;
        cam = 0.5 + (cam - 0.5)*contrast;
        cam = 0.5 + 0.55*tanh((cam - 0.5)/0.55);
        cam = mix(cam, 1.0 - cam, clamp(invert, 0.0, 1.0));

        // Key signal: input brightness -> fed-back brightness -> |input - feedback| (motion).
        vec4 prev = IMG_THIS_NORM_PIXEL(loopBuf);
        float ks = clamp(keySource, 0.0, 2.0);
        float sigIn = luma(src);
        float sigLoop = luma(cam);
        float sigDiff = clamp(abs(sigIn - luma(prev.rgb))*2.0, 0.0, 1.0);
        float sig = mix(sigIn, sigLoop, clamp(ks, 0.0, 1.0));
        sig = mix(sig, sigDiff, clamp(ks - 1.0, 0.0, 1.0));

        // Window key between keyLow and keyHigh, softened, optionally inverted, remembered.
        float s = max(keySoft, 0.002);
        float key = smoothstep(keyLow - s, keyLow + s, sig)*(1.0 - smoothstep(keyHigh - s, keyHigh + s, sig));
        key = mix(key, 1.0 - key, clamp(keyInvert, 0.0, 1.0));
        key = max(key, prev.a*keyMemory);

        vec3 inside = mix(src, cam, clamp(fbMix, 0.0, 1.0));
        vec3 outside = src*(1.0 - clamp(outsideBlack, 0.0, 1.0));
        vec3 v = mix(outside, inside, key);

        // Glowing key edges (where the key is half on) - fed back, so they echo outward.
        float edge = 1.0 - abs(2.0*key - 1.0);
        v += keyEdge*edge*edge*hsv(keyEdgeHue, 0.8, 1.0);

        if (noise > 0.0)
            v += (hash(px + float(FRAMEINDEX)) - 0.5)*noise*0.15;

        gl_FragColor = vec4(clamp(v, 0.0, 1.5), key);
        return;
    }

    // Colorizer: brightness is cut into colorBands bands, each its own hue (cycling over time),
    // keeping the band's brightness - Sandin's colourizer look. Mixed over the plain picture.
    vec3 c = clamp(IMG_THIS_NORM_PIXEL(loopBuf).rgb, 0.0, 1.0);
    if (colorize > 0.0){
        float n = floor(clamp(colorBands, 2.0, 16.0));
        float l = luma(c);
        float band = min(floor(l*n), n - 1.0);
        vec3 col = hsv(fract(colorHue + band/n + TIME*colorSpeed), 0.85, 0.25 + 0.75*(band + 1.0)/n);
        c = mix(c, col, clamp(colorize, 0.0, 1.0));
    }
    gl_FragColor = vec4(c, 1.0);
}
