/*{
  "DESCRIPTION": "Neon lines emitted at the bottom travel upward, flat or as a perspective floor to a synthwave horizon. Each line keeps the wave it recorded when it was emitted (sine, seismic, audio, digital, zigzag, heartbeat), so the floor shows the history of the signal. Sun and sky, floor grid, eight palettes, Weird and Glitch.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "synthwave", "lines", "glitch"],
  "INPUTS": [
    { "NAME": "uCadence",     "LABEL": "Rate (lines/s)",     "TYPE": "float", "DEFAULT": 4.0,  "MIN": 0.2,  "MAX": 40.0 },
    { "NAME": "uSpeed",       "LABEL": "Speed (<0 = downwards)", "TYPE": "float", "DEFAULT": 1.0,  "MIN": -4.0, "MAX": 4.0 },
    { "NAME": "uPerspective", "LABEL": "Perspective",           "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uHorizon",     "LABEL": "Horizon height",        "TYPE": "float", "DEFAULT": 0.55, "MIN": 0.15, "MAX": 0.95 },
    { "NAME": "uWaveType",    "LABEL": "Wave type (0-5)",      "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 5.0 },
    { "NAME": "uWaveAmp",     "LABEL": "Wave height",          "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uWaveFreq",    "LABEL": "Wave frequency",       "TYPE": "float", "DEFAULT": 6.0,  "MIN": 0.5,  "MAX": 40.0 },
    { "NAME": "uWaveMotion",  "LABEL": "Wave motion",       "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 6.0 },
    { "NAME": "uOcclusion",   "LABEL": "Hidden lines",            "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uThickness",   "LABEL": "Line width (px)",     "TYPE": "float", "DEFAULT": 2.0,  "MIN": 0.5,  "MAX": 24.0 },
    { "NAME": "uGlow",        "LABEL": "Glow",                  "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 3.0 },
    { "NAME": "uGridX",       "LABEL": "Floor grid",          "TYPE": "float", "DEFAULT": 12.0, "MIN": 0.0,  "MAX": 60.0 },
    { "NAME": "uPalette",     "LABEL": "Palette (0-7)",         "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 7.0 },
    { "NAME": "uSun",         "LABEL": "Sun",                 "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uWeird",       "LABEL": "Weird",                 "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uGlitch",      "LABEL": "Glitch",                "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
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

// SYNTHWAVE LINES — line emitter generator. GLSL ES 1.00 / ISF v2 style: gl_FragColor,
// TIME/RENDERSIZE, literal loop bounds. Self-contained, no image input.
//
// Coordinates: y = 0..1 screen height, x centred and in screen-height units. Lines live in a
// travel coordinate s (s = 0 at the bottom, where lines are emitted). The screen height of a
// line is   y(s) = mix(s, h*s/(s + h), perspective)   - flat at 0, a true perspective floor
// receding to the horizon h at 1 - and sOf() is its exact inverse. Each line's wave is the
// signal at the moment it was emitted, so the floor shows the signal's history.
//
// Wave types (uWaveType, morphing): 0 sine, 1 seismic, 2 audio, 3 digital, 4 zigzag, 5 heartbeat
// Palettes (uPalette, rounded): 0 Synthwave, 1 Outrun, 2 Vaporwave, 3 Matrix, 4 Ice, 5 Fire,
//                               6 Mono, 7 Rainbow

#define TAU 6.2831853

float gT;       // scene time (glitch can stutter it per band)
float gTick;    // glitch clock
float gH;       // horizon height for this pixel (weird bends it)

float hash11(float n){ return fract(sin(n*127.1 + 3.7)*43758.5453); }
float hash21(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }

float vnoise(float x){
    float i = floor(x), f = fract(x);
    f = f*f*(3.0 - 2.0*f);
    return mix(hash11(i), hash11(i + 1.0), f)*2.0 - 1.0;
}

vec3 hueRotate(vec3 c, float h){
    float a = h*TAU;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}

vec3 hsv(float h){
    return clamp(abs(mod(h*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
}

// ---- perspective mapping between travel coordinate s and screen y ----
float yOf(float s, float h, float p){
    return mix(s, h*s/(s + h), p);
}
// Exact inverse of yOf (stable form of the quadratic's root); huge above a full-perspective horizon.
float sOf(float y, float h, float p){
    float a = h - y;
    float den = a + sqrt(max(a*a + 4.0*(1.0 - p)*y*h, 0.0));
    return (den > 1e-5) ? 2.0*y*h/den : 1e5;
}
// Projection scale at distance s: 1 at the bottom, shrinking toward the horizon.
float projScale(float s, float h, float p){
    return mix(1.0, h/(s + h), p);
}

// ---- wave shapes (x = floor-x * frequency, fx = floor-x, te = emission time) ----
float shape(int ty, float x, float fx, float te){
    if (ty == 0) return sin(x + te*1.3);
    if (ty == 1){                                                   // seismic burst
        float jag = vnoise(x*1.7 + te*9.0)*0.6 + vnoise(x*4.3 - te*13.0)*0.4;
        float c = sin(te*0.37)*0.6;
        float burst = pow(0.5 + 0.5*sin(te*0.9), 3.0);
        float env = 0.08 + burst*exp(-abs(fx - c)*2.5)*1.6;
        return jag*env;
    }
    if (ty == 2){                                                   // audio / oscilloscope
        float beat = 0.3 + 0.7*exp(-fract(te*2.0)*4.0);
        float s = sin(x)*0.5 + sin(x*2.03 + te*3.0)*0.3 + sin(x*5.1 - te*5.0)*0.2;
        return s*beat*exp(-fx*fx*1.2)*1.4;
    }
    if (ty == 3)                                                    // digital steps
        return floor(hash11(floor(x*0.5) + floor(te*4.0)*13.0)*4.0)/1.5 - 1.0;
    if (ty == 4)                                                    // zigzag
        return abs(fract(x/TAU + te*0.2)*2.0 - 1.0)*2.0 - 1.0;
    float q = fract(x/TAU*0.5 - te*0.3);                            // heartbeat
    return exp(-pow((q - 0.5)*40.0, 2.0))*1.3 - exp(-pow((q - 0.47)*30.0, 2.0))*0.3
         - exp(-pow((q - 0.53)*30.0, 2.0))*0.35 + exp(-pow((q - 0.72)*12.0, 2.0))*0.25;
}

float waveAt(float fx, float te){
    float wt = clamp(uWaveType, 0.0, 5.0);
    int t0 = int(floor(wt));
    int t1 = int(min(floor(wt) + 1.0, 5.0));
    float x = fx*uWaveFreq;
    return mix(shape(t0, x, fx, te), shape(t1, x, fx, te), fract(wt));
}

// ---- palettes ----
void palette(int p, out vec3 lineNear, out vec3 lineFar, out vec3 skyTop, out vec3 skyBot,
             out vec3 sunTop, out vec3 sunBot, out vec3 floorC){
    if (p == 1){        // outrun
        lineNear = vec3(1.0, 0.45, 0.05); lineFar = vec3(1.0, 0.1, 0.6);
        skyTop = vec3(0.08, 0.0, 0.18); skyBot = vec3(0.9, 0.25, 0.2);
        sunTop = vec3(1.0, 0.95, 0.3); sunBot = vec3(1.0, 0.25, 0.1); floorC = vec3(0.06, 0.0, 0.08);
    } else if (p == 2){ // vaporwave
        lineNear = vec3(0.4, 1.0, 0.9); lineFar = vec3(1.0, 0.55, 0.9);
        skyTop = vec3(0.35, 0.3, 0.75); skyBot = vec3(1.0, 0.65, 0.85);
        sunTop = vec3(1.0, 0.85, 0.5); sunBot = vec3(1.0, 0.4, 0.75); floorC = vec3(0.15, 0.08, 0.3);
    } else if (p == 3){ // matrix
        lineNear = vec3(0.3, 1.0, 0.4); lineFar = vec3(0.0, 0.5, 0.15);
        skyTop = vec3(0.0); skyBot = vec3(0.0, 0.15, 0.05);
        sunTop = vec3(0.7, 1.0, 0.7); sunBot = vec3(0.0, 0.6, 0.2); floorC = vec3(0.0, 0.03, 0.0);
    } else if (p == 4){ // ice
        lineNear = vec3(0.85, 1.0, 1.0); lineFar = vec3(0.1, 0.5, 1.0);
        skyTop = vec3(0.0, 0.02, 0.1); skyBot = vec3(0.1, 0.35, 0.6);
        sunTop = vec3(1.0); sunBot = vec3(0.4, 0.8, 1.0); floorC = vec3(0.0, 0.02, 0.06);
    } else if (p == 5){ // fire
        lineNear = vec3(1.0, 0.9, 0.3); lineFar = vec3(0.9, 0.1, 0.0);
        skyTop = vec3(0.05, 0.0, 0.0); skyBot = vec3(0.5, 0.08, 0.0);
        sunTop = vec3(1.0, 1.0, 0.6); sunBot = vec3(1.0, 0.3, 0.0); floorC = vec3(0.05, 0.0, 0.0);
    } else if (p == 6){ // mono
        lineNear = vec3(1.0); lineFar = vec3(0.6);
        skyTop = vec3(0.0); skyBot = vec3(0.12);
        sunTop = vec3(1.0); sunBot = vec3(0.5); floorC = vec3(0.0);
    } else {            // synthwave (and rainbow's backdrop)
        lineNear = vec3(1.0, 0.15, 0.8); lineFar = vec3(0.2, 0.85, 1.0);
        skyTop = vec3(0.04, 0.0, 0.14); skyBot = vec3(0.55, 0.05, 0.45);
        sunTop = vec3(1.0, 0.9, 0.25); sunBot = vec3(1.0, 0.1, 0.55); floorC = vec3(0.03, 0.0, 0.08);
    }
}

// All lines + floor grid at (x, y). rgb is additive light, a is how much of the backdrop the
// nearer ridges hide (occlusion fill).
vec4 lineLayer(float x, float y, int pal, float chroma){
    float H = RENDERSIZE.y;
    float h = gH;
    float p = clamp(uPerspective, 0.0, 1.0);
    float w = clamp(uWeird, 0.0, 1.0);
    float g = clamp(uGlitch, 0.0, 1.0);

    vec3 lineNear, lineFar, skyTop, skyBot, sunTop, sunBot, floorC;
    palette(pal, lineNear, lineFar, skyTop, skyBot, sunTop, sunBot, floorC);

    // Emission: lines leave s = 0 at uCadence per second, travelling vs per second.
    float vs = uSpeed*0.25;
    float spacing = max(abs(vs), 0.05)/max(uCadence, 0.05);
    float Q = gT*vs/spacing;
    float off = fract(Q)*spacing;
    float baseId = floor(Q);

    float amp = uWaveAmp*0.35;
    float ampMax = amp*(1.0 + 2.0*g) + w*0.35;
    float wPx = max(uThickness, 0.5);
    float glowR = 2.0 + wPx*1.5;
    float margin = (wPx + glowR*4.0)/H + chroma;

    float sPix = sOf(clamp(y, -1.0, 2.0), h, p);
    float sLo = sOf(y - ampMax - margin, h, p);
    float kStart = max(ceil((sLo - off)/spacing), floor((min(sPix, 1e4) - off)/spacing) - 24.0);
    float sHi = sOf(y + ampMax + margin, h, p);

    vec3 acc = vec3(0.0);
    float vis = 1.0;
    float fill = 0.0;
    float occ = clamp(uOcclusion, 0.0, 1.0);

    for (int i = 0; i < 32; i++){
        float k = kStart + float(i);
        float s = k*spacing + off;
        if (s > sHi || s > 1e4) break;
        if (s < -1e-4) continue;                  // not emitted yet

        float id = baseId - k;
        float age = s/max(abs(vs), 0.01);
        float te = (gT - age*sign(vs + 1e-6))*uWaveMotion*0.5;
        float hk = hash11(id*1.31 + 0.7);

        float yk = yOf(s, h, p);
        float ps = projScale(s, h, p);
        float fxScale = 1.0/ps;                   // floor-x per screen-x at this distance
        float fx = x*fxScale;

        // Glitch: line dropouts, flashes and amplitude spikes, re-rolled every tick.
        float gr = hash21(vec2(id, gTick));
        if (g > 0.0 && gr < g*0.25) continue;
        float flash = (g > 0.0 && gr > 1.0 - g*0.08) ? 1.0 : 0.0;
        float spike = (g > 0.0 && hash21(vec2(id + 7.0, gTick)) < g*0.2) ? 1.0 + 2.0*g : 1.0;

        // Weird: melting lines (bend grows with age) and valley walls rising at the sides.
        float e = 1.5/H;
        float d0 = waveAt(fx, te);
        float d1 = waveAt((x + e)*fxScale, te);
        float bendW = w*(sin(age*0.8 + hk*TAU)*0.6*fx*fx + 0.4*max(abs(fx) - 0.35, 0.0));
        float dispC = clamp((amp*spike*d0 + bendW)*ps, -ampMax, ampMax);
        float dispR = clamp((amp*spike*d1 + w*(sin(age*0.8 + hk*TAU)*0.6*(fx + e*fxScale)*(fx + e*fxScale)
                     + 0.4*max(abs(fx + e*fxScale) - 0.35, 0.0)))*ps, -ampMax, ampMax);
        float slope = (dispR - dispC)/e;
        float curve = yk + dispC;
        float norm = inversesqrt(1.0 + slope*slope);

        // Distance per colour channel - glitch splits them vertically.
        vec3 dy = abs(vec3(y + chroma, y, y - chroma) - curve)*H*norm;
        float wl = max(wPx*mix(1.0, ps, 0.7*p), 0.6)*(1.0 + flash*2.0);
        vec3 core = 1.0 - smoothstep(wl*0.5 - 0.75, wl*0.5 + 0.75, dy);
        vec3 glow = exp(-dy/glowR)*uGlow*0.45;

        // Each line's light is weighted by the room it has, so where lines pack together
        // (near the horizon, or very thick/glowy lines) they settle into an even haze
        // instead of summing to white.
        float spacingPx = spacing*mix(1.0, h*h/((s + h)*(s + h)), p)*H;
        float densCore = clamp(spacingPx/(wl + 1.0), 0.02, 1.0);
        float densGlow = clamp(spacingPx/(glowR*uGlow*1.5 + 1.0), 0.02, 1.0);

        vec3 lc = (pal == 7) ? hsv(fract(id*0.071 + gT*0.08)) : mix(lineNear, lineFar, 1.0 - exp(-s*0.9));
        if (w > 0.0) lc = clamp(hueRotate(lc, w*0.35*sin(id*0.37 + gT*0.6)), 0.0, 1.0);
        lc = mix(lc, vec3(1.0), flash);

        acc += lc*(core*densCore + glow*densGlow)*vis;

        // Hidden-line removal: once this ridge rises above the pixel, everything behind it is hidden.
        if (y < curve - wl*0.5/H){
            if (yk < y) fill = max(fill, occ);    // inside the ridge's body
            vis *= 1.0 - occ;
            if (vis < 0.01) break;
        }
    }

    // Vertical floor grid (only on the floor, faded where the lines get too dense).
    if (uGridX > 0.5 && sPix < 1e4 && sPix > 0.0){
        float ps = projScale(sPix, h, p);
        float fx = x/ps;
        if (w > 0.0) fx += w*0.15*sin(sPix*2.0 + gT*1.3);
        float gs = 2.0/uGridX;
        float gd = abs(fract(fx/gs + 0.5) - 0.5)*gs*ps*H;
        float spacingPx = gs*ps*H;
        float gw = max(wPx*0.6*mix(1.0, ps, 0.7*p), 0.5);
        float gl = (1.0 - smoothstep(gw*0.5 - 0.75, gw*0.5 + 0.75, gd)) + exp(-gd/glowR)*uGlow*0.3;
        gl *= smoothstep(1.5, 6.0, spacingPx)*0.8;
        vec3 gc = (pal == 7) ? hsv(fract(fx*0.05 + gT*0.05)) : mix(lineNear, lineFar, 1.0 - exp(-sPix*0.9));
        acc += gc*gl*(1.0 - fill);
    }
    return vec4(acc, fill);
}

void main(){
    vec2 px = gl_FragCoord.xy;
    vec2 res = RENDERSIZE.xy;
    float g = clamp(uGlitch, 0.0, 1.0);
    float w = clamp(uWeird, 0.0, 1.0);
    int pal = int(floor(clamp(uPalette, 0.0, 7.0) + 0.5));

    gTick = floor(TIME*(3.0 + g*20.0));
    gT = TIME;

    // --- Glitch: horizontal tearing + per-band time stutter ---
    if (g > 0.0){
        float bandH = res.y*mix(0.15, 0.02, hash11(gTick*0.37));
        float band = floor(px.y/bandH);
        float hb = hash21(vec2(band, gTick));
        if (hb < g*0.4) px.x += (hash21(vec2(band, gTick + 5.0)) - 0.5)*g*res.x*0.4;
        if (hash21(vec2(band + 13.0, gTick)) < g*0.3) gT = floor(TIME*6.0)/6.0 - hb;
    }

    float x = (px.x - res.x*0.5)/res.y;
    float y = px.y/res.y;
    float h = clamp(uHorizon, 0.05, 0.98);

    // --- Weird: rolling horizon, bent horizon line ---
    if (w > 0.0){
        float r = w*0.35*sin(TIME*0.21);
        vec2 q = vec2(x, y - h);
        q = vec2(cos(r)*q.x - sin(r)*q.y, sin(r)*q.x + cos(r)*q.y);
        x = q.x; y = q.y + h;
        h += w*0.07*sin(x*3.0 + TIME*0.8);
    }
    // --- Glitch: horizon jumps ---
    if (g > 0.0 && hash11(gTick*1.7) < g*0.3) h += (hash11(gTick*2.3) - 0.5)*g*0.15;
    gH = h;

    vec3 lineNear, lineFar, skyTop, skyBot, sunTop, sunBot, floorC;
    palette(pal, lineNear, lineFar, skyTop, skyBot, sunTop, sunBot, floorC);

    // --- Backdrop: sky + striped sun above the horizon, dark floor with horizon haze below ---
    vec3 bg, bgPlain;    // bgPlain: sky / floor without the sun (the background for transparency)
    if (y > h){
        float sy = clamp((y - h)/max(1.0 - h, 0.05), 0.0, 1.0);
        bg = mix(skyBot, skyTop, pow(sy, 0.6));
        bgPlain = bg;
        if (uSun > 0.0){
            vec2 sc = vec2(0.0, h + 0.2);
            float r = 0.24;
            float dsun = length(vec2(x, y) - sc);
            float vy = (y - (sc.y - r))/(2.0*r);
            vec3 sunC = mix(sunBot, sunTop, clamp(vy, 0.0, 1.0));
            // Stripes cut the lower part, widening toward the bottom and scrolling down.
            float stripe = 1.0;
            if (vy < 0.55){
                float f = fract(vy*9.0 + TIME*0.25);
                stripe = step(mix(0.55, 0.0, vy/0.55), f);
            }
            float disk = (1.0 - smoothstep(r - 0.004, r + 0.004, dsun))*stripe;
            bg = mix(bg, sunC, disk*uSun);
            bg += sunBot*exp(-max(dsun - r, 0.0)*9.0)*0.35*uSun;
        }
    } else {
        bg = mix(floorC, skyBot*0.35, exp(-(h - y)*12.0)*clamp(uPerspective, 0.0, 1.0));
        bgPlain = bg;
    }

    // --- Lines ---
    float chroma = (g > 0.0) ? g*0.012*(0.4 + hash11(gTick*0.91)) : 0.0;
    vec4 L = lineLayer(x, y, pal, chroma);
    vec3 col = mix(bg, floorC, L.a) + L.rgb;
    float solid = L.a;    // ridges hiding the lines behind them

    // --- Weird: mirrored ceiling floor in the sky ---
    float ceilAmt = smoothstep(0.45, 0.9, w);
    if (ceilAmt > 0.0 && y > h){
        vec4 C = lineLayer(-x, 2.0*h - y, pal, chroma);
        col = mix(col, floorC, C.a*ceilAmt) + C.rgb*ceilAmt;
        solid = max(solid, C.a*ceilAmt);
    }

    // --- Glitch post: noise blocks, inverted bands, scanline flicker ---
    if (g > 0.0){
        vec2 blk = floor(px/res*vec2(16.0, 9.0));
        float hb = hash21(blk*0.137 + vec2(gTick*0.71, gTick*0.39));
        if (hb < g*0.035){
            vec3 bc = (hash11(hb*91.0) < 0.5) ? lineNear : lineFar;
            col = bc*step(0.5, fract(px.y*0.25 + hb*7.0));
        }
        float inv = hash21(vec2(floor(px.y/(res.y*0.04)), gTick + 2.0));
        if (inv < g*0.08) col = 1.0 - col;
        col *= 1.0 - g*0.3*step(0.5, fract(px.y*0.5));
    }

    // lines, grid, sun and solid ridges stay; the plain sky and floor are the background
    col = clamp(col, 0.0, 1.0);
    bgPlain = clamp(bgPlain, 0.0, 1.0);
    vec3 lit = max(col - bgPlain, 0.0);
    gl_FragColor = withBackground(col, bgPlain, max(solid, max(lit.r, max(lit.g, lit.b))));
}
