/*{
  "DESCRIPTION": "Glowing oscilloscope curves with afterglow. The phase drifts, so the figure turns slowly in space; the trail buffer keeps the phosphor glow.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "lines", "retro", "feedback"],
  "INPUTS": [
    { "NAME": "freqX", "LABEL": "Frequency X", "TYPE": "float", "DEFAULT": 3.0, "MIN": 1.0, "MAX": 9.0 },
    { "NAME": "freqY", "LABEL": "Frequency Y", "TYPE": "float", "DEFAULT": 2.0, "MIN": 1.0, "MAX": 9.0 },
    { "NAME": "freqZ", "LABEL": "Frequency Z (depth)", "TYPE": "float", "DEFAULT": 5.0, "MIN": 0.0, "MAX": 9.0 },
    { "NAME": "drift", "LABEL": "Phase drift", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "spin", "LABEL": "Spin", "TYPE": "float", "DEFAULT": 0.2, "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.2, "MAX": 1.2 },
    { "NAME": "beam", "LABEL": "Beam width", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 6.0 },
    { "NAME": "glow", "LABEL": "Glow", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "afterglow", "LABEL": "Afterglow", "TYPE": "float", "DEFAULT": 0.85, "MIN": 0.0, "MAX": 0.98 },
    { "NAME": "beamColor", "LABEL": "Beam color", "TYPE": "color", "DEFAULT": [0.45, 1.0, 0.6, 1.0] },
    { "NAME": "bgColor", "LABEL": "Background", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "graticule", "LABEL": "Graticule", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "bgAlpha", "LABEL": "Background (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "trail", "PERSISTENT": true },
    { }
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

const int SEGMENTS = 128;

vec2 curve(float s, float ph, float rot) {
    // whole numbers: the curve closes
    float fx = floor(freqX + 0.5), fy = floor(freqY + 0.5), fz = floor(freqZ + 0.5);
    vec3 q = vec3(sin(fx * s + ph), sin(fy * s), sin(fz * s + ph * 0.7));
    // turn around the vertical axis: depth shows as the figure turning
    float c = cos(rot), si = sin(rot);
    vec2 r = vec2(q.x * c + q.z * si * (freqZ > 0.0 ? 1.0 : 0.0), q.y);
    return r * size * 0.5;
}

float segDist(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-8), 0.0, 1.0);
    return length(pa - ba * h);
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    if (PASSINDEX == 0) {
        float ph = TIME * drift;
        float rot = TIME * spin;
        float d = 1e9;
        // far outside the figure only the halo matters: one cheap estimate instead of all segments
        float reach = size * 0.5 * 1.42;
        // as many segments as the figure needs (more loops = more segments)
        float segs = floor(clamp(14.0 * (floor(freqX + 0.5) + floor(freqY + 0.5) + floor(freqZ + 0.5) * 0.5), 24.0, float(SEGMENTS)));
        vec2 prev = curve(0.0, ph, rot);
        if (length(p) > reach + 0.15) d = length(p) - reach;
        else for (int i = 1; i <= SEGMENTS; i++) {
            if (float(i) > segs) break;
            vec2 cur = curve(6.2831853 * float(i) / segs, ph, rot);
            d = min(d, segDist(p, prev, cur));
            prev = cur;
        }
        float px = d * RENDERSIZE.y;
        float core = 1.0 - smoothstep(beam * 0.5, beam * 0.5 + 1.0, px);
        float halo = glow * 0.5 * exp(-px / (beam * 4.0));
        float old = IMG_NORM_PIXEL(trail, isf_FragNormCoord).r;
        float v = max(old * afterglow, core + halo * 0.6);
        gl_FragColor = vec4(v, v, v, 1.0);
        return;
    }
    float v = IMG_NORM_PIXEL(trail, isf_FragNormCoord).r;
    // graticule: the screen grid of an oscilloscope
    vec2 g = abs(fract(p * 8.0 + 0.5) - 0.5) / (8.0 / RENDERSIZE.y);
    float grid = (1.0 - smoothstep(0.0, 1.0, min(g.x, g.y))) * graticule * 0.35;
    vec3 col = bgColor.rgb + beamColor.rgb * grid;
    col += beamColor.rgb * v + vec3(1.0) * pow(v, 4.0) * 0.6;
    gl_FragColor = keyed(col, bgColor.rgb);
}
