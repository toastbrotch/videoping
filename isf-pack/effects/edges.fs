/*{
  "DESCRIPTION": "Finds the edges in the picture: glowing lines on black like a neon drawing, lines laid over the picture, or dark ink outlines for a comic look.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "stylize"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "style", "LABEL": "Style", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["lines on black", "lines over the picture", "ink outlines"] },
    { "NAME": "strength", "LABEL": "Strength", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 40.0 },
    { "NAME": "width", "LABEL": "Width (px)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.5, "MAX": 30.0 },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.08, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "colored", "LABEL": "Picture colors", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "edgeColor", "LABEL": "Line color", "TYPE": "color", "DEFAULT": [1.0, 1.0, 1.0, 1.0] }
  ]
}*/

float lum(vec2 uv) { return dot(texture(inputImage, uv).rgb, vec3(0.2126, 0.7152, 0.0722)); }

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 px = width / RENDERSIZE;
    vec4 s = texture(inputImage, uv);
    // Sobel on the brightness
    float tl = lum(uv + px * vec2(-1, 1)), t = lum(uv + px * vec2(0, 1)), tr = lum(uv + px * vec2(1, 1));
    float l = lum(uv + px * vec2(-1, 0)), r = lum(uv + px * vec2(1, 0));
    float bl = lum(uv + px * vec2(-1, -1)), b = lum(uv + px * vec2(0, -1)), br = lum(uv + px * vec2(1, -1));
    float gx = (tr + 2.0 * r + br) - (tl + 2.0 * l + bl);
    float gy = (tl + 2.0 * t + tr) - (bl + 2.0 * b + br);
    float e = length(vec2(gx, gy));
    e = clamp((e - threshold) * strength, 0.0, 1.0);
    // the line color: the set color, or the picture's own color (brightened, so dark parts still show)
    vec3 own = s.rgb / max(max(s.r, max(s.g, s.b)), 0.25);
    vec3 lc = mix(edgeColor.rgb, edgeColor.rgb * own, colored);
    vec3 c;
    if (style == 0) c = lc * e;
    else if (style == 1) c = mix(s.rgb, lc, e);
    else c = s.rgb * (1.0 - e);
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), s.a);
}
