/*{
  "DESCRIPTION": "A lit, turning 3D shape drawn with halftone dots like a print. The style switch turns it into shaded, wireframe, outline or x-ray.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "3d", "print"],
  "INPUTS": [
    { "NAME": "style", "LABEL": "Style", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["halftone", "shaded", "wireframe", "outline", "x-ray"] },
    { "NAME": "shape", "LABEL": "Shape", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3], "LABELS": ["sphere", "torus", "cube", "octahedron"] },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.3, "MAX": 1.8 },
    { "NAME": "spin", "LABEL": "Rotation", "TYPE": "float", "DEFAULT": 0.3, "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "lightAngle", "LABEL": "Light angle", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lightMove", "LABEL": "Light moves", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "dotSize", "LABEL": "Dot size (px)", "TYPE": "float", "DEFAULT": 9.0, "MIN": 3.0, "MAX": 120.0 },
    { "NAME": "screenAngle", "LABEL": "Screen angle", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lineWidth", "LABEL": "Line width (px)", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.5, "MAX": 24.0 },
    { "NAME": "wireDensity", "LABEL": "Wire density", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 24.0 },
    { "NAME": "fill", "LABEL": "Fill under lines", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "paper", "LABEL": "Paper (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "inkColor", "LABEL": "Ink", "TYPE": "color", "DEFAULT": [0.08, 0.08, 0.1, 1.0] },
    { "NAME": "shadowColor", "LABEL": "Shadow color", "TYPE": "color", "DEFAULT": [0.78, 0.39, 0.04, 1.0] },
    { "NAME": "paperColor", "LABEL": "Paper", "TYPE": "color", "DEFAULT": [0.94, 0.91, 0.84, 1.0] }
  ]
}*/

// "a over b" with straight (not premultiplied) alpha
vec4 over(vec4 b, vec4 a) {
    float oa = a.a + b.a * (1.0 - a.a);
    vec3 c = (a.rgb * a.a + b.rgb * b.a * (1.0 - a.a)) / max(oa, 1e-4);
    return vec4(c, oa);
}

mat3 rotY(float a) { float c = cos(a), s = sin(a); return mat3(c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c); }
mat3 rotX(float a) { float c = cos(a), s = sin(a); return mat3(1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c); }

mat3 objRot() { return rotY(TIME * spin) * rotX(TIME * spin * 0.37 + 0.4); }

float scene(vec3 p) {
    p = transpose(objRot()) * p / size;
    float d;
    if (shape == 1) { vec2 q = vec2(length(p.xz) - 1.0, p.y); d = length(q) - 0.4; }
    else if (shape == 2) { vec3 q = abs(p) - vec3(0.8); d = length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - 0.05; }
    else if (shape == 3) { d = (abs(p.x) + abs(p.y) + abs(p.z) - 1.3) * 0.57735; }
    else d = length(p) - 1.2;
    return d * size;
}

vec3 normalAt(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(scene(p + e.xyy) - scene(p - e.xyy), scene(p + e.yxy) - scene(p - e.yxy), scene(p + e.yyx) - scene(p - e.yyx)));
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    vec3 ro = vec3(0.0, 0.0, -5.0);
    vec3 rd = normalize(vec3(uv, 1.7));
    float t = 0.0, minD = 1e5, tMin = 0.0, xray = 0.0;
    bool hit = false;
    for (int i = 0; i < 80; i++) {
        vec3 p = ro + rd * t;
        float d = scene(p);
        if (d < minD) { minD = d; tMin = t; }
        if (style == 4) {
            xray += exp(-abs(d) * 14.0) * 0.06;
            t += max(abs(d) * 0.6, 0.03);
        } else {
            if (d < 0.001) { hit = true; break; }
            t += d;
        }
        if (t > 10.0) break;
    }
    vec3 p = ro + rd * t;
    vec3 n = hit ? normalAt(p) : vec3(0.0);
    vec3 q = (transpose(objRot()) * p) * wireDensity * 0.5;

    float la = lightAngle * 6.2831 + TIME * lightMove;
    vec3 l = normalize(vec3(cos(la), 0.7, sin(la) - 0.6));
    float diff = max(dot(n, l), 0.0);
    float spec = pow(max(dot(reflect(rd, n), l), 0.0), 30.0);
    // halftone: dark where the light does not reach
    float shade = clamp(diff * 0.9 + spec * 0.5, 0.0, 1.0);

    vec3 fq = fwidth(q);
    float fn = length(fwidth(n));
    float ft = fwidth(t) / max(t, 1e-3);
    // the inner side of the silhouette: where the surface turns away from the eye
    float ndv = hit ? max(dot(n, -rd), 0.0) : 1.0;
    float rimLine = hit ? 1.0 - smoothstep(0.0, max(fwidth(ndv), 1e-3) * lineWidth, ndv) : 0.0;
    float px = 1.0 / RENDERSIZE.y;
    float sil = hit ? 0.0 : 1.0 - smoothstep(0.0, lineWidth * px * tMin / 1.7, minD);

    // layers, bottom to top: paper (opacity "paper"), the surface (shaded: always; line styles: "fill"),
    // the shadow screen, the ink. Kept with straight alpha, so with paper < 1 only the print stays and
    // the layer below shows through (mixer: A).
    vec3 shaded = mix(shadowColor.rgb, paperColor.rgb, shade);
    vec4 o = vec4(paperColor.rgb, paper);
    vec3 surf = shaded;
    float surfA = 0.0, shadowA = 0.0, inkA = 0.0;
    if (style == 0) {
        float a = screenAngle * 1.5708;
        vec2 g = mat2(cos(a), sin(a), -sin(a), cos(a)) * gl_FragCoord.xy / dotSize;
        vec2 c = fract(g) - 0.5;
        float r = sqrt(1.0 - shade) * 0.62;
        float aa = 1.2 / dotSize;
        inkA = hit ? 1.0 - smoothstep(r - aa, r + aa, length(c)) : 0.0;
        // a second, finer screen in the shadow color gives the print its depth
        vec2 c2 = fract(g * 1.5 + 0.25) - 0.5;
        float r2 = sqrt(clamp(0.6 - shade, 0.0, 1.0)) * 0.55;
        shadowA = (hit ? 1.0 - smoothstep(r2 - aa * 1.5, r2 + aa * 1.5, length(c2)) : 0.0) * 0.7;
    } else if (style == 1) {
        surf = mix(inkColor.rgb, shaded, 0.15 + 0.85 * shade) + spec * 0.4;
        surfA = hit ? 1.0 : 0.0;
    } else if (style == 2) {
        vec3 g = abs(fract(q - 0.5) - 0.5) / max(fq, vec3(1e-4));
        float line = 1.0 - smoothstep(lineWidth * 0.5, lineWidth * 0.5 + 1.0, min(min(g.x, g.y), g.z));
        surfA = hit ? fill : 0.0;
        inkA = max(hit ? line * mix(0.45, 1.0, shade) : 0.0, sil);
    } else if (style == 3) {
        float edge = hit ? smoothstep(0.06, 0.16, fn) : 0.0;
        edge = max(edge, hit ? smoothstep(0.03, 0.08, ft) : 0.0);
        edge = max(edge, rimLine);
        surfA = hit ? fill : 0.0;
        inkA = max(edge, sil);
    } else {
        inkA = clamp(xray, 0.0, 1.0);
    }
    o = over(o, vec4(surf, surfA));
    o = over(o, vec4(shadowColor.rgb, shadowA));
    o = over(o, vec4(inkColor.rgb, inkA));
    gl_FragColor = o;
}
