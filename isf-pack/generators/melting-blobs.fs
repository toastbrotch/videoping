/*{
  "DESCRIPTION": "Soft 3D blobs melting into each other and apart again (raymarched metaballs). Five looks: shaded, wireframe, outline, dots, x-ray.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "3d", "organic"],
  "INPUTS": [
    { "NAME": "style", "LABEL": "Style", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["shaded", "wireframe", "outline", "dots", "x-ray"] },
    { "NAME": "count", "LABEL": "Blobs", "TYPE": "float", "DEFAULT": 5.0, "MIN": 2.0, "MAX": 8.0 },
    { "NAME": "blend", "LABEL": "Melt", "TYPE": "float", "DEFAULT": 0.75, "MIN": 0.05, "MAX": 1.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 3.0 },
    { "NAME": "spin", "LABEL": "Rotation", "TYPE": "float", "DEFAULT": 0.15, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.2, "MAX": 2.0 },
    { "NAME": "gloss", "LABEL": "Gloss", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lineWidth", "LABEL": "Line width (px)", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.5, "MAX": 5.0 },
    { "NAME": "wireDensity", "LABEL": "Wire density", "TYPE": "float", "DEFAULT": 6.0, "MIN": 1.0, "MAX": 20.0 },
    { "NAME": "fill", "LABEL": "Fill under lines", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "color1", "LABEL": "Color", "TYPE": "color", "DEFAULT": [0.95, 0.6, 0.9, 1.0] },
    { "NAME": "color2", "LABEL": "Shadow color", "TYPE": "color", "DEFAULT": [0.11, 0.25, 0.55, 1.0] },
    { "NAME": "bgColor", "LABEL": "Background", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
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
// for light on the background: how much brighter than the background a pixel is
vec4 keyed(vec3 col, vec3 bg) {
    vec3 d = max(col - bg, 0.0);
    return withBackground(col, bg, max(d.r, max(d.g, d.b)));
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

mat3 rotY(float a) { float c = cos(a), s = sin(a); return mat3(c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c); }
mat3 rotX(float a) { float c = cos(a), s = sin(a); return mat3(1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c); }

// blob centers and radii: computed once per pixel, not in every march step
vec4 blobs[8];

void placeBlobs() {
    float t = TIME * speed;
    for (int i = 0; i < 8; i++) {
        float fi = float(i);
        vec3 c = spread * vec3(sin(t * (0.61 + 0.13 * fi) + fi * 1.7), cos(t * (0.47 + 0.11 * fi) + fi * 2.3), sin(t * (0.53 + 0.07 * fi) + fi * 0.9)) * 0.9;
        blobs[i] = vec4(c, 0.45 + 0.18 * sin(fi * 3.1 + t * 0.7));
    }
}

float scene(vec3 p) {
    float d = 1e5;
    int n = int(count + 0.5);
    for (int i = 0; i < 8; i++) {
        if (i >= n) break;
        d = smin(d, length(p - blobs[i].xyz) - blobs[i].w, blend * 0.8);
    }
    return d;
}

vec3 normalAt(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(scene(p + e.xyy) - scene(p - e.xyy), scene(p + e.yxy) - scene(p - e.yxy), scene(p + e.yyx) - scene(p - e.yyx)));
}

void main() {
    placeBlobs();
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    mat3 rot = rotY(TIME * spin) * rotX(0.35);
    vec3 ro = rot * vec3(0.0, 0.0, -5.0);
    vec3 rd = rot * normalize(vec3(uv, 1.6));

    // march; for x-ray keep going through the inside and collect the density
    float t = 0.0, minD = 1e5, tMin = 0.0, xray = 0.0;
    bool hit = false;
    for (int i = 0; i < 64; i++) {
        vec3 p = ro + rd * t;
        float d = scene(p);
        if (d < minD) { minD = d; tMin = t; }
        if (style == 4) {
            xray += exp(-abs(d) * 12.0) * 0.06;
            t += max(abs(d) * 0.6, 0.03);
        } else {
            if (d < 0.002 * (1.0 + t * 0.3)) { hit = true; break; }
            t += d;
        }
        if (t > 12.0) break;
    }
    vec3 p = ro + rd * t;
    vec3 n = hit ? normalAt(p) : vec3(0.0);
    vec3 q = (transpose(rot) * p) * wireDensity * 0.5;     // object space: lines stick to the surface

    // shading (used by shaded, as fill, and for the dots)
    vec3 l = normalize(vec3(0.6, 0.8, -0.5));
    float diff = max(dot(n, l), 0.0);
    float spec = pow(max(dot(reflect(rd, n), l), 0.0), 40.0) * gloss;
    float rim = pow(1.0 - max(dot(-rd, n), 0.0), 3.0);
    vec3 shaded = mix(color2.rgb, color1.rgb, diff) + spec + rim * color1.rgb * 0.4;
    float lum = dot(shaded, vec3(0.3, 0.5, 0.2));

    // derivatives for all pixels (outside branches)
    vec3 fq = fwidth(q);
    float fn = length(fwidth(n));
    float ft = fwidth(t) / max(t, 1e-3);
    // the inner side of the silhouette: where the surface turns away from the eye
    float ndv = hit ? max(dot(n, -rd), 0.0) : 1.0;
    float rimLine = hit ? 1.0 - smoothstep(0.0, max(fwidth(ndv), 1e-3) * lineWidth, ndv) : 0.0;
    float px = 1.0 / RENDERSIZE.y;

    vec3 col = bgColor.rgb;
    float fgA = 0.0;      // how much of the pixel is the shape (not background)
    float sil = hit ? 0.0 : 1.0 - smoothstep(0.0, lineWidth * px * tMin / 1.6, minD);
    if (style == 0) {
        if (hit) col = shaded;
        fgA = hit ? 1.0 : 0.0;
    } else if (style == 1) {
        vec3 g = abs(fract(q - 0.5) - 0.5) / max(fq, vec3(1e-4));
        float line = 1.0 - smoothstep(lineWidth * 0.5, lineWidth * 0.5 + 1.0, min(min(g.x, g.y), g.z));
        if (hit) col = mix(bgColor.rgb, shaded, fill) + color1.rgb * line * mix(0.5, 1.0, diff + 0.3);
        col += color1.rgb * sil;
        fgA = hit ? max(fill, line) : sil;
    } else if (style == 2) {
        float edge = hit ? smoothstep(0.06, 0.16, fn) : 0.0;
        edge = max(edge, hit ? smoothstep(0.03, 0.08, ft) : 0.0);
        edge = max(edge, rimLine);
        if (hit) col = mix(bgColor.rgb, shaded, fill);
        col = mix(col, color1.rgb, max(edge, sil));
        fgA = max(max(edge, sil), hit ? fill : 0.0);
    } else if (style == 3) {
        float cell = 4.0 + lineWidth * 3.0;
        vec2 c = mod(gl_FragCoord.xy, cell) - cell * 0.5;
        float r = cell * 0.5 * sqrt(clamp(lum, 0.0, 1.0)) * 1.1;
        float dotm = hit ? 1.0 - smoothstep(r - 0.8, r + 0.8, length(c)) : 0.0;
        col = mix(bgColor.rgb, mix(bgColor.rgb, shaded, fill), hit ? 1.0 : 0.0);
        col = mix(col, color1.rgb, dotm);
        fgA = max(dotm, hit ? fill : 0.0);
    } else {
        col = bgColor.rgb + color1.rgb * xray + color2.rgb * smoothstep(0.08, 0.0, minD) * 0.3;
    }
    gl_FragColor = style == 4 ? keyed(col, bgColor.rgb) : withBackground(col, bgColor.rgb, fgA);
}
