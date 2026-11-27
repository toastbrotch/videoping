/*{
  "DESCRIPTION": "Re-colors the picture's brightness through a palette: thermal, synthwave, sepia, neon, ice, rainbow, toxic, blood. Repeat turns the palette into false-color bands, which can keep flowing.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uPalette", "LABEL": "Palette (0-7)",   "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 7.0 },
    { "NAME": "uRepeat",  "LABEL": "Repeat",    "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 12.0 },
    { "NAME": "uShift",   "LABEL": "Shift",    "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uSpeed",   "LABEL": "Speed",           "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 }
  ]
}*/
vec3 ramp(float t, vec3 a, vec3 b, vec3 c, vec3 d){
    t = clamp(t, 0.0, 1.0)*3.0;
    return t < 1.0 ? mix(a, b, t) : (t < 2.0 ? mix(b, c, t - 1.0) : mix(c, d, t - 2.0));
}
vec3 palette(int p, float t){
    if (p == 0) return ramp(t, vec3(0.0, 0.0, 0.15), vec3(0.6, 0.0, 0.6), vec3(1.0, 0.45, 0.0), vec3(1.0, 1.0, 0.85));
    if (p == 1) return ramp(t, vec3(0.05, 0.0, 0.15), vec3(0.55, 0.0, 0.6), vec3(1.0, 0.2, 0.6), vec3(0.3, 0.95, 1.0));
    if (p == 2) return ramp(t, vec3(0.1, 0.06, 0.03), vec3(0.4, 0.26, 0.14), vec3(0.75, 0.6, 0.4), vec3(1.0, 0.95, 0.85));
    if (p == 3) return ramp(t, vec3(0.0), vec3(0.0, 0.3, 1.0), vec3(1.0, 0.0, 0.8), vec3(0.6, 1.0, 0.2));
    if (p == 4) return ramp(t, vec3(0.0, 0.02, 0.1), vec3(0.05, 0.3, 0.6), vec3(0.5, 0.85, 1.0), vec3(1.0));
    if (p == 5) return clamp(abs(mod(t*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
    if (p == 6) return ramp(t, vec3(0.05, 0.0, 0.1), vec3(0.3, 0.0, 0.4), vec3(0.4, 0.9, 0.0), vec3(0.9, 1.0, 0.5));
    return ramp(t, vec3(0.0), vec3(0.35, 0.0, 0.0), vec3(0.8, 0.05, 0.0), vec3(1.0, 0.85, 0.7));
}
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    float t = l*floor(uRepeat) + uShift + TIME*uSpeed*0.25;
    // Repeats run back and forth (triangle), so bands don't jump from end to start.
    t = uRepeat < 1.5 && uShift == 0.0 && uSpeed == 0.0 ? l : 1.0 - abs(fract(t*0.5)*2.0 - 1.0);
    int p = int(floor(clamp(uPalette, 0.0, 7.0) + 0.5));
    gl_FragColor = vec4(palette(p, t), IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).a);   // keeps the input's transparency
}
