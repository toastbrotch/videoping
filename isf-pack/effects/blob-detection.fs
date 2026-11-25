/*{
  "DESCRIPTION": "Finds connected regions in the picture (bright, dark, moving or of one color) and outlines them like a machine-vision system: smooth contour lines along each blob's real edge, a color per blob, bounding boxes with IDs and sizes, centroid crosses and optional links between the blobs.",
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
    { "NAME": "uSmooth",   "LABEL": "Contour smoothing",  "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uOutline",  "LABEL": "Contours",          "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uFill",     "LABEL": "Fill",          "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uBoxes",    "LABEL": "Boxes",            "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uLinks",    "LABEL": "Links",      "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uColors",   "LABEL": "Color per blob",    "TYPE": "float", "DEFAULT": 0.7,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uLabels",   "LABEL": "Labels",            "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uTextSize", "LABEL": "Text size",       "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0, "MAX": 4.0 },
    { "NAME": "uDarken",   "LABEL": "Darken picture",    "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uHudHue",   "LABEL": "HUD color",         "TYPE": "float", "DEFAULT": 0.33, "MIN": 0.0, "MAX": 1.0 }
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

// BLOB DETECTION - eight passes.
// Buffers are 8 bit per channel (0..1) and alpha is not used for data, so:
//   labBuf: each cell's blob bounding box (min x, min y, max x, max y; 6 bits each) packed into RGB.
//           Inactive: min x 63 > max x 0. The box is the blob's identity.
//   listBuf: pixel (k,0) = packed box; (k,1) = (area hi byte, area lo byte, valid);
//            (k,2) = centroid / grid size.

#define MAXB 24.0

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

float segment(vec2 p, vec2 a, vec2 b, float w){
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba)/max(dot(ba, ba), 1e-4), 0.0, 1.0);
    return 1.0 - smoothstep(w*0.5, w*0.5 + 1.0, length(pa - ba*h));
}
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
float boxId(vec4 b){ return ((b.x*64.0 + b.z)*64.0 + b.y)*64.0 + b.w; }

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
        if (px.y < 1.0){ gl_FragColor = pack(found); return; }
        float area = 0.0; vec2 cen = vec2(0.0);
        for (int j = 0; j < 36; j++)
            for (int i = 0; i < 64; i++){
                vec2 c = vec2(float(i), float(j));
                if (!inGrid(c, g)) continue;
                if (same(lab(c), found)){ area += 1.0; cen += c + 0.5; }
            }
        if (px.y < 2.0){
            float hi = floor(area/256.0);
            gl_FragColor = vec4(hi/255.0, (area - hi*256.0)/255.0, 1.0, 1.0);
        } else {
            gl_FragColor = vec4(cen/max(area, 1.0)/g, 0.0, 1.0);
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
    vec3 hud = hsv(uHudHue, 0.75, 1.0);
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

    // Blob colour of this cell, or of the nearest active neighbour on the contour.
    vec2 c0 = floor(px/cellPx);
    vec4 cb = lab(c0);
    if (!act(cb)){
        for (int k = 0; k < 4; k++){
            vec2 nc = c0 + (k == 0 ? vec2(1.0, 0.0) : k == 1 ? vec2(-1.0, 0.0) : k == 2 ? vec2(0.0, 1.0) : vec2(0.0, -1.0));
            if (inGrid(nc, g) && !act(cb)) cb = lab(nc);
        }
    }
    vec3 bc = act(cb) ? mix(hud, hsv(hash1(boxId(cb)*0.001), 0.8, 1.0), uColors) : hud;

    if (m > 0.5) col = mix(col, bc*0.7, uFill);
    col = mix(col, bc, (1.0 - smoothstep(lw, lw + 1.0, dist))*uOutline);

    float ink = 0.0;
    vec3 inkCol = hud;
    float count = 0.0;
    vec2 lastC = vec2(-1.0);
    for (int k = 0; k < 24; k++){
        vec4 info = IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 1.5));
        if (info.b < 0.5) continue;
        vec4 bx = unpack(IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 0.5)));
        float area = floor(info.r*255.0 + 0.5)*256.0 + floor(info.g*255.0 + 0.5);
        vec2 cenN = IMG_PIXEL(listBuf, vec2(float(k) + 0.5, 2.5)).rg;
        count += 1.0;
        vec3 kc = mix(hud, hsv(hash1(boxId(bx)*0.001), 0.8, 1.0), uColors);
        vec2 a = bx.xy*cellPx - 3.0*lw, b = (bx.zw + 1.0)*cellPx + 3.0*lw;
        vec2 cen = cenN*res;
        float e = 0.0;
        if (uBoxes > 0.0){
            vec2 dd = min(px - a, b - px);
            float legs = min(b.x - a.x, b.y - a.y)*0.25;
            e += rectLine(px, a, b, lw)*(1.0 - step(legs, min(dd.x, dd.y)))*uBoxes
               + rectLine(px, a, b, lw)*0.35*uBoxes;
            vec2 q = abs(px - cen);
            e += (step(q.x, lw*0.5)*step(q.y, 6.0*lw) + step(q.y, lw*0.5)*step(q.x, 6.0*lw))*uBoxes;
        }
        if (uLinks > 0.0 && lastC.x >= 0.0) e += segment(px, lastC, cen, lw)*uLinks*0.7;
        lastC = cen;
        if (uLabels > 0.5){
            vec2 to = vec2(a.x, b.y + 2.0*lw);
            e += glyph(15083.0, (px - to)/s) + number(px, to + vec2(4.0*s, 0.0), s, float(k) + 1.0, 2.0, 0.0);
            e += number(px, to + vec2(14.0*s, 0.0), s, area, 3.0, 0.0)*0.7;
        }
        if (e > ink){ ink = e; inkCol = kc; }
    }

    // Telemetry.
    if (uLabels > 0.5){
        vec2 o = vec2(10.0*lw, res.y - 10.0*lw - 5.0*s);
        float t = (glyph(15083.0, (px - (o))/s - vec2(0.0, 0.0)) + glyph(29257.0, (px - (o))/s - vec2(4.0, 0.0)) + glyph(11114.0, (px - (o))/s - vec2(8.0, 0.0)) + glyph(15083.0, (px - (o))/s - vec2(12.0, 0.0)) + glyph(15211.0, (px - (o))/s - vec2(20.0, 0.0)) + glyph(29391.0, (px - (o))/s - vec2(24.0, 0.0)) + glyph(9367.0, (px - (o))/s - vec2(28.0, 0.0)) + glyph(29391.0, (px - (o))/s - vec2(32.0, 0.0)) + glyph(25166.0, (px - (o))/s - vec2(36.0, 0.0)) + glyph(9367.0, (px - (o))/s - vec2(40.0, 0.0)))
                + (glyph(25166.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(0.0, 0.0)) + glyph(11114.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(4.0, 0.0)) + glyph(31597.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(8.0, 0.0)) + glyph(23403.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(12.0, 0.0)) + glyph(9367.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(16.0, 0.0))) + number(px, o + vec2(24.0*s, -8.0*s), s, count, 2.0, 0.0)
                + (glyph(4815.0, (px - (o - vec2(0.0, 16.0*s)))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (o - vec2(0.0, 16.0*s)))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (o - vec2(0.0, 16.0*s)))/s - vec2(8.0, 0.0))) + number(px, o + vec2(16.0*s, -16.0*s), s, mod(float(FRAMEINDEX), 1e6), 6.0, 0.0);
        if (t > ink){ ink = t; inkCol = hud; }
    }

    col = mix(col, inkCol, clamp(ink, 0.0, 1.0));
    gl_FragColor = vec4(col, 1.0);
}
