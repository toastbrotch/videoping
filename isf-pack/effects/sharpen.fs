/*{
  "DESCRIPTION": "Sharpens the picture (unsharp mask): a little crisps up a soft clip, a lot gives hard halos and a crunchy, over-processed look.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "blur"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.0, "MAX": 20.0 },
    { "NAME": "radius", "LABEL": "Radius (px)", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.5, "MAX": 40.0 },
    { "NAME": "threshold", "LABEL": "Leave flat areas", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 0.5 }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 px = radius / RENDERSIZE;
    vec4 s = texture(inputImage, uv);
    // a small blur from 8 neighbors on a circle plus 8 halfway: the "unsharp" picture
    vec3 blur = vec3(0.0);
    for (int i = 0; i < 8; i++) {
        float a = float(i) * 0.785398 + 0.39;
        vec2 o = vec2(cos(a), sin(a));
        blur += texture(inputImage, uv + o * px).rgb * 0.6 + texture(inputImage, uv + o * px * 0.5).rgb * 0.4;
    }
    blur /= 8.0;
    vec3 detail = s.rgb - blur;
    // leave flat areas: small differences (noise, gradients) are not sharpened
    float m = max(abs(detail.r), max(abs(detail.g), abs(detail.b)));
    detail *= smoothstep(threshold, threshold * 1.5 + 1e-4, m);
    gl_FragColor = vec4(clamp(s.rgb + detail * amount, 0.0, 1.0), s.a);
}
