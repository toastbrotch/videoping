/*{
  "DESCRIPTION": "A wild halftone print: the picture re-printed as up to four rotated dot screens, one per ink. Huge dots, seven dot shapes that morph into each other, eight color schemes, ink offsets that spin, smeared colors, a warped screen, pulsing waves and a Chaos control.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "stylize", "glitch", "color-effect"],
  "INPUTS": [
    { "NAME": "inputImage",     "TYPE": "image" },
    { "NAME": "uCellSize",      "LABEL": "Cell size (px)",     "TYPE": "float", "DEFAULT": 14.0, "MIN": 3.0,  "MAX": 400.0 },
    { "NAME": "uDotScale",      "LABEL": "Dot size",           "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uContrast",      "LABEL": "Contrast",                "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.2,  "MAX": 6.0 },
    { "NAME": "uShape",         "LABEL": "Dot shape (circle .. line)","TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0,  "MAX": 6.0 },
    { "NAME": "uSoftness",      "LABEL": "Softness",               "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uAngle",         "LABEL": "Screen angle",           "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uRotSpeed",      "LABEL": "Screen rotation",         "TYPE": "float", "DEFAULT": 0.0,  "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "uAngleSpread",   "LABEL": "Ink angle spread",  "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 6.0 },
    { "NAME": "uColorMode",     "LABEL": "Color scheme (0-7)",        "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 7.0 },
    { "NAME": "uHueShift",      "LABEL": "Hue shift",    "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uDisplace",      "LABEL": "Ink offset (cells)",   "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 6.0 },
    { "NAME": "uDisplaceSpin",  "LABEL": "Offset spin",         "TYPE": "float", "DEFAULT": 0.0,  "MIN": -4.0, "MAX": 4.0 },
    { "NAME": "uSmear",         "LABEL": "Color smear",      "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uWarp",          "LABEL": "Screen warp",       "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uPulse",         "LABEL": "Pulse waves",             "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uChaos",         "LABEL": "Chaos",                   "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 }
  ]
}*/

// HALFTONE CHAOS — multi-screen halftone. GLSL ES 1.00 / ISF v2 style:
// gl_FragColor, TIME/RENDERSIZE, IMG_NORM_PIXEL, literal loop bounds.
//
// Each ink layer gets its own rotated (and possibly displaced/warped) square
// cell grid. Per pixel, the 3x3 neighbouring cells of each layer are checked
// so dots up to 1.5 cells in radius overlap their neighbours correctly; the
// tone each dot is sized by is sampled at its own cell centre.
//
// Colour schemes (uColorMode, rounded):
//   0 Mono      black dots on white paper              (multiply)
//   1 CMYK      classic 4-ink print                     (multiply)
//   2 RGB       red/green/blue light dots on black      (additive)
//   3 Original  dots in the image's own colours         (over)
//   4 Neon      3 palette-coloured luma screens         (additive)
//   5 Acid      hue-rotating CMYK on hue-cycling paper  (multiply)
//   6 Thermal   hot dots for lights, cold for shadows   (over)
//   7 Spectral  cyan/magenta/yellow light on black      (additive)

#define PI  3.14159265
#define TAU 6.2831853

float hash1(vec2 p){
    return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453);
}
vec2 hash2(vec2 p){
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(p)*43758.5453);
}

mat2 rot(float a){
    float c = cos(a), s = sin(a);
    return mat2(c, -s, s, c);
}

vec3 hueRotate(vec3 c, float h){
    // Rodrigues rotation around the grey axis - cheap, keeps luminance roughly.
    float a = h*TAU;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}

vec3 cosPalette(float t){
    return 0.5 + 0.5*cos(TAU*(t + vec3(0.0, 0.33, 0.67)));
}

vec3 heatRamp(float t){
    t = clamp(t, 0.0, 1.0);
    return clamp(vec3(t*3.0, t*3.0 - 1.0, t*3.0 - 2.0), 0.0, 1.0);
}

// Signed distance of a dot of "radius" r (cell units) for one of the 7 shapes.
float shapeSD(vec2 p, float r, int s){
    if (s == 0) return length(p) - r;                                   // circle
    if (s == 1) return (abs(p.x) + abs(p.y))*0.7071 - r*0.886;          // diamond
    if (s == 2) return max(abs(p.x), abs(p.y)) - r*0.886;               // square
    if (s == 3) return abs(length(p) - r*0.75) - r*0.3;                 // ring
    if (s == 4){                                                        // 5-point star
        float a = atan(p.y, p.x);
        return length(p) - r*(0.75 + 0.45*cos(5.0*a));
    }
    if (s == 5){                                                        // cross
        vec2 q = abs(p);
        return min(max(q.x - r*1.1, q.y - r*0.35), max(q.y - r*1.1, q.x - r*0.35));
    }
    return abs(p.y) - r*0.785;                                          // line screen
}

// Continuous morph between neighbouring shapes.
float dotSD(vec2 p, float r){
    float s = clamp(uShape, 0.0, 6.0);
    int s0 = int(floor(s));
    int s1 = int(min(floor(s) + 1.0, 6.0));
    return mix(shapeSD(p, r, s0), shapeSD(p, r, s1), fract(s));
}

int colorMode(){
    return int(floor(clamp(uColorMode, 0.0, 7.0) + 0.5));
}

int layerCount(int m){
    if (m == 0 || m == 3) return 1;
    if (m == 1 || m == 5) return 4;
    if (m == 6) return 2;
    return 3;
}

// 0 multiply onto paper, 1 additive light, 2 opaque "over"
int blendMode(int m){
    if (m == 0 || m == 1 || m == 5) return 0;
    if (m == 3 || m == 6) return 2;
    return 1;
}

vec3 paperColor(int m, float hue){
    if (m == 0 || m == 1) return vec3(1.0, 0.98, 0.94);
    if (m == 5) return cosPalette(hue + TIME*0.07)*0.6 + 0.4;
    if (m == 4) return vec3(0.02, 0.0, 0.06);
    return vec3(0.0);
}

// Classic screen angles (degrees) for each layer, before spreading.
float baseAngle(int m, int i){
    if (m == 1 || m == 5){
        if (i == 0) return 15.0;
        if (i == 1) return 75.0;
        if (i == 2) return 0.0;
        return 45.0;
    }
    if (m == 6) return (i == 0) ? 45.0 : 0.0;
    if (i == 0) return 15.0;
    if (i == 1) return 45.0;
    return 75.0;
}

// Tone (0..1, how big the dot is) of layer i for a source colour.
float layerTone(int m, int i, vec3 s){
    float l = dot(s, vec3(0.299, 0.587, 0.114));
    if (m == 0) return 1.0 - l;
    if (m == 1 || m == 5){
        float k = 1.0 - max(s.r, max(s.g, s.b));
        float d = max(1.0 - k, 1e-4);
        if (i == 0) return (1.0 - s.r - k)/d;
        if (i == 1) return (1.0 - s.g - k)/d;
        if (i == 2) return (1.0 - s.b - k)/d;
        return k;
    }
    if (m == 2 || m == 7){
        if (i == 0) return s.r;
        if (i == 1) return s.g;
        return s.b;
    }
    if (m == 6) return (i == 0) ? l : pow(1.0 - l, 2.0)*0.8;
    return l;  // original, neon
}

// Ink colour of layer i (s = the dot's own source colour, for modes that use it).
vec3 layerInk(int m, int i, vec3 s, float hue, float tone){
    float fi = float(i);
    if (m == 0) return vec3(0.05);
    if (m == 1){
        if (i == 0) return vec3(0.0, 0.68, 0.94);
        if (i == 1) return vec3(0.93, 0.0, 0.55);
        if (i == 2) return vec3(1.0, 0.94, 0.0);
        return vec3(0.08);
    }
    if (m == 2){
        if (i == 0) return vec3(1.0, 0.1, 0.05);
        if (i == 1) return vec3(0.1, 1.0, 0.15);
        return vec3(0.1, 0.25, 1.0);
    }
    if (m == 3){
        float l = dot(s, vec3(0.299, 0.587, 0.114));
        return clamp(mix(vec3(l), s, 1.6)*1.25, 0.0, 1.0);
    }
    if (m == 4) return cosPalette(hue + fi*0.33 + tone*0.6 + TIME*0.05)*1.1;
    if (m == 5){
        vec3 ink;
        if (i == 0) ink = vec3(0.0, 1.0, 1.0);
        else if (i == 1) ink = vec3(1.0, 0.0, 1.0);
        else if (i == 2) ink = vec3(1.0, 1.0, 0.0);
        else ink = vec3(0.25, 0.0, 0.45);
        return clamp(hueRotate(ink, TIME*0.11 + fi*0.17), 0.0, 1.0);
    }
    if (m == 6){
        if (i == 0) return heatRamp(0.35 + tone*0.75);
        return mix(vec3(0.0, 0.05, 0.3), vec3(0.2, 0.9, 1.0), tone);
    }
    // spectral
    if (i == 0) return vec3(0.0, 1.0, 1.0);
    if (i == 1) return vec3(1.0, 0.0, 1.0);
    return vec3(1.0, 1.0, 0.0);
}

void main(){
    vec2 px = gl_FragCoord.xy;
    vec2 res = RENDERSIZE.xy;
    float chaos = clamp(uChaos, 0.0, 1.0);
    int mode = colorMode();
    int nLayers = layerCount(mode);
    int blend = blendMode(mode);

    // Chaos clock: glitches re-roll at an increasing rate.
    float tick = floor(TIME*(2.0 + chaos*16.0));

    // --- Chaos: horizontal block tearing ---------------------------------
    if (chaos > 0.0){
        float bandH = res.y*mix(0.12, 0.02, hash1(vec2(tick, 3.1)));
        float band = floor(px.y/bandH);
        float hb = hash1(vec2(band, tick));
        if (hb < chaos*0.45)
            px.x += (hash1(vec2(band, tick + 7.0)) - 0.5)*chaos*res.x*0.5;
    }

    // --- Cell size (chaos: mismatched screen regions) --------------------
    float cell = max(uCellSize, 2.0);
    if (chaos > 0.0){
        vec2 region = floor(px/(res*0.25)) + floor(tick*0.25)*17.0;
        float hr = hash1(region + 1.3);
        if (hr < chaos*0.7){
            float pick = floor(hash1(region + 9.7)*4.0);
            cell *= (pick == 0.0) ? 0.4 : (pick == 1.0) ? 2.0 : (pick == 2.0) ? 3.5 : 0.7;
        }
    }

    // --- Grid warp (domain warp of the screens, not of the image) --------
    vec2 gp = px;
    if (uWarp > 0.0){
        float w = uWarp*(1.0 + chaos*2.0);
        float wf = 1.0/max(res.y*0.25, 1.0);
        gp += w*cell*2.5*vec2(sin(px.y*wf*3.0 + TIME*1.3) + sin(px.y*wf*7.1 - TIME*0.7)*0.5,
                              cos(px.x*wf*2.6 - TIME*1.1) + cos(px.x*wf*6.3 + TIME*0.9)*0.5);
    }

    vec2 center = res*0.5;
    float globalAng = uAngle*TAU + TIME*uRotSpeed;
    float hue = uHueShift;
    float aa = max(uSoftness*0.75, 0.0) + 1.0/cell;
    float pulse = uPulse*(1.0 + chaos);
    float opaqueInk = smoothstep(1.0, 2.0, uDotScale);

    vec3 paper = paperColor(mode, hue);
    vec3 col = paper;
    vec3 light = vec3(0.0);
    // Opaque-ink path: across all layers, the dot this pixel sits most deeply
    // inside (relative to its radius) wins.
    float opDepth = -1e9;
    float opMask = 0.0;
    vec3 opInk = paper;

    // Chaos: random ink permutation offset for this tick.
    int swap = (hash1(vec2(tick, 42.0)) < chaos*0.35) ? int(floor(hash1(vec2(tick, 5.0))*3.0)) + 1 : 0;

    for (int li = 0; li < 4; li++){
        if (li >= nLayers) break;
        float fl = float(li);

        // Layer screen angle.
        float ang = globalAng + radians(baseAngle(mode, li))*uAngleSpread;
        if (chaos > 0.0)
            ang += (hash1(vec2(tick, fl + 11.0)) - 0.5)*chaos*PI;

        // Layer displacement (whole screen shifted), spinning over time.
        float dAng = fl*TAU/float(nLayers) + TIME*uDisplaceSpin;
        vec2 disp = uDisplace*cell*vec2(cos(dAng), sin(dAng));
        if (chaos > 0.0)
            disp += (hash2(vec2(tick, fl + 23.0)) - 0.5)*chaos*cell*4.0;

        // Source colour smear: each layer reads the image from a shifted spot.
        float sAng = fl*2.399 + TIME*0.5;
        vec2 smear = uSmear*0.15*vec2(cos(sAng), sin(sAng))*(1.0 + chaos);

        mat2 R  = rot(ang);
        mat2 Ri = rot(-ang);
        vec2 q = (R*(gp - center - disp))/cell;   // layer cell space
        vec2 base = floor(q);

        float bestMask = 0.0;
        float bestDepth = -1e9;
        vec3 bestInk = vec3(0.0);

        for (int j = -1; j <= 1; j++){
            for (int i = -1; i <= 1; i++){
                vec2 c = base + vec2(float(i), float(j));
                vec2 cc = c + 0.5;
                vec2 ch = hash2(c + fl*31.7);

                // Cell centre back in image pixels -> sample tone there.
                vec2 cpx = Ri*(cc*cell) + center + disp;
                vec2 suv = cpx/res + smear;
                if (chaos > 0.0 && ch.x < chaos*0.2)
                    suv = hash2(c + tick);             // sample shuffle
                vec3 s = IMG_NORM_PIXEL(inputImage, clamp(suv, 0.0, 1.0)).rgb;

                int inkIdx = li;
                if (swap > 0 && nLayers > 1){
                    inkIdx = li + swap;
                    if (inkIdx >= nLayers) inkIdx -= nLayers;
                }
                float tone = clamp(layerTone(mode, inkIdx, s), 0.0, 1.0);
                tone = clamp((tone - 0.5)*uContrast + 0.5, 0.0, 1.0);

                // Pulse waves rolling outward from the centre.
                if (pulse > 0.0){
                    float dist = length(cpx - center)/res.y;
                    tone *= 1.0 + pulse*1.5*sin(dist*18.0 - TIME*4.0 + fl*1.7);
                }
                // Chaos: per-cell tone flicker.
                if (chaos > 0.0)
                    tone *= 1.0 + (hash1(c + tick*0.37 + fl) - 0.5)*chaos*2.0;
                tone = max(tone, 0.0);

                // Capped at 1.5 cells - the furthest a 3x3 neighbourhood can still draw.
                float r = min(0.5*sqrt(tone)*uDotScale*1.414, 1.5);
                float sd = dotSD(q - cc, r);
                float m = 1.0 - smoothstep(-aa, aa, sd);
                if (r <= 0.0) m = 0.0;
                float depth = -sd/max(r, 1e-3);

                // Overlapping dots: the one we're deepest inside is on top.
                if (m > 0.0 && depth > bestDepth){
                    bestMask = m;
                    bestDepth = depth;
                    bestInk = layerInk(mode, inkIdx, s, hue, tone);
                }
            }
        }

        // Neon already folds the hue shift into its palette.
        vec3 ink = (mode == 4) ? bestInk : clamp(hueRotate(bestInk, hue), 0.0, 1.0);
        if (chaos > 0.0)
            ink = clamp(hueRotate(ink, (hash1(vec2(tick, fl + 77.0)) - 0.5)*chaos*0.6), 0.0, 1.0);

        if (bestMask > 0.0 && bestDepth > opDepth){
            opDepth = bestDepth;
            opMask = bestMask;
            opInk = ink;
        }

        if (blend == 0)      col *= mix(vec3(1.0), ink, bestMask);
        else if (blend == 1) light += ink*bestMask;
        else                 col = mix(col, ink, bestMask);
    }

    if (blend == 1) col += light;
    // Multiplied inks turn to mud once huge dots overlap all layers, so past
    // dot size 1 they fade into opaque screen-print inks instead.
    if (blend == 0) col = mix(col, mix(paper, opInk, opMask), opaqueInk);

    // Chaos: invert flashes.
    if (chaos > 0.0 && hash1(vec2(tick, 99.0)) < chaos*0.12)
        col = 1.0 - col;

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
