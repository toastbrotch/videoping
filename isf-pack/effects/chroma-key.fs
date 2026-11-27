/*{
  "DESCRIPTION": "Makes one color transparent, like a green screen: in the mixer (mix type A) the layer below shows there. Spill removal takes the green (or blue) cast off the edges; the edge can shrink or grow; the matte view shows what is kept.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "key", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "keyColor", "LABEL": "Key color", "TYPE": "color", "DEFAULT": [0.1, 0.75, 0.25, 1.0] },
    { "NAME": "tolerance", "LABEL": "Tolerance", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lumaWeight", "LABEL": "Brightness counts", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "spill", "LABEL": "Spill removal", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "edge", "LABEL": "Edge (shrink .. grow, px)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -12.0, "MAX": 12.0 },
    { "NAME": "view", "LABEL": "Show", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["keyed picture", "matte (white = kept)", "only the key color"] }
  ]
}*/

// Distance in YCbCr: mostly the color (Cb, Cr), a little the brightness (Y), so shadows and
// highlights on the screen key out together. Straight alpha out: the kept part keeps its color,
// the matte goes into alpha (times the input's alpha).

vec3 ycc(vec3 c) {
    float y = dot(c, vec3(0.299, 0.587, 0.114));
    return vec3(y, (c.b - y) * 0.565, (c.r - y) * 0.713);
}

// 1 = keep, 0 = key color. The color is compared relative to its brightness (a shadow on the
// screen is the same green, only darker); near black that ratio is unsure, so it fades to the
// plain color difference there.
float matteAt(vec2 uv) {
    vec3 a = ycc(texture(inputImage, uv).rgb);
    vec3 k = ycc(keyColor.rgb);
    float ka = max(k.x, 0.05);
    vec2 an = a.yz * (ka / max(a.x, 0.05));
    vec2 dc = mix(a.yz, an, smoothstep(0.03, 0.15, a.x)) - k.yz;
    float d = sqrt(dot(dc, dc) + (a.x - k.x) * (a.x - k.x) * lumaWeight * lumaWeight);
    // tolerance 0..1 covers the whole color plane; softness widens the transition
    float t = tolerance * 0.6;
    float s = softness * 0.4 + 0.003;
    return smoothstep(t, t + s, d);
}

void main() {
    vec2 uv = isf_FragNormCoord;
    vec4 src = texture(inputImage, uv);
    float m = matteAt(uv);
    // edge: the smallest (shrink) or largest (grow) matte around, at the set distance
    if (abs(edge) > 0.25) {
        vec2 px = abs(edge) / RENDERSIZE;
        float ex = m;
        for (int i = 0; i < 8; i++) {
            float a = float(i) * 0.785398;
            float n = matteAt(uv + vec2(cos(a), sin(a)) * px);
            ex = edge < 0.0 ? min(ex, n) : max(ex, n);
        }
        m = ex;
    }
    // spill: take the key color's tint off what is kept (strongest near the edge)
    vec3 c = src.rgb;
    if (spill > 0.0) {
        vec3 a = ycc(c);
        vec2 kd = ycc(keyColor.rgb).yz;
        float kl = length(kd);
        if (kl > 1e-4) {
            vec2 kn = kd / kl;
            float along = max(dot(a.yz, kn), 0.0);
            a.yz -= kn * along * spill;
            // back to rgb
            float r = a.x + a.z / 0.713;
            float b = a.x + a.y / 0.565;
            float g = (a.x - 0.299 * r - 0.114 * b) / 0.587;
            c = clamp(vec3(r, g, b), 0.0, 1.0);
        }
    }
    if (view == 1) { gl_FragColor = vec4(vec3(m), 1.0); return; }
    if (view == 2) { gl_FragColor = vec4(src.rgb, (1.0 - m) * src.a); return; }
    gl_FragColor = vec4(c, m * src.a);
}
