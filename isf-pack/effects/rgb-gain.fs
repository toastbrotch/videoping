/*{
  "DESCRIPTION": "One gain per color channel: 1 leaves it, 0 removes it, above 1 boosts it, below 0 inverts it.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uRed",   "LABEL": "Red",  "TYPE": "float", "DEFAULT": 1.0, "MIN": -2.0, "MAX": 3.0 },
    { "NAME": "uGreen", "LABEL": "Green","TYPE": "float", "DEFAULT": 1.0, "MIN": -2.0, "MAX": 3.0 },
    { "NAME": "uBlue",  "LABEL": "Blue", "TYPE": "float", "DEFAULT": 1.0, "MIN": -2.0, "MAX": 3.0 }
  ]
}*/
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    vec3 g = vec3(uRed, uGreen, uBlue);
    // Negative gain inverts the channel first, then scales it.
    vec3 v = mix(c*g, (1.0 - c)*(-g), step(g, vec3(0.0)));
    gl_FragColor = vec4(clamp(v, 0.0, 1.0), IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).a);   // keeps the input's transparency
}
