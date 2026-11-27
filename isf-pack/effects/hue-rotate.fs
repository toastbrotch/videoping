/*{
  "DESCRIPTION": "Turns all colors around the color wheel, or keeps them cycling; saturation goes from black and white to wildly over-saturated.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uHue",   "LABEL": "Hue",   "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uSpeed", "LABEL": "Speed",     "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uSat",   "LABEL": "Saturation","TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 4.0 }
  ]
}*/
vec3 hueRotate(vec3 c, float h){
    float a = h*6.2831853; vec3 k = vec3(0.57735); float ca = cos(a);
    return c*ca + cross(k, c)*sin(a) + k*dot(k, c)*(1.0 - ca);
}
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    c = hueRotate(c, uHue + TIME*uSpeed);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(clamp(mix(vec3(l), c, uSat), 0.0, 1.0), IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).a);   // keeps the input's transparency
}
