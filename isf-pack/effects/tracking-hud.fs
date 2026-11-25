/*{
  "DESCRIPTION": "A machine-vision overlay that finds and tracks things in the picture: detector cells look for motion, brightness or one color and frame every active cell as a numbered box; a main tracker follows the center of all activity with crosshair, LOCK / SEARCH status and coordinates.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "hud", "analysis", "overlay"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uSource",   "LABEL": "Source (0 motion, 1 bright, 2 color)", "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "uThresh",   "LABEL": "Threshold",            "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.01, "MAX": 1.0 },
    { "NAME": "uHue",      "LABEL": "Target hue",        "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uCells",    "LABEL": "Grid",              "TYPE": "float", "DEFAULT": 10.0, "MIN": 3.0, "MAX": 24.0 },
    { "NAME": "uObjects",  "LABEL": "Object sensitivity", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uSmooth",   "LABEL": "Tracker inertia",   "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uMask",     "LABEL": "Show mask",        "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uDarken",   "LABEL": "Darken picture",      "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uHudHue",   "LABEL": "HUD color",           "TYPE": "float", "DEFAULT": 0.33, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uText",     "LABEL": "Text",                "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uTextSize", "LABEL": "Text size",         "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0, "MAX": 5.0 }
  ],
  "PASSES": [
    { "TARGET": "statBuf", "PERSISTENT": true, "FLOAT": true, "FILTER": "NEAREST" },
    { "TARGET": "maskBuf", "FLOAT": true },
    { "TARGET": "cellBuf", "FLOAT": true, "FILTER": "NEAREST" },
    { "TARGET": "prevBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// TRACKING HUD
//   pass 0 statBuf (pixel 0,0): weighted mean position and spread of the activity over a 64x36
//          sample grid, smoothed over time -> the main tracker (x, y, spread x, spread y);
//          pixel 1,0: (active share, lock 0/1).
//   pass 1 maskBuf: activity at every pixel (computed here, while prevBuf still holds the last
//          frame - by the overlay pass it already holds this one).
//   pass 2 cellBuf: pixel (i, j) of the detector grid = share of active samples in that cell.
//   pass 3 prevBuf: this frame, for next frame's motion.
//   pass 4: the overlay. Text uses a built-in 3x5 pixel font (glyph()).
// "Activity" per sample: motion = brightness change since last frame, brightness, or closeness
// to the target hue - compared against the threshold.

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

float hueOf(vec3 c){
    vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
    vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
    float d = q.x - min(q.w, q.y);
    return abs(q.z + (q.w - q.y)/(6.0*d + 1e-10));
}

vec3 hsv(float h, float s, float v){
    return v*mix(vec3(1.0), clamp(abs(mod(h*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0), s);
}

// How active a spot is, 0..1 (soft around the threshold).
float activity(vec2 uv){
    vec3 c = IMG_NORM_PIXEL(inputImage, uv).rgb;
    float src = floor(clamp(uSource, 0.0, 2.0) + 0.5);
    float v;
    if (src < 0.5){
        v = abs(luma(c) - luma(IMG_NORM_PIXEL(prevBuf, uv).rgb));
    } else if (src < 1.5){
        v = luma(c);
    } else {
        float sat = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
        float dh = abs(fract(hueOf(c) - uHue + 0.5) - 0.5);
        v = (1.0 - smoothstep(0.03, 0.09, dh))*smoothstep(0.15, 0.35, sat);
        return v;
    }
    return smoothstep(uThresh*0.8, uThresh*1.2, v);
}

// ---------------------------------------------------------------- 3x5 font
float glyph(float code, vec2 g){
    if (g.x < 0.0 || g.y < 0.0 || g.x >= 3.0 || g.y >= 5.0) return 0.0;
    float idx = (4.0 - floor(g.y))*3.0 + floor(g.x);
    return mod(floor(code/exp2(idx)), 2.0);
}
float digitCode(float d){
    if (d < 0.5) return 31599.0; if (d < 1.5) return 29850.0; if (d < 2.5) return 29671.0;
    if (d < 3.5) return 31207.0; if (d < 4.5) return 18925.0; if (d < 5.5) return 31183.0;
    if (d < 6.5) return 31695.0; if (d < 7.5) return 9383.0;  if (d < 8.5) return 31727.0;
    return 31215.0;
}
// Number with `ints` integer digits and `decs` decimals (a dot in between), drawn at o.
float number(vec2 p, vec2 o, float s, float v, float ints, float decs){
    vec2 g = (p - o)/s;
    float i = floor(g.x/4.0);
    if (g.y < 0.0 || g.y >= 5.0 || i < 0.0 || i >= ints + decs + (decs > 0.0 ? 1.0 : 0.0)) return 0.0;
    vec2 cell = g - vec2(i*4.0, 0.0);
    v = abs(v);
    if (decs > 0.0 && abs(i - ints) < 0.5) return glyph(8192.0, cell);    // '.'
    float k = i < ints ? ints - 1.0 - i : -(i - ints);
    float d = mod(floor(v/pow(10.0, k) + 1e-4), 10.0);
    return glyph(digitCode(d), cell);
}

// ---------------------------------------------------------------- drawing helpers
float rectLine(vec2 p, vec2 a, vec2 b, float w){
    vec2 c = (a + b)*0.5, h = abs(b - a)*0.5;
    vec2 d = abs(p - c) - h;
    float outer = max(d.x, d.y);
    return step(outer, 0.0)*step(-w, outer);
}
// Only the four corners of a box (brackets), each leg `len` long.
float brackets(vec2 p, vec2 a, vec2 b, float w, float len){
    float r = rectLine(p, a, b, w);
    vec2 lo = min(a, b), hi = max(a, b);
    vec2 near = min(p - lo, hi - p);
    return r*step(min(near.x, near.y), len)*max(step(near.x, len), step(near.y, len))
         *(1.0 - step(len, near.x)*step(len, near.y));
}

void main(){
    vec2 res = RENDERSIZE.xy;
    vec2 px = gl_FragCoord.xy;

    if (PASSINDEX == 0){
        if (px.y >= 1.0 || px.x >= 2.0){ gl_FragColor = vec4(0.0); return; }
        float sw = 0.0; vec2 sp = vec2(0.0), sp2 = vec2(0.0);
        for (int j = 0; j < 36; j++)
            for (int i = 0; i < 64; i++){
                vec2 uv = (vec2(float(i), float(j)) + 0.5)/vec2(64.0, 36.0);
                float w = activity(uv);
                sw += w; sp += w*uv; sp2 += w*uv*uv;
            }
        vec4 last0 = IMG_PIXEL(statBuf, vec2(0.5, 0.5));
        vec4 last1 = IMG_PIXEL(statBuf, vec2(1.5, 0.5));
        float share = sw/2304.0;
        bool found = sw > 4.0;
        vec2 mean = found ? sp/sw : last0.xy;
        vec2 spread = found ? sqrt(max(sp2/sw - mean*mean, vec2(0.0))) : last0.zw;
        float tau = mix(0.02, 1.5, uSmooth*uSmooth);
        float k = FRAMEINDEX < 2 ? 1.0 : 1.0 - exp(-max(TIMEDELTA, 0.0)/tau);
        if (px.x < 1.0)
            gl_FragColor = mix(last0, vec4(mean, spread), k);
        else
            gl_FragColor = vec4(mix(last1.x, share, k), found ? 1.0 : 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 1){
        gl_FragColor = vec4(activity(px/res), 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 2){
        float n = floor(uCells);
        vec2 grid = vec2(floor(n*res.x/res.y + 0.5), n);
        if (px.x >= grid.x || px.y >= grid.y){ gl_FragColor = vec4(0.0); return; }
        float sum = 0.0;
        for (int j = 0; j < 6; j++)
            for (int i = 0; i < 6; i++){
                vec2 uv = (floor(px) + (vec2(float(i), float(j)) + 0.5)/6.0)/grid;
                sum += IMG_NORM_PIXEL(maskBuf, uv).r;
            }
        gl_FragColor = vec4(sum/36.0, 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 3){
        gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(inputImage).rgb, 1.0);
        return;
    }

    // ---------------------------------------------------------------- overlay
    vec2 uv = px/res;
    vec3 img = IMG_NORM_PIXEL(inputImage, uv).rgb;
    vec3 hud = hsv(uHudHue, 0.75, 1.0);
    vec3 col = img*(1.0 - uDarken);                 // 1 = picture fully black, HUD only
    float lw = max(1.0, floor(res.y/540.0 + 0.5));            // line width in pixels
    float s = floor(uTextSize*max(1.0, res.y/720.0));        // font pixel size
    float ink = 0.0, dim = 0.0;

    // Detection mask: where the detector sees activity.
    float act = IMG_NORM_PIXEL(maskBuf, uv).r;
    col = mix(col, hud*0.8, act*uMask*0.7);

    // Detector cells: every active cell framed as a numbered detection box.
    float n = floor(uCells);
    vec2 grid = vec2(floor(n*res.x/res.y + 0.5), n);
    vec2 cid = floor(uv*grid);
    float e = IMG_PIXEL(cellBuf, cid + 0.5).r;
    float need = mix(0.6, 0.03, uObjects);
    if (e > need){
        vec2 a = cid/grid*res + 2.0*lw, b = (cid + 1.0)/grid*res - 2.0*lw;
        dim += rectLine(px, a, b, lw)*0.6;
        ink += brackets(px, a, b, lw*2.0, min(b.x - a.x, b.y - a.y)*0.25);
        if (uText > 0.5){
            float id = cid.y*grid.x + cid.x;
            vec2 to = vec2(a.x + 2.0*lw, b.y - 7.0*s);
            ink += number(px, to, s, id, 3.0, 0.0);
            ink += number(px, to - vec2(0.0, 7.0*s), s, e*100.0, 2.0, 0.0)*0.8;
        }
    }

    // Main tracker.
    vec4 st = IMG_PIXEL(statBuf, vec2(0.5, 0.5));
    vec4 st1 = IMG_PIXEL(statBuf, vec2(1.5, 0.5));
    bool lock = st1.y > 0.5;
    vec2 c = st.xy*res;
    vec2 half_ = clamp(st.zw*res*2.0, vec2(24.0*lw), res*0.45);
    float blink = step(0.5, fract(TIME*2.0));
    float on = lock ? 1.0 : blink;
    // Crosshair: dashed lines across the frame through the target, solid near it.
    float dash = step(0.5, fract((px.x + px.y)/(12.0*lw)));
    float hl = step(abs(px.y - c.y), lw*0.5)*(abs(px.x - c.x) < half_.x ? 0.0 : dash);
    float vl = step(abs(px.x - c.x), lw*0.5)*(abs(px.y - c.y) < half_.y ? 0.0 : dash);
    dim += (hl + vl)*0.5*on;
    ink += brackets(px, c - half_, c + half_, lw*2.0, min(half_.x, half_.y)*0.5)*on;
    ink += step(length(px - c), 3.0*lw)*on;
    if (uText > 0.5){
        // Labels sit above and below the box, but always stay inside the frame.
        vec2 lo = vec2(4.0*s), hi = res - vec2(80.0*s, 8.0*s);
        vec2 to = clamp(c + vec2(-half_.x, half_.y + 3.0*s), lo, hi);
        if (lock) ink += (glyph(29257.0, (px - (to))/s - vec2(0.0, 0.0)) + glyph(11114.0, (px - (to))/s - vec2(4.0, 0.0)) + glyph(25166.0, (px - (to))/s - vec2(8.0, 0.0)) + glyph(23277.0, (px - (to))/s - vec2(12.0, 0.0)));
        else      ink += (glyph(14478.0, (px - (to))/s - vec2(0.0, 0.0)) + glyph(29391.0, (px - (to))/s - vec2(4.0, 0.0)) + glyph(23530.0, (px - (to))/s - vec2(8.0, 0.0)) + glyph(23275.0, (px - (to))/s - vec2(12.0, 0.0)) + glyph(25166.0, (px - (to))/s - vec2(16.0, 0.0)) + glyph(23533.0, (px - (to))/s - vec2(20.0, 0.0)))*blink;
        vec2 t2 = clamp(c + vec2(-half_.x, -half_.y - 8.0*s), lo, hi);
        ink += (glyph(23213.0, (px - (t2))/s - vec2(0.0, 0.0))) + number(px, t2 + vec2(8.0*s, 0.0), s, st.x, 1.0, 3.0);
        ink += (glyph(9389.0, (px - (t2 + vec2(32.0*s, 0.0)))/s - vec2(0.0, 0.0))) + number(px, t2 + vec2(40.0*s, 0.0), s, st.y, 1.0, 3.0);

        // Frame telemetry.
        vec2 tl = vec2(6.0*s, res.y - 11.0*s);
        ink += (glyph(9367.0, (px - (tl))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (tl))/s - vec2(4.0, 0.0)) + glyph(23530.0, (px - (tl))/s - vec2(8.0, 0.0)) + glyph(25166.0, (px - (tl))/s - vec2(12.0, 0.0)) + glyph(23277.0, (px - (tl))/s - vec2(16.0, 0.0)) + glyph(29847.0, (px - (tl))/s - vec2(20.0, 0.0)) + glyph(23403.0, (px - (tl))/s - vec2(24.0, 0.0)) + glyph(27470.0, (px - (tl))/s - vec2(28.0, 0.0)));
        dim += (glyph(4815.0, (px - (tl - vec2(0.0, 8.0*s)))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (tl - vec2(0.0, 8.0*s)))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (tl - vec2(0.0, 8.0*s)))/s - vec2(8.0, 0.0))) + number(px, tl + vec2(16.0*s, -8.0*s), s, mod(float(FRAMEINDEX), 1e6), 6.0, 0.0);
        dim += (glyph(23530.0, (px - (tl - vec2(0.0, 16.0*s)))/s - vec2(0.0, 0.0)) + glyph(25166.0, (px - (tl - vec2(0.0, 16.0*s)))/s - vec2(4.0, 0.0)) + glyph(9367.0, (px - (tl - vec2(0.0, 16.0*s)))/s - vec2(8.0, 0.0))) + number(px, tl + vec2(16.0*s, -16.0*s), s, st1.x*100.0, 2.0, 1.0)
             + (glyph(21157.0, (px - (tl + vec2(32.0*s, -16.0*s)))/s - vec2(0.0, 0.0)));
        float src = floor(clamp(uSource, 0.0, 2.0) + 0.5);
        vec2 sl = tl - vec2(0.0, 24.0*s);
        if (src < 0.5)      dim += (glyph(14478.0, (px - (sl))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (sl))/s - vec2(4.0, 0.0)) + glyph(25166.0, (px - (sl))/s - vec2(8.0, 0.0)) + glyph(23549.0, (px - (sl))/s - vec2(16.0, 0.0)) + glyph(11114.0, (px - (sl))/s - vec2(20.0, 0.0)) + glyph(9367.0, (px - (sl))/s - vec2(24.0, 0.0)) + glyph(29847.0, (px - (sl))/s - vec2(28.0, 0.0)) + glyph(11114.0, (px - (sl))/s - vec2(32.0, 0.0)) + glyph(23403.0, (px - (sl))/s - vec2(36.0, 0.0)));
        else if (src < 1.5) dim += (glyph(14478.0, (px - (sl))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (sl))/s - vec2(4.0, 0.0)) + glyph(25166.0, (px - (sl))/s - vec2(8.0, 0.0)) + glyph(29257.0, (px - (sl))/s - vec2(16.0, 0.0)) + glyph(31597.0, (px - (sl))/s - vec2(20.0, 0.0)) + glyph(23549.0, (px - (sl))/s - vec2(24.0, 0.0)) + glyph(23530.0, (px - (sl))/s - vec2(28.0, 0.0)));
        else                dim += (glyph(14478.0, (px - (sl))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (sl))/s - vec2(4.0, 0.0)) + glyph(25166.0, (px - (sl))/s - vec2(8.0, 0.0)) + glyph(23533.0, (px - (sl))/s - vec2(16.0, 0.0)) + glyph(31597.0, (px - (sl))/s - vec2(20.0, 0.0)) + glyph(29391.0, (px - (sl))/s - vec2(24.0, 0.0))) + number(px, sl + vec2(32.0*s, 0.0), s, uHue*360.0, 3.0, 0.0);
    }

    // Frame corner brackets.
    ink += brackets(px, vec2(4.0*lw), res - 4.0*lw, lw*2.0, res.y*0.08);

    col = mix(col, hud*0.7, clamp(dim, 0.0, 1.0));
    col = mix(col, hud, clamp(ink, 0.0, 1.0));
    gl_FragColor = vec4(col, 1.0);
}
