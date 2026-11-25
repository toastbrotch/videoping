/*{
  "DESCRIPTION": "Reduces the picture to a few color steps, like a screen print or an old game. Dither breaks the steps into a fine pattern of dots.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color", "retro"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "levels", "LABEL": "Steps", "TYPE": "float", "DEFAULT": 5.0, "MIN": 2.0, "MAX": 32.0 },
    { "NAME": "mode", "LABEL": "Mode", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1], "LABELS": ["color", "brightness (keeps the hue)"] },
    { "NAME": "dither", "LABEL": "Dither", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "ditherSize", "LABEL": "Dither size (px)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 32.0 },
    { "NAME": "gamma", "LABEL": "Step spread", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.2, "MAX": 5.0 }
  ]
}*/

// 4x4 ordered dither (Bayer), -0.5 .. 0.5
float bayer(vec2 p) {
    p = mod(floor(p), 4.0);
    float i = p.x + p.y * 4.0;
    // the 4x4 Bayer matrix, row by row
    float m[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return m[int(i)] / 16.0 - 0.47;
}

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    float n = floor(levels + 0.5) - 1.0;
    float d = dither * bayer(gl_FragCoord.xy / ditherSize) / n;
    vec3 c = pow(max(s.rgb, 0.0), vec3(gamma));
    if (mode == 0) {
        c = floor(clamp(c + d, 0.0, 1.0) * n + 0.5) / n;
    } else {
        float l = max(max(c.r, c.g), c.b);
        float q = floor(clamp(l + d, 0.0, 1.0) * n + 0.5) / n;
        c = l > 1e-4 ? c * (q / l) : vec3(q);
    }
    c = pow(c, vec3(1.0 / gamma));
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), s.a);
}
