/*{
  "DESCRIPTION": "A mountain range made of horizontal ridge lines, flying towards you, with real hidden-line removal. Styles: ridge lines, wire grid, solid shaded, dots.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "3d", "lines", "retro"],
  "INPUTS": [
    { "NAME": "style", "LABEL": "Style", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3], "LABELS": ["ridge lines", "wire grid", "solid", "dots"] },
    { "NAME": "lineCount", "LABEL": "Lines", "TYPE": "float", "DEFAULT": 40.0, "MIN": 8.0, "MAX": 80.0 },
    { "NAME": "height", "LABEL": "Mountain height", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 3.0 },
    { "NAME": "roughness", "LABEL": "Roughness", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 1.5, "MIN": -6.0, "MAX": 6.0 },
    { "NAME": "valley", "LABEL": "Valley in the middle", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "horizon", "LABEL": "Horizon", "TYPE": "float", "DEFAULT": 0.62, "MIN": 0.3, "MAX": 0.95 },
    { "NAME": "lineWidth", "LABEL": "Line width", "TYPE": "float", "DEFAULT": 1.3, "MIN": 0.5, "MAX": 24.0 },
    { "NAME": "lineColor", "LABEL": "Line color", "TYPE": "color", "DEFAULT": [0.92, 0.95, 1.0, 1.0] },
    { "NAME": "fillColor", "LABEL": "Fill color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "skyColor", "LABEL": "Sky color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
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

float hash(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }

float noise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

// terrain height at world (x, z)
float terrain(vec2 w) {
    float v = 0.0, a = 0.55, fr = 0.35;
    for (int i = 0; i < 4; i++) {
        v += a * noise(w * fr);
        fr *= 2.1;
        a *= mix(0.25, 0.6, roughness);
    }
    float side = smoothstep(0.0, 6.0, abs(w.x));
    return v * height * mix(1.0, side, valley) * 2.2;
}

void main() {
    vec2 p = (gl_FragCoord.xy - vec2(0.5 * RENDERSIZE.x, horizon * RENDERSIZE.y)) / RENDERSIZE.y;
    float pxw = 1.0 / RENDERSIZE.y;
    float t = TIME * speed;
    float camH = 1.6;
    float focal = 0.9;
    vec3 col = skyColor.rgb;
    float spacing = 1.0;
    float zStart = floor(t / spacing) + 1.0;
    float prevY = -10.0;
    int n = int(lineCount);
    bool done = false;
    // front to back: a ridge hides everything below it
    for (int i = 0; i < 80; i++) {
        if (i >= n) break;
        float zw = (zStart + float(i)) * spacing;
        float d = zw - t;
        if (d < 0.35) continue;
        // the highest any ridge can be: above that bound at this depth (and so at all further ones) = sky
        float maxH = height * 2.2 * 0.85;
        if (p.y > max(0.0, (maxH - camH) * focal / d) + lineWidth * pxw) break;
        float wx = p.x * d / focal;
        float h = terrain(vec2(wx, zw));
        float y = (h - camH) * focal / d;
        float fade = 1.0 - smoothstep(0.6, 1.0, float(i) / max(float(n), 1.0));
        float lw = lineWidth * pxw;
        float distY = p.y - y;
        if (distY < lw) {
            // on or below this ridge: the ridge line itself, or its face
            float onLine = 1.0 - smoothstep(0.0, lw, abs(distY));
            vec3 face = fillColor.rgb;
            if (style == 1) {
                // longitudinal lines at whole world x
                float gx = abs(fract(wx + 0.5) - 0.5) / (d / focal * pxw);
                face = mix(face, lineColor.rgb * fade * 0.8, 1.0 - smoothstep(0.0, lineWidth, gx));
            } else if (style == 2) {
                float slope = terrain(vec2(wx + 0.15, zw)) - h;
                float light = clamp(0.55 + slope * 2.5, 0.0, 1.0) * fade;
                face = mix(fillColor.rgb, lineColor.rgb, light * 0.85 * (0.4 + 0.6 * h / max(height * 2.2, 1e-3)));
                onLine *= 0.25;
            } else if (style == 3) {
                onLine = 0.0;
                float gx = fract(wx * 2.0 + 0.5) - 0.5;
                float dd = length(vec2(gx * focal / d * RENDERSIZE.y / 2.0, distY / pxw));
                face = mix(face, lineColor.rgb * fade, 1.0 - smoothstep(lineWidth, lineWidth + 1.0, dd));
            }
            col = mix(face, lineColor.rgb * fade, onLine);
            done = true;
            break;
        }
        prevY = y;
    }
    if (!done) col = skyColor.rgb;
    gl_FragColor = keyed(col, skyColor.rgb);
}
