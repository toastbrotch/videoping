/*{
  "DESCRIPTION": "One curved CRT tube in a TV cabinet zapping between channels (oscilloscope Lissajous figures, drifting test cards, static), or staying on one. Channels can come with a fixed overlay (sync roll, sweep, grain, ASCII, Matrix rain, docking HUD); close up the tube shows scanlines, shadow mask, glow and a glass reflection.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "retro", "glitch", "crt"],
  "INPUTS": [
    { "NAME": "uSpeed",       "LABEL": "Speed",        "TYPE": "float", "DEFAULT": 1.84, "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uChannelRate", "LABEL": "Channel change rate (0 = never)","TYPE": "float", "DEFAULT": 0.06, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uChannel",     "LABEL": "Channel",                  "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 30.0 },
    { "NAME": "uZapFx",       "LABEL": "Channel change effect",    "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uGlitch",      "LABEL": "VHS glitch",       "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uColorNoise",  "LABEL": "Color noise",           "TYPE": "float", "DEFAULT": 0.95, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uScopeGlow",   "LABEL": "Scope glow",       "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.0,  "MAX": 1.5 },
    { "NAME": "uOverlayShare","LABEL": "Overlay share",         "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uOverlayMix",  "LABEL": "Overlay strength",    "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.0,  "MAX": 1.2 },
    { "NAME": "uHueDrift",    "LABEL": "Hue drift",          "TYPE": "float", "DEFAULT": 0.99, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uTubeSize",    "LABEL": "Tube size",        "TYPE": "float", "DEFAULT": 0.976, "MIN": 0.4,  "MAX": 1.2 },
    { "NAME": "uCurvature",   "LABEL": "Tube curvature",       "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uCabinet",     "LABEL": "Cabinet",               "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uScanlines",   "LABEL": "Scanlines",                 "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uMask",        "LABEL": "Shadow mask (RGB)",        "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uGlow",        "LABEL": "Phosphor glow",          "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uReflection",  "LABEL": "Glass reflection",        "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSeed",        "LABEL": "Random seed",           "TYPE": "float", "DEFAULT": 7.0, "MIN": 0.0,  "MAX": 100.0 },
    { "NAME": "bgAlpha", "LABEL": "Background (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// background transparency ("Background" slider < 1): straight alpha, so in the mixer (mix type A) the
// layer below shows through where only background is
vec4 bgOver(vec4 b, vec4 a) {
    float oa = a.a + b.a * (1.0 - a.a);
    return vec4((a.rgb * a.a + b.rgb * b.a * (1.0 - a.a)) / max(oa, 1e-4), oa);
}
// col = the finished pixel, bg = the background color, fgA = how much of the pixel is not background
vec4 withBackground(vec3 col, vec3 bg, float fgA) {
    if (bgAlpha >= 0.999) return vec4(col, 1.0);
    fgA = clamp(fgA, 0.0, 1.0);
    vec3 fg = fgA > 1e-4 ? (col - bg * (1.0 - fgA)) / fgA : col;
    return bgOver(vec4(bg, bgAlpha), vec4(clamp(fg, 0.0, 1.0), fgA));
}

// CRT TV V2 — crt-tv.fs with the overlay chosen per channel (screenOverlay). Single-tube
// version of crt-tvwall-v2.fs (GLSL ES 1.00: literal loop bounds,
// gl_FragColor, TIME/RENDERSIZE). Self-contained, no image input required.
//
// Coordinates: p = centred frame coords (x scaled by aspect). The tube is p/uTubeSize, bent
// by tubeWarp; uv01 = 0..1 position on the (warped) picture, which every channel and overlay
// draws in, so overlays curve with the glass. cpx = uv01*RENDERSIZE is the same in pixels.

#define PI 3.14159265

float gT, gGlitch, gColorNoise, gCurvature, gGlow, gSeed, gAspect;

// ---------------------------------------------------------------- hashing --
float hash11(float n){ return fract(sin(n)*43758.5453); }
float hash21(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }
float sat(float x){ return clamp(x, 0.0, 1.0); }

float segDist(vec2 p, vec2 a, vec2 b){
    vec2 pa = p - a, ba = b - a;
    return length(pa - ba*clamp(dot(pa, ba)/max(dot(ba, ba), 1e-6), 0.0, 1.0));
}

// ------------------------------------------------------------ tube shape --
// Barrel bulge, normalised so the tube's corners stay put whatever the curvature: the
// screen only bends towards the middle instead of the whole tube shrinking.
vec2 tubeWarp(vec2 p){
    float k = gCurvature*0.30;
    vec2 corner = vec2(gAspect*0.92, 0.90);
    return p*(1.0 + k*dot(p, p))/(1.0 + k*dot(corner, corner));
}
float tubeDist(vec2 p){
    vec2 b = vec2(gAspect*0.92, 0.90);
    float r = 0.12;
    vec2 d = abs(p) - b + r;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

// ------------------------------------------------------- oscilloscope --
vec3 scopeCell(vec2 p, float seed){
    float fa = 2.0 + floor(hash11(seed*3.1 + 1.0)*4.0);
    float fb = 2.0 + floor(hash11(seed*4.7 + 2.0)*4.0);
    float ph = hash11(seed*5.3 + 3.0)*6.2831;
    float spd = 0.5 + hash11(seed*6.1 + 4.0)*0.6;

    vec3 phosGreen = vec3(0.25, 1.0, 0.35);
    vec3 phosAmber = vec3(1.0, 0.55, 0.08);
    vec3 phos = hash11(seed*7.7 + 5.0) > 0.5 ? phosGreen : phosAmber;

    vec2 g = abs(fract(p*2.5) - 0.5);
    float grid = smoothstep(0.02, 0.0, min(g.x, g.y));
    float axis = smoothstep(0.015, 0.0, min(abs(p.x), abs(p.y)));
    vec3 col = phos*0.05*grid + phos*0.10*axis;

    // Distance to the Lissajous curve as a closed chain of segments, so the trace stays a
    // continuous line at full-screen size (the wall's 40 sample points read as dots here).
    float minD = 10.0;
    float minD2 = 10.0;
    vec2 prev  = vec2(sin(ph + gT*spd), sin(gT*spd*0.7));
    vec2 prev2 = vec2(sin(ph + (gT-0.10)*spd), sin((gT-0.10)*spd*0.7));
    for (int i = 1; i <= 96; i++){
        float u = float(i)/96.0*2.0*PI;
        vec2 q  = vec2(sin(fa*u + ph + gT*spd),        sin(fb*u + gT*spd*0.7));
        vec2 q2 = vec2(sin(fa*u + ph + (gT-0.10)*spd), sin(fb*u + (gT-0.10)*spd*0.7));
        minD  = min(minD,  segDist(p, prev, q));
        minD2 = min(minD2, segDist(p, prev2, q2));
        prev = q; prev2 = q2;
    }
    float trace = 0.006/(minD*minD*40.0 + 0.006);
    float trail = 0.006/(minD2*minD2*45.0 + 0.006)*0.35;
    col += phos*(trace + trail)*gGlow;
    return col;
}

// -------------------------------------------------------------- test-card --
vec3 smpteBars(float x, float y){
    vec3 col;
    if (y > 0.33){
        float idx = floor(x*7.0);
        if      (idx < 1.0) col = vec3(0.75);
        else if (idx < 2.0) col = vec3(0.75, 0.75, 0.0);
        else if (idx < 3.0) col = vec3(0.0, 0.75, 0.75);
        else if (idx < 4.0) col = vec3(0.0, 0.75, 0.0);
        else if (idx < 5.0) col = vec3(0.75, 0.0, 0.75);
        else if (idx < 6.0) col = vec3(0.75, 0.0, 0.0);
        else                col = vec3(0.0, 0.0, 0.75);
    } else if (y > 0.25){
        float idx = floor(x*7.0);
        if      (idx < 1.0) col = vec3(0.0, 0.0, 0.75);
        else if (idx < 2.0) col = vec3(0.05);
        else if (idx < 3.0) col = vec3(0.75, 0.0, 0.75);
        else if (idx < 4.0) col = vec3(0.05);
        else if (idx < 5.0) col = vec3(0.0, 0.75, 0.75);
        else if (idx < 6.0) col = vec3(0.05);
        else                col = vec3(0.75);
    } else {
        if      (x < 0.15) col = vec3(0.0, 0.10, 0.30);
        else if (x < 0.30) col = vec3(0.05);
        else if (x < 0.45) col = vec3(0.75);
        else if (x < 0.60) col = vec3(0.15, 0.0, 0.30);
        else {
            float px = (x - 0.60)/0.40;
            float pidx = floor(px*4.0);
            if      (pidx < 1.0) col = vec3(0.02);
            else if (pidx < 2.0) col = vec3(0.05);
            else if (pidx < 3.0) col = vec3(0.08);
            else                 col = vec3(0.75);
        }
    }
    return col;
}
vec3 resChart(vec2 p, vec2 uv01){
    vec3 col;
    if (uv01.y > 0.5){
        float steps = 10.0;
        float idx = floor(uv01.x*steps);
        col = vec3(idx/(steps - 1.0));
    } else {
        float r = length(p);
        float ring = step(0.5, fract(r*8.0));
        col = vec3(0.05) + vec3(ring)*0.65;
        float cross = 1.0 - smoothstep(0.0, 0.01, min(abs(p.x), abs(p.y)));
        col = mix(col, vec3(1.0, 0.82, 0.1), cross);
    }
    return col;
}
vec3 testcardCell(vec2 uv01, vec2 p, float seed){
    float variant = hash11(seed*21.0 + 6.0);
    float drift = sin(gT*0.11 + seed*10.0)*0.02 + gT*0.004*sin(seed*3.0);
    float jumpH = hash21(vec2(seed, floor(gT/2.2)));
    float jump = step(0.85, jumpH)*(hash11(seed*8.0 + floor(gT/2.2)) - 0.5)*0.5;
    float dx = drift + jump;

    if (variant < 0.6) return smpteBars(fract(uv01.x + dx + 1.0), uv01.y);
    return resChart(p, uv01);
}

// ------------------------------------------------------------- VHS glitch --
vec3 vhsGlitch(vec3 color, vec2 uv01, float seed, float burst){
    float rowH = hash21(vec2(floor(uv01.y*60.0), floor(gT*2.0 + seed*10.0)));
    float tracking = step(0.975 - gGlitch*0.12, rowH)*burst;   // rarer than on the wall: one big screen
    color = mix(color, vec3(1.0), tracking*0.55);

    vec2 blk = floor(uv01*vec2(28.0, 20.0));
    float blkH = hash21(blk + floor(gT*6.0 + seed*30.0));
    float dropout = step(0.992 - gGlitch*0.06, blkH)*burst;
    float dn = hash21(uv01*900.0 + gT);
    color = mix(color, vec3(dn), dropout);

    float cn = (hash21(uv01*1600.0 + gT*3.0 + seed) - 0.5)*gColorNoise;
    color.r = sat(color.r + cn*0.8);
    color.g = sat(color.g - cn*0.3);
    color.b = sat(color.b + cn*0.9);

    float speck = step(0.995 - gColorNoise*0.3, hash21(uv01*1800.0 - gT*7.0 + seed));
    color = mix(color, vec3(hash21(uv01*2000.0 + gT)), speck*0.8);
    return color;
}

// --------------------------------------------------------- channel director --
// x = channel number, y = phase 0..1 within the channel. uChannel picks the channel; at a
// channel rate of 0 the TV stays on it for good (mid-phase, so no zap effect ever fires),
// otherwise it's where the zapping starts from.
vec2 channelClock(){
    float ch = floor(max(uChannel, 0.0));
    if (uChannelRate <= 0.001) return vec2(ch, 0.5);
    float period = mix(30.0, 3.0, uChannelRate);
    float t = gT + hash11(gSeed)*50.0;
    return vec2(ch + floor(t/period), fract(t/period));
}

// Picture of the current channel at picture position uv01 (no tube effects yet).
vec3 channelPicture(vec2 uv01){
    vec2 cc = channelClock();
    float seed = hash21(vec2(cc.x*1.37, gSeed));
    vec2 p = (uv01*2.0 - 1.0)*vec2(gAspect, 1.0);
    float modeH = hash11(seed*9.1 + 2.0);

    vec3 col;
    float burst = 0.15 + 0.5*hash11(seed*9.0 + 8.0);
    if (modeH < 0.45){
        col = scopeCell(p*1.1, seed);
    } else if (modeH < 0.85){
        col = testcardCell(uv01, p, seed);
    } else {
        float n = hash21(floor(uv01*vec2(RENDERSIZE.x*0.5, RENDERSIZE.y*0.5)) + gT*20.0 + seed);
        col = vec3(n)*0.9 + vec3(0.05, 0.08, 0.12);
        burst = max(burst, 0.8);
    }
    col = vhsGlitch(col, uv01, seed, sat(burst*0.5 + gGlitch*0.3));
    return col;
}

float pictureLuma(vec2 uv01){
    return dot(channelPicture(uv01), vec3(0.299, 0.587, 0.114));
}

// --------------------------------------------------------- ASCII overlay --
// Monospace ASCII/terminal conversion of the live picture: each block samples one stable
// brightness and picks a procedural glyph from an emptiest-to-fullest ramp (space . - + / x o #).
vec3 asciiOverlay(vec2 cpx){
    float cellPx = 12.0;
    vec2 blockID = floor(cpx/cellPx);
    float lum = pictureLuma((blockID + 0.5)*cellPx/RENDERSIZE);
    vec2 q = (fract(cpx/cellPx) - 0.5)*2.0;

    float ink = 0.0;
    if (lum < 0.12){
        ink = 0.0;
    } else if (lum < 0.28){
        ink = 1.0 - smoothstep(0.0, 0.35, length(q));
    } else if (lum < 0.42){
        ink = (1.0 - smoothstep(0.0, 0.30, abs(q.y)))*step(abs(q.x), 0.7);
    } else if (lum < 0.56){
        float h = (1.0 - smoothstep(0.0, 0.28, abs(q.y)))*step(abs(q.x), 0.7);
        float v = (1.0 - smoothstep(0.0, 0.28, abs(q.x)))*step(abs(q.y), 0.7);
        ink = max(h, v);
    } else if (lum < 0.70){
        float d = abs(q.x - q.y)/1.41421356;
        ink = (1.0 - smoothstep(0.0, 0.28, d))*step(length(q), 0.9);
    } else if (lum < 0.82){
        float d1 = abs(q.x - q.y)/1.41421356;
        float d2 = abs(q.x + q.y)/1.41421356;
        ink = max(1.0 - smoothstep(0.0, 0.28, d1), 1.0 - smoothstep(0.0, 0.28, d2))*step(length(q), 0.9);
    } else if (lum < 0.92){
        float m = max(abs(q.x), abs(q.y));
        ink = 1.0 - smoothstep(0.0, 0.10, abs(m - 0.75));
    } else {
        ink = step(max(abs(q.x), abs(q.y)), 0.85);
    }
    return vec3(0.25, 1.0, 0.4)*ink;
}

// -------------------------------------------------------- Matrix rain --
vec3 matrixOverlay(vec2 cpx){
    float colW = 16.0;
    float colId = floor(cpx.x/colW);
    float colSeed = hash11(colId*17.13 + 3.7);
    float rowH = 18.0;
    float row = floor(cpx.y/rowH);

    float fallSpeed = mix(3.0, 10.0, colSeed);
    float streamLen = mix(5.0, 18.0, hash11(colId*5.5 + 1.0));
    float phase = row - gT*fallSpeed;
    float posInStream = mod(phase, streamLen);
    float bright = max(1.0 - posInStream/streamLen, 0.0);
    bright *= step(0.15, hash21(vec2(colId, row) + floor(gT*10.0)));

    // A blocky glyph in each cell, so the rain reads as characters on the big screen.
    vec2 g = fract(cpx/vec2(colW, rowH));
    vec2 gi = floor(g*vec2(3.0, 4.0));
    float glyph = step(0.45, hash21(gi + vec2(colId, row)*7.3 + floor(gT*3.0)))
                * step(0.12, g.x)*step(g.x, 0.88)*step(0.1, g.y)*step(g.y, 0.9);

    float headMask = smoothstep(0.85, 1.0, bright);
    vec3 green = vec3(0.1, 1.0, 0.3);
    vec3 headC = vec3(0.85, 1.0, 0.9);
    return mix(green, headC, headMask)*bright*glyph;
}

// ------------------------------------------------------- docking HUD --
// Soyuz/spacecraft docking-camera overlay: alignment rings, a wandering crosshair (a probe
// hunting for lock), corner brackets, blinking telemetry boxes and a slow radar sweep.
vec3 dockingOverlay(vec2 uv01){
    vec2 wc = uv01*2.0 - 1.0;
    wc.x *= gAspect;
    vec3 hc = vec3(0.25, 1.0, 0.65);
    vec3 hud = vec3(0.0);

    float r = length(wc);
    hud += hc*(1.0 - smoothstep(0.0, 0.006, abs(r - 0.55)));
    hud += hc*(1.0 - smoothstep(0.0, 0.006, abs(r - 0.30)));

    vec2 drift = vec2(sin(gT*0.37)*0.05, cos(gT*0.29)*0.04);
    vec2 wcd = wc - drift;
    float armLen = 0.9; float gap = 0.06; float armW = 0.004;
    float hLine = step(abs(wcd.y), armW)*step(gap, abs(wcd.x))*step(abs(wcd.x), armLen);
    float vLine = step(abs(wcd.x), armW)*step(gap, abs(wcd.y))*step(abs(wcd.y), armLen);
    hud += hc*(hLine + vLine);
    hud += hc*0.6*(1.0 - smoothstep(0.0, 0.015, length(wc)));

    vec2 ac = abs(wc);
    vec2 ca = vec2(gAspect*0.85, 0.82);
    float bl = 0.15; float bw = 0.01;
    float legH = step(abs(ac.x - ca.x), bw)*step(ac.y, ca.y)*step(ca.y - bl, ac.y);
    float legV = step(abs(ac.y - ca.y), bw)*step(ac.x, ca.x)*step(ca.x - bl, ac.x);
    hud += hc*(legH + legV);

    float blinkA = step(0.5, fract(gT*2.3));
    vec2 tl = wc - vec2(-ca.x + 0.12, ca.y - 0.08);
    hud += hc*step(abs(tl.x), 0.05)*step(abs(tl.y), 0.02)*blinkA*1.2;
    float blinkB = step(0.5, fract(gT*1.7 + 0.5));
    vec2 tr = wc - vec2(ca.x - 0.12, ca.y - 0.08);
    hud += hc*step(abs(tr.x), 0.05)*step(abs(tr.y), 0.02)*blinkB*1.2;

    float ang = atan(wc.y, wc.x);
    float sweepAng = mod(gT*0.6, 6.2831853) - 3.14159265;
    float angDiff = abs(mod(ang - sweepAng + 3.14159265, 6.2831853) - 3.14159265);
    hud += hc*(1.0 - smoothstep(0.0, 0.05, angDiff))*step(r, 0.55)*0.5;
    return hud;
}

// ---------------------------------------------------------------- hue drift --
vec3 hueRotate(vec3 v, float a){
    float c = cos(a), s = sin(a);
    float r = v.r*(0.299 + 0.701*c + 0.168*s) + v.g*(0.587 - 0.587*c + 0.330*s) + v.b*(0.114 - 0.114*c - 0.497*s);
    float g = v.r*(0.299 - 0.299*c - 0.328*s) + v.g*(0.587 + 0.413*c + 0.035*s) + v.b*(0.114 - 0.114*c + 0.292*s);
    float b = v.r*(0.299 - 0.300*c + 1.250*s) + v.g*(0.587 - 0.588*c - 1.050*s) + v.b*(0.114 + 0.886*c - 0.203*s);
    return clamp(vec3(r, g, b), 0.0, 1.0);
}

// -------------------------------------------------------------- screen FX --
// Each channel is dealt one overlay when it's rolled, like its picture: a uOverlayShare
// fraction of channels get one of the six looks (evenly), the rest none. It stays as long as
// the channel does and changes with it.
vec3 screenOverlay(vec3 col, vec2 uv01, vec2 cpx){
    vec2 cc = channelClock();
    float seed = hash21(vec2(cc.x*1.37, gSeed));
    float fxCyc = cc.x;
    float fxMode = 0.0;
    if (hash11(seed*31.7 + 5.0) < uOverlayShare)
        fxMode = 1.0 + floor(hash11(seed*43.3 + 1.0)*6.0);   // 1 sync-roll, 2 sweep, 3 grain,
                                                            // 4 ASCII, 5 Matrix, 6 docking HUD
    float fade = uOverlayMix;

    if (fxMode == 1.0){
        float rollY = fract(gT*0.12 + fxCyc*0.37);
        float band = 1.0 - smoothstep(0.0, 0.02, abs(uv01.y - rollY));
        col = mix(col, vec3(1.0), band*fade*0.8);
        col *= 1.0 + 0.12*sin(gT*45.0)*fade;
    } else if (fxMode == 2.0){
        float diag = (uv01.x + uv01.y)*0.5;
        float band = 1.0 - smoothstep(0.0, 0.06, abs(diag - fract(gT*0.15)));
        col += vec3(0.6, 0.7, 0.8)*band*fade*0.5;
    } else if (fxMode == 3.0){
        col += (hash21(floor(cpx) + floor(gT*24.0)) - 0.5)*0.25*fade;
    } else if (fxMode == 4.0){
        col = mix(col, col*0.15 + asciiOverlay(cpx), fade);
    } else if (fxMode == 5.0){
        col = mix(col, col*0.15 + matrixOverlay(cpx), fade);
    } else if (fxMode == 6.0){
        col = mix(col, col*0.6 + dockingOverlay(uv01), fade);
    }
    return col;
}

// ------------------------------------------------------------------ main --
void main(){
    vec2 res = RENDERSIZE.xy;
    gAspect = res.x/res.y;
    gT = TIME*uSpeed;
    gGlitch = uGlitch; gColorNoise = uColorNoise; gCurvature = uCurvature;
    gGlow = uScopeGlow; gSeed = uSeed;

    vec2 frameUV = gl_FragCoord.xy/res;
    vec2 p = (frameUV*2.0 - 1.0)*vec2(gAspect, 1.0);

    // Tube geometry: scaled by the tube size, bent by the curvature.
    vec2 tp = p/max(uTubeSize, 0.05);
    vec2 tw = tubeWarp(tp);
    float d = tubeDist(tw);
    float inside = 1.0 - smoothstep(-0.01, 0.01, d);
    vec2 uv01 = (tw/vec2(gAspect*0.92, 0.90))*0.5 + 0.5;

    // Channel change: the picture collapses into a bright horizontal line at the end of a
    // channel, a short static burst follows, then it opens out again on the next channel.
    vec2 cc = channelClock();
    float zap = clamp(uZapFx, 0.0, 1.0);
    float closeT = smoothstep(0.96, 1.0, cc.y);           // 0 -> 1 while closing
    float openT = 1.0 - smoothstep(0.0, 0.05, cc.y);      // 1 -> 0 while opening
    float squash = max(closeT, openT)*zap;
    float burst = (1.0 - smoothstep(0.0, 0.08, cc.y))*zap;

    vec2 puv = uv01;
    puv.y = 0.5 + (puv.y - 0.5)/max(1.0 - squash*0.995, 0.005);
    vec3 col = channelPicture(clamp(puv, 0.0, 1.0));
    col *= step(abs(puv.y - 0.5), 0.5);                   // outside the squashed picture: dark
    col += vec3(1.0)*squash*squash*(1.0 - smoothstep(0.0, 0.004 + 0.3*(1.0 - squash), abs(uv01.y - 0.5)))*2.0;
    float zn = hash21(floor(uv01*vec2(res.x*0.4, res.y*0.4)) + floor(gT*30.0));
    col = mix(col, vec3(zn)*0.9, burst*0.85*step(squash, 0.4));

    col = screenOverlay(col, uv01, uv01*res);
    col = hueRotate(col, sin(gT*0.11)*uHueDrift*PI);

    // Tube anatomy: phosphor glow, scanlines, RGB shadow mask, vignette, glass reflection.
    col += col*col*uGlow*0.6;
    float scan = 0.5 + 0.5*cos(uv01.y*res.y*0.45*2.0*PI);
    col *= mix(1.0, 0.55 + 0.45*scan, uScanlines);
    float triad = mod(floor(gl_FragCoord.x), 3.0);
    vec3 mask = triad < 1.0 ? vec3(1.0, 0.45, 0.45) : (triad < 2.0 ? vec3(0.45, 1.0, 0.45) : vec3(0.45, 0.45, 1.0));
    col *= mix(vec3(1.0), mask*1.35, uMask);
    col *= 1.0 - 0.45*smoothstep(0.3, 1.2, length(tw/vec2(gAspect, 1.0)));
    vec2 rp = tw - vec2(-gAspect*0.45, 0.55);
    float refl = exp(-dot(rp*vec2(0.7, 1.4), rp*vec2(0.7, 1.4))*3.0);
    col += vec3(0.85, 0.9, 1.0)*refl*uReflection*0.25;

    // Cabinet: dark plastic around the tube, a lit inner bezel edge, and a power LED that
    // flashes red during a channel change.
    vec3 cab = mix(vec3(0.015), vec3(0.09, 0.085, 0.08), uCabinet)*(0.75 + 0.25*frameUV.y);
    cab += vec3(0.18)*uCabinet*(1.0 - smoothstep(0.0, 0.03, abs(d - 0.02)))*0.5;
    vec2 ledPos = vec2(gAspect*0.92, -0.90)*min(uTubeSize, 1.0) + vec2(0.0, -0.06);
    float led = 1.0 - smoothstep(0.008, 0.016, length(p - ledPos));
    float zapping = step(0.01, burst + squash);
    vec3 ledCol = mix(vec3(0.1, 0.9, 0.2), vec3(1.0, 0.1, 0.05), zapping);
    float ledOn = mix(1.0, step(0.5, fract(gT*8.0)), zapping);
    cab += ledCol*led*uCabinet*ledOn;

    vec3 final = mix(cab, col, inside);
    // the cabinet around the tube is the background
    gl_FragColor = withBackground(clamp(final, 0.0, 1.0), clamp(cab, 0.0, 1.0), inside);
}
