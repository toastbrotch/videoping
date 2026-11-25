/*{
  "DESCRIPTION": "The Rutt-Etra scan-processor look of the 1970s: the picture is read as scanlines (and optionally vertical lines for a mesh), each pushed sideways by the brightness, so the picture turns into a glowing line terrain with a pseudo-3D tilt and depth fog.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "stylize", "retro", "glitch"],
  "INPUTS": [
    { "NAME": "inputImage",    "TYPE": "image" },
    { "NAME": "uLines",        "LABEL": "Scanlines",            "TYPE": "float", "DEFAULT": 60.0,  "MIN": 8.0,  "MAX": 160.0 },
    { "NAME": "uThickness",    "LABEL": "Line width (px)",    "TYPE": "float", "DEFAULT": 1.5,   "MIN": 0.5,  "MAX": 6.0 },
    { "NAME": "uDisplace",     "LABEL": "Displacement",            "TYPE": "float", "DEFAULT": 1.0,   "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uPerspective",  "LABEL": "Perspective / tilt",      "TYPE": "float", "DEFAULT": 0.35,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uDepthFog",     "LABEL": "Depth fog",           "TYPE": "float", "DEFAULT": 0.3,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uMesh",         "LABEL": "Mesh (vertical lines)", "TYPE": "float", "DEFAULT": 0.3,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSmoothing",    "LABEL": "Smoothing",             "TYPE": "float", "DEFAULT": 0.5,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uGlow",         "LABEL": "Glow",                  "TYPE": "float", "DEFAULT": 0.3,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uColorMode",    "LABEL": "Color mode",             "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uBgBrightness", "LABEL": "Background brightness","TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uInvert",       "LABEL": "Invert",           "TYPE": "bool",  "DEFAULT": false },
    { "NAME": "uNoise",        "LABEL": "Noise / jitter",       "TYPE": "float", "DEFAULT": 0.05,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uRotate",       "LABEL": "Rotation (0-180°)",   "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uScrollSpeed",  "LABEL": "Scroll speed","TYPE": "float", "DEFAULT": 0.0,   "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uChaos",        "LABEL": "Chaos",                 "TYPE": "float", "DEFAULT": 0.0,   "MIN": 0.0,  "MAX": 1.0 }
  ]
}*/

// RUTT-ETRA — per-pixel scan-conversion: each fragment checks a handful of
// candidate scanlines near it (the ones whose luminance-driven displacement
// could plausibly reach this far) and lights up if one passes close by.
// GLSL ES 1.00 / ISF v2: gl_FragColor, TIME/RENDERSIZE, IMG_NORM_PIXEL,
// literal loop bounds.

float hash1(float n){ return fract(sin(n)*43758.5453); }

// One family of scanlines (horizontal when swapAxes=false, vertical for the
// mesh overlay when true) - same logic either way, just with the scan axis
// and the displacement axis swapped.
float scanLineField(vec2 ruv, float lineCount, bool swapAxes, float pixelScale,
                     float scrollOff, out vec3 lineColor){
    vec2 axisUV = swapAxes ? ruv.yx : ruv;

    // Fewer along-scan samples = a blockier, more "sampled waveform" look,
    // authentic to how a real scan converter only has finite horizontal
    // resolution; more samples = a smooth continuous curve.
    float scanSamples = mix(24.0, 400.0, uSmoothing);
    float qx = fract(floor((axisUV.x + scrollOff)*scanSamples)/scanSamples);

    float bestInk = 0.0;
    vec3 bestColor = vec3(0.0);

    for (int i = 0; i < 5; i++){
        float dr = float(i - 2);
        float rowIndex = floor(axisUV.y*lineCount) + dr;
        float rowCenter = (rowIndex + 0.5)/lineCount;

        vec2 sampleUV = swapAxes ? vec2(rowCenter, qx) : vec2(qx, rowCenter);
        vec3 srcColor = IMG_NORM_PIXEL(inputImage, clamp(sampleUV, 0.0, 1.0)).rgb;
        float lum = dot(srcColor, vec3(0.299, 0.587, 0.114));
        if (uInvert > 0.5) lum = 1.0 - lum;

        // depth: 0 at the near edge (axisUV.y == 1), 1 at the far edge
        // (axisUV.y == 0) - the plane recedes "into" the screen upward.
        float depth = 1.0 - rowCenter;
        float depthScale = mix(1.0, 1.0 - depth, uPerspective);
        float fog = mix(1.0, depthScale, uDepthFog);

        float dispAmt = uDisplace * (1.0 + uChaos*1.5);
        float jitter = (hash1(rowIndex*17.3 + floor(TIME*20.0) + (swapAxes ? 500.0 : 0.0)) - 0.5)
                       * (uNoise + uChaos*0.4) * 0.03;
        float displaced = rowCenter - (lum*dispAmt)/lineCount*depthScale + jitter;

        float d = abs(axisUV.y - displaced);
        float dPixels = d * pixelScale;
        float thick = uThickness * (1.0 + uChaos*0.5);
        float core = 1.0 - smoothstep(0.0, thick, dPixels);
        float glowWidth = thick * (1.0 + uGlow*10.0);
        float glowMask = (1.0 - smoothstep(0.0, glowWidth, dPixels)) * uGlow * 0.5;
        float inkAmt = max(core, glowMask) * fog;

        if (inkAmt > bestInk){
            bestInk = inkAmt;
            if (uColorMode < 0.5)      bestColor = vec3(0.2, 1.0, 0.35);          // phosphor green
            else if (uColorMode < 1.5) bestColor = vec3(1.0);                      // white
            else if (uColorMode < 2.5) bestColor = srcColor / max(lum, 0.2);       // colorized scan, brightness-normalized so dark source hues still show
            else                       bestColor = 0.5 + 0.5*cos(6.2831*(depth + vec3(0.0, 0.33, 0.67))); // depth rainbow
        }
    }

    lineColor = bestColor;
    return bestInk;
}

void main(){
    vec2 uv = gl_FragCoord.xy / RENDERSIZE.xy;

    float ang = uRotate * 3.14159265;
    float ca = cos(ang), sa = sin(ang);
    vec2 p = uv - 0.5;
    vec2 ruv = vec2(ca*p.x + sa*p.y, -sa*p.x + ca*p.y) + 0.5;

    float scrollOff = TIME * uScrollSpeed * 0.15;

    vec3 lineColor;
    float totalInk = scanLineField(ruv, uLines, false, RENDERSIZE.y, scrollOff, lineColor);

    if (uMesh > 0.0){
        vec3 meshColor;
        float meshLines = max(4.0, uLines*0.5);
        float inkV = scanLineField(ruv, meshLines, true, RENDERSIZE.x, scrollOff*0.6, meshColor) * uMesh;
        if (inkV > totalInk){
            totalInk = inkV;
            lineColor = meshColor;
        }
    }

    vec3 col = mix(vec3(uBgBrightness), lineColor, clamp(totalInk, 0.0, 1.0));
    gl_FragColor = vec4(col, 1.0);
}
