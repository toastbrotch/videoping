/*{
  "DESCRIPTION": "Folds the brightness of every channel back on itself (a solarizer with many folds), cuts it into steps and spins the hue: from a light tint to psychedelic color chaos.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uFold",   "LABEL": "Fold",   "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uLevels", "LABEL": "Steps",    "TYPE": "float", "DEFAULT": 32.0,"MIN": 2.0, "MAX": 32.0 },
    { "NAME": "uSpin",   "LABEL": "Hue spin","TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/
vec3 hueRotate(vec3 c, float h){
    float a = h*6.2831853; vec3 k = vec3(0.57735); float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    float k = 1.0 + uFold*uFold*12.0;                       // number of folds
    c = 1.0 - abs(fract(c*k*0.5)*2.0 - 1.0);
    float n = floor(uLevels);
    if (n < 31.5) c = floor(c*n + 0.5)/n;
    c = clamp(hueRotate(c, TIME*uSpin), 0.0, 1.0);
    gl_FragColor = vec4(c, IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).a);   // keeps the input's transparency
}
