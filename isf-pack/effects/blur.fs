/*{
  "DESCRIPTION": "A smooth gaussian blur, from a soft focus to a picture dissolved into color fields; or a directional blur that smears everything along one angle.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "blur"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "radius", "LABEL": "Radius (px)", "TYPE": "float", "DEFAULT": 6.0, "MIN": 0.0, "MAX": 400.0 },
    { "NAME": "shape", "LABEL": "Shape", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1], "LABELS": ["round", "directional"] },
    { "NAME": "angle", "LABEL": "Direction", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "glow", "LABEL": "Glow (blur added on top)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 4.0 }
  ],
  "PASSES": [
    { "TARGET": "bufA" },
    { "TARGET": "bufB" },
    { "TARGET": "bufA" },
    { }
  ]
}*/

// Four separable passes: horizontal, vertical, horizontal, vertical (round), or four times along
// one direction. Two rounds of a gaussian hide the gaps between the taps at big radii.
// The buffers hold premultiplied color, so transparent parts don't bleed dark edges.

const int N = 12;

vec4 tapsAlong(sampler2D img, vec2 dir, float r, bool premultiply) {
    vec2 px = 1.0 / RENDERSIZE;
    vec2 uv = isf_FragNormCoord;
    float sigma = max(r, 1e-3) / 2.0;
    float stepPx = r / float(N);
    vec4 acc = vec4(0.0);
    float wsum = 0.0;
    for (int i = -N; i <= N; i++) {
        float x = float(i) * stepPx;
        float w = exp(-0.5 * x * x / (sigma * sigma));
        vec4 c = texture(img, uv + dir * x * px);
        if (premultiply) c.rgb *= c.a;
        acc += c * w;
        wsum += w;
    }
    return acc / wsum;
}

void main() {
    // each of the two rounds blurs with radius / sqrt(2): together the set radius
    float r = radius * 0.7071;
    float a = angle * 3.14159265;
    vec2 d1 = shape == 0 ? vec2(1.0, 0.0) : vec2(cos(a), sin(a));
    vec2 d2 = shape == 0 ? vec2(0.0, 1.0) : d1;
    if (PASSINDEX == 0) { gl_FragColor = tapsAlong(inputImage, d1, r, true); return; }
    if (PASSINDEX == 1) { gl_FragColor = tapsAlong(bufA, d2, r, false); return; }
    if (PASSINDEX == 2) { gl_FragColor = tapsAlong(bufB, d1, r * 0.5, false); return; }
    vec4 b = tapsAlong(bufA, d2, r * 0.5, false);
    vec3 col = b.a > 1e-4 ? b.rgb / b.a : vec3(0.0);
    // glow: the blur added to the sharp picture
    if (glow > 0.0) {
        vec4 s = IMG_THIS_PIXEL(inputImage);
        float oa = max(s.a, b.a);
        col = (s.rgb * s.a + b.rgb * glow) / max(oa, 1e-4);
        gl_FragColor = vec4(clamp(col, 0.0, 1.0), oa);
        return;
    }
    gl_FragColor = vec4(clamp(col, 0.0, 1.0), b.a);
}
