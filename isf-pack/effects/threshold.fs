/*{
  "DESCRIPTION": "Splits the picture into two colors at a brightness threshold: hard black and white, a soft two-tone print, or per color channel for a pop-art look.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 0.03, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "mode", "LABEL": "Mode", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1], "LABELS": ["brightness", "per channel"] },
    { "NAME": "noise", "LABEL": "Noise", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "darkColor", "LABEL": "Dark color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "lightColor", "LABEL": "Light color", "TYPE": "color", "DEFAULT": [1.0, 1.0, 1.0, 1.0] }
  ]
}*/

float hash(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    float w = softness * 0.5 + 0.002;
    // noise: a moving grain on the threshold, so the edge crawls like a photocopy
    float t = threshold + (hash(gl_FragCoord.xy + floor(TIME * 24.0) * 17.0) - 0.5) * noise;
    vec3 c;
    if (mode == 0) {
        float l = dot(s.rgb, vec3(0.2126, 0.7152, 0.0722));
        c = mix(darkColor.rgb, lightColor.rgb, smoothstep(t - w, t + w, l));
    } else {
        c = mix(darkColor.rgb, lightColor.rgb, smoothstep(vec3(t - w), vec3(t + w), s.rgb));
    }
    gl_FragColor = vec4(c, s.a);
}
