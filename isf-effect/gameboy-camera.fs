/*{
  "DESCRIPTION": "Game Boy Camera rasterizer. Reduces the image to a coarse grid of square sensor pixels, applies the camera's edge enhancement, then quantises to 4 shades using ordered-dither pattern tiles: classic 4x4 Bayer, the chunky 'X' / dot / inverted-X ladder, or diagonal cross-hatching. Four palettes (DMG green, pocket gray, sepia printer, GB light), optional LCD dot grid and a patterned camera frame.",
  "CREDIT": "CC0",
  "ISFVSN": "2",
  "CATEGORIES": ["stylize", "retro", "dither"],
  "INPUTS": [
    { "NAME": "inputImage",   "TYPE": "image" },
    { "NAME": "uResolution",  "LABEL": "Aufloesung (Zeilen)",        "TYPE": "float", "DEFAULT": 112.0, "MIN": 32.0, "MAX": 288.0 },
    { "NAME": "uContrast",    "LABEL": "Kontrast",                   "TYPE": "float", "DEFAULT": 1.20,  "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uStyle",       "LABEL": "Muster (Bayer/X/Schraffur)", "TYPE": "float", "DEFAULT": 1.00,  "MIN": 0.0,  "MAX": 2.0 },
    { "NAME": "uEdge",        "LABEL": "Kantenschaerfung",           "TYPE": "float", "DEFAULT": 0.60,  "MIN": 0.0,  "MAX": 2.0 },
    { "NAME": "uBrightness",  "LABEL": "Helligkeit",                 "TYPE": "float", "DEFAULT": 0.00,  "MIN": -0.5, "MAX": 0.5 },
    { "NAME": "uPalette",     "LABEL": "Palette",                    "TYPE": "float", "DEFAULT": 0.00,  "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uPatternSize", "LABEL": "Mustergroesse",              "TYPE": "float", "DEFAULT": 1.00,  "MIN": 1.0,  "MAX": 4.0 },
    { "NAME": "uNoise",       "LABEL": "Sensorrauschen",             "TYPE": "float", "DEFAULT": 0.10,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uLCDGrid",     "LABEL": "LCD-Raster",                 "TYPE": "float", "DEFAULT": 0.25,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uFrame",       "LABEL": "Kamera-Rahmen",              "TYPE": "float", "DEFAULT": 0.00,  "MIN": 0.0,  "MAX": 1.0 }
  ]
}*/

// GAME BOY CAMERA — square sensor pixels, 4 shades, pattern dithering.
// GLSL 1.20 / ISF v2: IMG_NORM_PIXEL + gl_FragCoord + gl_FragColor, no int
// arithmetic, so it compiles both in regular ISF hosts and in ISFBrowser.

float hash21(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }
float is(float a, float b){ return 1.0 - step(0.5, abs(a - b)); }  // a == b for whole numbers

float luma(vec2 uv){
    vec3 img = IMG_NORM_PIXEL(inputImage, uv).rgb;
    return dot(img, vec3(0.299, 0.587, 0.114));
}

// -------------------------------------------------------- dither patterns --
// Each returns 1 where the lighter of the two neighbouring shades is used.
// c = position inside the 4x4 tile (0..3), f = fraction between shades.

float bayer2(vec2 a){ a = floor(a); return fract(a.x/2.0 + a.y*a.y*0.75); }
float bayer4(vec2 a){ return bayer2(0.5*a)*0.25 + bayer2(a); }

float patBayer(vec2 c, float f){ return step(bayer4(c) + 0.5/16.0, f); }

// Game Boy Camera style ladder: dot, two dots, dot grid, small 'x',
// big diagonal X lattice, then the same shapes inverted.
float patX(vec2 c, float f){
    float x = c.x, y = c.y;
    float dot1  = is(x, 0.0)*is(y, 0.0);
    float dot2  = max(dot1, is(x, 2.0)*is(y, 2.0));
    float grid  = is(mod(x, 2.0), 0.0)*is(mod(y, 2.0), 0.0);
    float smallX = step(x, 2.0)*step(y, 2.0)*max(is(x, y), is(x + y, 2.0));
    float bigX  = max(is(x, y), is(x + y, 3.0));
    float lv = floor(f*10.0 + 0.5);
    if (lv < 0.5) return 0.0;
    if (lv < 1.5) return dot1;
    if (lv < 2.5) return dot2;
    if (lv < 3.5) return grid;
    if (lv < 4.5) return smallX;
    if (lv < 5.5) return bigX;
    if (lv < 6.5) return 1.0 - smallX;
    if (lv < 7.5) return 1.0 - grid;
    if (lv < 8.5) return 1.0 - dot2;
    if (lv < 9.5) return 1.0 - dot1;
    return 1.0;
}

// Diagonal hatching: single diagonal, cross-hatch, then inverted.
float patHatch(vec2 c, float f){
    float d1 = is(mod(c.x + c.y, 4.0), 0.0);
    float d2 = max(d1, is(mod(c.x - c.y + 4.0, 4.0), 0.0));
    float lv = floor(f*5.0 + 0.5);
    if (lv < 0.5) return 0.0;
    if (lv < 1.5) return d1;
    if (lv < 2.5) return d2;
    if (lv < 3.5) return 1.0 - d2;
    if (lv < 4.5) return 1.0 - d1;
    return 1.0;
}

float pattern(vec2 c, float f){
    float s = floor(uStyle + 0.5);
    if (s < 0.5) return patBayer(c, f);
    if (s < 1.5) return patX(c, f);
    return patHatch(c, f);
}

// --------------------------------------------------------------- palettes --
vec3 palette(float i){
    float p = floor(uPalette + 0.5);
    vec3 c0, c1, c2, c3;
    if (p < 0.5){        // DMG green
        c0 = vec3(0.059, 0.220, 0.059); c1 = vec3(0.188, 0.384, 0.188);
        c2 = vec3(0.545, 0.675, 0.059); c3 = vec3(0.608, 0.737, 0.059);
    } else if (p < 1.5){ // pocket gray
        c0 = vec3(0.106);               c1 = vec3(0.333);
        c2 = vec3(0.667);               c3 = vec3(0.910);
    } else if (p < 2.5){ // sepia printer paper
        c0 = vec3(0.169, 0.106, 0.055); c1 = vec3(0.420, 0.290, 0.169);
        c2 = vec3(0.722, 0.604, 0.416); c3 = vec3(0.949, 0.890, 0.776);
    } else {             // GB light
        c0 = vec3(0.043, 0.239, 0.227); c1 = vec3(0.184, 0.478, 0.435);
        c2 = vec3(0.498, 0.820, 0.725); c3 = vec3(0.839, 1.000, 0.949);
    }
    if (i < 0.5) return c0;
    if (i < 1.5) return c1;
    if (i < 2.5) return c2;
    return c3;
}

// ------------------------------------------------------------------- main --
void main(){
    vec2 fc = gl_FragCoord.xy;

    // Sensor pixel grid (square pixels, uResolution rows)
    float rows = max(floor(uResolution), 8.0);
    float pix = RENDERSIZE.y/rows;
    vec2 px = floor(fc/pix);
    vec2 gridSize = ceil(RENDERSIZE/pix);

    float psize = max(floor(uPatternSize + 0.5), 1.0);
    vec2 cell = mod(floor(px/psize), 4.0);

    float border = floor(uFrame*rows*0.14);
    vec2 inner = gridSize - 2.0*border;
    vec2 ip = px - border;
    float inside = step(0.0, ip.x)*step(0.0, ip.y)*step(ip.x, inner.x - 1.0)*step(ip.y, inner.y - 1.0);

    float idx;
    if (inside > 0.5){
        vec2 uv = (ip + 0.5)/inner;
        vec2 st = 1.0/inner;
        float v = luma(uv);
        float nb = luma(uv + vec2(st.x, 0.0)) + luma(uv - vec2(st.x, 0.0))
                 + luma(uv + vec2(0.0, st.y)) + luma(uv - vec2(0.0, st.y));
        v += uEdge*(v - nb*0.25)*2.0;                                  // edge enhancement
        v = (v - 0.5)*uContrast + 0.5 + uBrightness;
        v += (hash21(px + floor(TIME*15.0)*17.13) - 0.5)*uNoise*0.25;  // sensor noise
        v = clamp(v, 0.0, 1.0);

        float s = v*3.0;
        float base = min(floor(s), 2.0);
        idx = base + pattern(cell, s - base);
    } else {
        // Frame: dark X-patterned border with a light rim around the picture
        vec2 dIn = max(-ip, ip - (inner - 1.0));
        float rim = is(max(dIn.x, dIn.y), 1.0);
        float outer = is(min(min(px.x, px.y), min(gridSize.x - 1.0 - px.x, gridSize.y - 1.0 - px.y)), 0.0);
        idx = patX(mod(px, 4.0), 0.5);
        idx = max(idx, 3.0*max(rim, outer));
    }

    vec3 col = palette(idx);

    // LCD dot grid: thin light gaps between screen pixels
    vec2 fp = fract(fc/pix);
    float g = clamp(1.2/pix, 0.0, 0.5);
    float gap = max(step(fp.x, g), step(fp.y, g))*step(3.0, pix);
    col = mix(col, palette(3.0), uLCDGrid*gap*0.6);

    gl_FragColor = vec4(col, 1.0);
}
