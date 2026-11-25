/*{
  "DESCRIPTION": "Keeps one color and drains the rest to gray (the 'Sin City' look), or with Rest saturation above 1 cranks the rest into over-saturation instead.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uHue",   "LABEL": "Hue",      "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uWidth", "LABEL": "Range",      "TYPE": "float", "DEFAULT": 0.08, "MIN": 0.01, "MAX": 0.5 },
    { "NAME": "uRest",  "LABEL": "Rest saturation", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 }
  ]
}*/
float hueOf(vec3 c){
    vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
    vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);
    float d = q.x - min(q.w, q.y);
    return abs(q.z + (q.w - q.y)/(6.0*d + 1e-10));
}
void main(){
    vec3 c = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
    float sat = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
    float dh = abs(fract(hueOf(c) - uHue + 0.5) - 0.5);
    float keep = (1.0 - smoothstep(uWidth*0.6, uWidth, dh))*smoothstep(0.05, 0.2, sat);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    vec3 rest = clamp(mix(vec3(l), c, uRest), 0.0, 1.0);
    gl_FragColor = vec4(mix(rest, c, keep), 1.0);
}
