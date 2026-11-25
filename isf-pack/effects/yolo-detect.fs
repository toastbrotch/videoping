/*{
  "DESCRIPTION": "The look of an AI object detector (YOLO style), built on blob detection: every connected region (bright, dark, moving or of one color) gets a class-colored box with a label bar naming its class and a confidence, like PERSON 0.93, plus an optional segmentation mask and outline. No neural network inside.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "hud", "analysis", "overlay"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uSource",   "LABEL": "Source (0 bright, 1 dark, 2 motion, 3 color)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 },
    { "NAME": "uThresh",   "LABEL": "Threshold",          "TYPE": "float", "DEFAULT": 0.55, "MIN": 0.01, "MAX": 1.0 },
    { "NAME": "uHue",      "LABEL": "Target hue",      "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uGrid",     "LABEL": "Grid",            "TYPE": "float", "DEFAULT": 24.0, "MIN": 8.0, "MAX": 36.0 },
    { "NAME": "uMinSize",  "LABEL": "Minimum size",    "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0, "MAX": 40.0 },
    { "NAME": "uSmooth",   "LABEL": "Mask smoothing",  "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uConfMin",  "LABEL": "Confidence threshold","TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0, "MAX": 0.99 },
    { "NAME": "uMask",     "LABEL": "Mask (segmentation)", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uOutline",  "LABEL": "Mask outline",     "TYPE": "float", "DEFAULT": 0.6,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uJitter",   "LABEL": "Detector jitter",  "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uBoxWidth", "LABEL": "Box width",      "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0, "MAX": 6.0 },
    { "NAME": "uLabels",   "LABEL": "Labels",            "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uTextSize", "LABEL": "Text size",       "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0, "MAX": 4.0 },
    { "NAME": "uDarken",   "LABEL": "Darken picture",    "TYPE": "float", "DEFAULT": 0.2,  "MIN": 0.0, "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "maskBuf" },
    { "TARGET": "labBuf",  "FILTER": "NEAREST" },
    { "TARGET": "labBuf",  "FILTER": "NEAREST" },
    { "TARGET": "labBuf",  "FILTER": "NEAREST" },
    { "TARGET": "labBuf",  "FILTER": "NEAREST" },
    { "TARGET": "listBuf", "FILTER": "NEAREST" },
    { "TARGET": "prevBuf", "PERSISTENT": true },
    {}
  ]
}*/

// YOLO DETECT - blob detection with a classifier look.
// Buffers are 8 bit per channel and alpha is not reliable for data, so:
//   labBuf: the cell's blob bounding box (min x, max x, min y, max y; 6 bits each) packed into RGB.
//           Inactive cell: min x 63 > max x 0. The box is the blob's identity.
//   listBuf row 0: packed box; row 1: (valid, fill, -); row 2: (mean brightness, saturation, hue).

#define MAXB 24.0

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

float hueOf(vec3 c){
    vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
    vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
    float d = q.x - min(q.w, q.y);
    return abs(q.z + (q.w - q.y)/(6.0*d + 1e-10));
}

float hash1(float n){ return fract(sin(n*12.9898 + 1.7)*43758.5453); }

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
float number(vec2 p, vec2 o, float s, float v, float ints, float decs){
    vec2 g = (p - o)/s;
    float i = floor(g.x/4.0);
    if (g.y < 0.0 || g.y >= 5.0 || i < 0.0 || i >= ints + decs + (decs > 0.0 ? 1.0 : 0.0)) return 0.0;
    vec2 cell = g - vec2(i*4.0, 0.0);
    v = abs(v);
    if (decs > 0.0 && abs(i - ints) < 0.5) return glyph(8192.0, cell);
    float k = i < ints ? ints - 1.0 - i : -(i - ints);
    float d = mod(floor(v/pow(10.0, k) + 1e-4), 10.0);
    return glyph(digitCode(d), cell);
}

float inRect(vec2 p, vec2 a, vec2 b){ return step(a.x, p.x)*step(p.x, b.x)*step(a.y, p.y)*step(p.y, b.y); }
float rectLine(vec2 p, vec2 a, vec2 b, float w){
    vec2 c = (a + b)*0.5, h = abs(b - a)*0.5;
    vec2 d = abs(p - c) - h;
    float outer = max(d.x, d.y);
    return step(outer, 0.0)*step(-w, outer);
}

// ---------------------------------------------------------------- activity
float activity(vec2 uv){
    vec3 c = IMG_NORM_PIXEL(inputImage, uv).rgb;
    float src = floor(clamp(uSource, 0.0, 3.0) + 0.5);
    float v;
    if (src < 0.5)      v = luma(c) - uThresh;
    else if (src < 1.5) v = (1.0 - luma(c)) - (1.0 - uThresh);
    else if (src < 2.5) v = abs(luma(c) - luma(IMG_NORM_PIXEL(prevBuf, uv).rgb)) - uThresh*0.25;
    else {
        float sat = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
        float dh = abs(fract(hueOf(c) - uHue + 0.5) - 0.5);
        return (1.0 - smoothstep(0.03, 0.09, dh))*smoothstep(0.15, 0.35, sat);
    }
    return smoothstep(-0.03, 0.03, v);
}

// ---------------------------------------------------------------- grid & packing
vec2 gridSize(){
    float n = floor(uGrid);
    return vec2(min(floor(n*RENDERSIZE.x/RENDERSIZE.y + 0.5), 64.0), n);
}
bool inGrid(vec2 c, vec2 g){ return c.x >= 0.0 && c.y >= 0.0 && c.x < g.x && c.y < g.y; }

// box = (min x, min y, max x, max y), each 0..63 -> 24 bits in RGB
vec4 pack(vec4 b){
    float n = ((b.x*64.0 + b.z)*64.0 + b.y)*64.0 + b.w;
    float r = floor(n/65536.0); n -= r*65536.0;
    float g = floor(n/256.0);   n -= g*256.0;
    return vec4(r, g, n, 255.0)/255.0;
}
vec4 unpack(vec4 t){
    vec3 c = floor(t.rgb*255.0 + 0.5);
    float n = c.r*65536.0 + c.g*256.0 + c.b;
    float maxy = mod(n, 64.0); n = floor(n/64.0);
    float miny = mod(n, 64.0); n = floor(n/64.0);
    float maxx = mod(n, 64.0); n = floor(n/64.0);
    return vec4(n, miny, maxx, maxy);
}
const vec4 NONE = vec4(63.0, 0.0, 0.0, 0.0);
vec4 lab(vec2 c){ return unpack(IMG_PIXEL(labBuf, c + 0.5)); }
bool act(vec4 v){ return v.x <= v.z; }
bool same(vec4 a, vec4 b){ return all(lessThan(abs(a - b), vec4(0.5))); }

bool cellActive(vec2 c, vec2 g){
    float a = 0.0;
    for (int j = 0; j < 4; j++)
        for (int i = 0; i < 4; i++)
            a += IMG_NORM_PIXEL(maskBuf, (c + (vec2(float(i), float(j)) + 0.5)/4.0)/g).r;
    return a > 7.0;
}

vec4 cellValue(vec2 c, vec2 g, bool first){
    if (first) return cellActive(c, g) ? vec4(c, c) : NONE;
    return lab(c);
}

vec4 merge(vec4 a, vec4 b){ return vec4(min(a.xy, b.xy), max(a.zw, b.zw)); }

vec4 sweep(vec2 c, vec2 g, vec2 dir, bool first){
    vec4 v = cellValue(c, g, first);
    if (!act(v)) return NONE;
    for (int s = 0; s < 2; s++){
        vec2 d = s == 0 ? dir : -dir;
        for (int k = 1; k < 64; k++){
            vec2 n = c + d*float(k);
            if (!inGrid(n, g)) break;
            vec4 w = cellValue(n, g, first);
            if (!act(w)) break;
            v = merge(v, w);
        }
    }
    return v;
}

// root = first cell of the blob's lowest row carrying this box
bool isRoot(vec2 c, vec4 v){
    if (abs(c.y - v.y) > 0.5) return false;
    for (int i = 0; i < 64; i++){
        float x = v.x + float(i);
        if (x >= c.x - 0.5) break;
        if (same(lab(vec2(x, c.y)), v)) return false;
    }
    return true;
}

void main(){
    vec2 res = RENDERSIZE.xy;
    vec2 px = gl_FragCoord.xy;
    vec2 g = gridSize();
    vec2 cellPx = res/g;

    if (PASSINDEX == 0){
        gl_FragColor = vec4(FRAMEINDEX < 2 && uSource >= 1.5 && uSource < 2.5 ? 0.0 : activity(px/res), 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX >= 1 && PASSINDEX <= 4){
        if (!inGrid(floor(px), g)){ gl_FragColor = pack(NONE); return; }
        vec2 dir = (PASSINDEX == 1 || PASSINDEX == 3) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
        gl_FragColor = pack(sweep(floor(px), g, dir, PASSINDEX == 1));
        return;
    }

    if (PASSINDEX == 5){
        if (px.y >= 3.0 || px.x >= MAXB){ gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0); return; }
        float k = floor(px.x);
        float n = 0.0;
        vec4 found = NONE;
        for (int j = 0; j < 36; j++){
            for (int i = 0; i < 64; i++){
                vec2 c = vec2(float(i), float(j));
                if (!inGrid(c, g) || act(found)) continue;
                vec4 v = lab(c);
                if (!act(v) || !isRoot(c, v)) continue;
                float boxArea = (v.z - v.x + 1.0)*(v.w - v.y + 1.0);
                if (boxArea < floor(uMinSize)) continue;
                if (abs(n - k) < 0.5) found = v;
                n += 1.0;
            }
        }
        if (!act(found)){ gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0); return; }
        if (px.y < 1.0){
            gl_FragColor = pack(found);
            return;
        }
        float area = 0.0, sl = 0.0, ss = 0.0; vec2 hv = vec2(0.0);
        for (int j = 0; j < 36; j++)
            for (int i = 0; i < 64; i++){
                vec2 c = vec2(float(i), float(j));
                if (!inGrid(c, g)) continue;
                if (same(lab(c), found)){
                    area += 1.0;
                    if (px.y >= 2.0){
                        vec3 cc = IMG_NORM_PIXEL(inputImage, (c + 0.5)/g).rgb;
                        float sat = max(cc.r, max(cc.g, cc.b)) - min(cc.r, min(cc.g, cc.b));
                        sl += luma(cc); ss += sat;
                        float a = hueOf(cc)*6.2831853;
                        hv += sat*vec2(cos(a), sin(a));
                    }
                }
            }
        if (px.y < 2.0){
            float boxArea = (found.z - found.x + 1.0)*(found.w - found.y + 1.0);
            gl_FragColor = vec4(1.0, area/max(boxArea, 1.0), 0.0, 1.0);
        } else {
            float hue = fract(atan(hv.y, hv.x)/6.2831853 + 1.0);
            gl_FragColor = vec4(sl/max(area, 1.0), ss/max(area, 1.0), hue, 1.0);
        }
        return;
    }

    if (PASSINDEX == 6){
        gl_FragColor = vec4(IMG_PIXEL(inputImage, px).rgb, 1.0);
        return;
    }

    // ---------------------------------------------------------------- overlay
    vec2 uv = px/res;
    vec3 img = IMG_NORM_PIXEL(inputImage, uv).rgb;
    vec3 col = img*(1.0 - uDarken);
    float lw = max(1.0, floor(res.y/540.0 + 0.5));
    float s = floor(uTextSize*max(1.0, res.y/540.0));

    float r = 1.0 + uSmooth*10.0;
    vec2 d = r/res;
    float mC = IMG_NORM_PIXEL(maskBuf, uv).r;
    float mL = IMG_NORM_PIXEL(maskBuf, uv - vec2(d.x, 0.0)).r, mR = IMG_NORM_PIXEL(maskBuf, uv + vec2(d.x, 0.0)).r;
    float mD = IMG_NORM_PIXEL(maskBuf, uv - vec2(0.0, d.y)).r, mU = IMG_NORM_PIXEL(maskBuf, uv + vec2(0.0, d.y)).r;
    float m = (mC*2.0 + mL + mR + mD + mU)/6.0;
    vec2 grad = vec2(mR - mL, mU - mD)/(2.0*r);
    float dist = abs(m - 0.5)/max(length(grad), 1e-3);

    vec2 c0 = floor(px/cellPx);
    vec4 myBox = lab(c0);

    float ink = 0.0; vec3 inkCol = vec3(1.0);
    float bar = 0.0; vec3 barCol = vec3(1.0);
    float txt = 0.0;
    vec3 maskCol = vec3(-1.0);
    float frameJ = floor(float(FRAMEINDEX)/2.0);

    for (int k = 0; k < 24; k++){
        vec4 info = IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 1.5));
        if (info.r < 0.5) continue;
        vec4 bx = unpack(IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 0.5)));
        vec4 st = vec4(IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 2.5)).rgb, info.g);
        float id = bx.y*64.0 + bx.x + bx.z*0.37 + bx.w*0.11;

        float w = bx.z - bx.x + 1.0, h = bx.w - bx.y + 1.0;
        float aspect = (h*cellPx.y)/(w*cellPx.x);
        float cls = 7.0;
        float fit = 0.5;
        if (st.x > 0.75 && w*h < 20.0)        { cls = 2.0; fit = st.x; }
        else if (aspect > 1.6)                { cls = 0.0; fit = min(aspect/2.5, 1.0); }
        else if (aspect < 0.55 && w*h > 6.0)  { cls = 1.0; fit = min(0.55/aspect*0.6, 1.0); }
        else if (st.y > 0.45){
            float hh = st.z;
            cls = (hh < 0.1 || hh > 0.9) ? 3.0 : (hh < 0.45 ? 4.0 : 5.0);
            fit = st.y;
        }
        else if (st.w < 0.45)                 { cls = 6.0; fit = 1.0 - st.w; }
        float conf = clamp(0.45 + 0.5*fit + (hash1(id + 3.1) - 0.5)*0.12
                         + (hash1(id + frameJ*0.137) - 0.5)*0.06*uJitter, 0.0, 0.99);
        if (conf < uConfMin) continue;

        vec3 cc = cls < 0.5 ? vec3(1.0, 0.22, 0.32) : cls < 1.5 ? vec3(0.2, 0.55, 1.0) : cls < 2.5 ? vec3(1.0, 0.85, 0.1)
                : cls < 3.5 ? vec3(1.0, 0.45, 0.0) : cls < 4.5 ? vec3(0.2, 0.9, 0.3) : cls < 5.5 ? vec3(0.0, 0.9, 0.95)
                : cls < 6.5 ? vec3(0.75, 0.4, 1.0) : vec3(0.95, 0.95, 0.95);
        if (same(myBox, bx)) maskCol = cc;

        vec2 jt = (vec2(hash1(id + frameJ*1.3), hash1(id*1.7 + frameJ)) - 0.5)*uJitter*cellPx*0.3;
        vec2 a = bx.xy*cellPx - 2.0*lw + jt, b = (bx.zw + 1.0)*cellPx + 2.0*lw + jt.yx;
        float bw = floor(uBoxWidth)*lw;
        float e = rectLine(px, a, b, bw);
        if (e > ink){ ink = e; inkCol = cc; }

        if (uLabels > 0.5){
            vec2 to = vec2(a.x, b.y + 2.0*s);
            float len = cls < 0.5 ? 6.0 : cls < 1.5 ? 7.0 : cls < 2.5 ? 5.0 : cls < 3.5 ? 4.0 : cls < 4.5 ? 5.0
                      : cls < 5.5 ? 6.0 : cls < 6.5 ? 9.0 : 6.0;
            vec2 ba = vec2(a.x - bw*0.5, b.y), bb = vec2(a.x + (len + 5.5)*4.0*s + 2.0*s, b.y + 9.0*s);
            if (inRect(px, ba, bb) > 0.5){
                bar = 1.0; barCol = cc;
                vec2 q = (px - to)/s;
                float t;
                if (cls < 0.5)      t = glyph(4843.0, q) + glyph(29391.0, q - vec2(4.0, 0.0)) + glyph(23275.0, q - vec2(8.0, 0.0)) + glyph(14478.0, q - vec2(12.0, 0.0)) + glyph(11114.0, q - vec2(16.0, 0.0)) + glyph(23403.0, q - vec2(20.0, 0.0));
                else if (cls < 1.5) t = glyph(11117.0, q) + glyph(29391.0, q - vec2(4.0, 0.0)) + glyph(23533.0, q - vec2(8.0, 0.0)) + glyph(29847.0, q - vec2(12.0, 0.0)) + glyph(25166.0, q - vec2(16.0, 0.0)) + glyph(29257.0, q - vec2(20.0, 0.0)) + glyph(29391.0, q - vec2(24.0, 0.0));
                else if (cls < 2.5) t = glyph(29257.0, q) + glyph(29847.0, q - vec2(4.0, 0.0)) + glyph(27470.0, q - vec2(8.0, 0.0)) + glyph(23533.0, q - vec2(12.0, 0.0)) + glyph(9367.0, q - vec2(16.0, 0.0));
                else if (cls < 3.5) t = glyph(14478.0, q) + glyph(29847.0, q - vec2(4.0, 0.0)) + glyph(27470.0, q - vec2(8.0, 0.0)) + glyph(23403.0, q - vec2(12.0, 0.0));
                else if (cls < 4.5) t = glyph(4843.0, q) + glyph(29257.0, q - vec2(4.0, 0.0)) + glyph(23530.0, q - vec2(8.0, 0.0)) + glyph(23403.0, q - vec2(12.0, 0.0)) + glyph(9367.0, q - vec2(16.0, 0.0));
                else if (cls < 5.5) t = glyph(14478.0, q) + glyph(25166.0, q - vec2(4.0, 0.0)) + glyph(23275.0, q - vec2(8.0, 0.0)) + glyph(29391.0, q - vec2(12.0, 0.0)) + glyph(29391.0, q - vec2(16.0, 0.0)) + glyph(23403.0, q - vec2(20.0, 0.0));
                else if (cls < 6.5) t = glyph(14478.0, q) + glyph(9367.0, q - vec2(4.0, 0.0)) + glyph(23275.0, q - vec2(8.0, 0.0)) + glyph(31597.0, q - vec2(12.0, 0.0)) + glyph(25166.0, q - vec2(16.0, 0.0)) + glyph(9367.0, q - vec2(20.0, 0.0)) + glyph(31597.0, q - vec2(24.0, 0.0)) + glyph(23275.0, q - vec2(28.0, 0.0)) + glyph(29391.0, q - vec2(32.0, 0.0));
                else                t = glyph(11114.0, q) + glyph(15083.0, q - vec2(4.0, 0.0)) + glyph(11044.0, q - vec2(8.0, 0.0)) + glyph(29391.0, q - vec2(12.0, 0.0)) + glyph(25166.0, q - vec2(16.0, 0.0)) + glyph(9367.0, q - vec2(20.0, 0.0));
                t += number(px, to + vec2((len + 1.0)*4.0*s, 0.0), s, conf, 1.0, 2.0);
                txt = max(txt, t);
            }
        }
    }

    if (maskCol.x >= 0.0){
        if (m > 0.5) col = mix(col, maskCol, uMask*0.6);
        col = mix(col, maskCol, (1.0 - smoothstep(lw, lw + 1.0, dist))*uOutline);
    }
    col = mix(col, inkCol, clamp(ink, 0.0, 1.0));
    if (bar > 0.5) col = mix(barCol, vec3(0.05), clamp(txt, 0.0, 1.0));
    gl_FragColor = vec4(col, 1.0);
}
