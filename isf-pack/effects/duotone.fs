/*{
  "DESCRIPTION": "Shadows in one hue, highlights in another, with a contrast push from a soft print look to a hard two-color poster.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uHueDark",  "LABEL": "Dark hue", "TYPE": "float", "DEFAULT": 0.7,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uHueLight", "LABEL": "Light hue",   "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uContrast", "LABEL": "Contrast",       "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.5, "MAX": 8.0 }
  ]
}*/
vec3 hsv(float h, float s, float v){
    return v*mix(vec3(1.0), clamp(abs(mod(h*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0), s);
}
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    float l = clamp((dot(c, vec3(0.299, 0.587, 0.114)) - 0.5)*uContrast + 0.5, 0.0, 1.0);
    vec3 dark = hsv(uHueDark, 0.9, 0.35), light = hsv(uHueLight, 0.6, 1.0);
    gl_FragColor = vec4(mix(dark*l*2.0, mix(dark, light, l*2.0 - 1.0), step(0.5, l)), 1.0);
}
