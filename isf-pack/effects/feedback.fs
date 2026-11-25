/*{
  "DESCRIPTION": "Classic video feedback: the last output is zoomed, turned and shifted and mixed back over the picture. Low feedback gives echoes, high feedback lets the loop take over into tunnels, spirals and streams.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0", "ISFVSN": "2", "CATEGORIES": ["effect", "feedback"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uMix",    "LABEL": "Feedback",   "TYPE": "float", "DEFAULT": 0.7, "MIN": 0.0, "MAX": 1.2 },
    { "NAME": "uZoom",   "LABEL": "Zoom",       "TYPE": "float", "DEFAULT": 0.3, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uRotate", "LABEL": "Turn",    "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uShiftX", "LABEL": "Shift X",  "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uShiftY", "LABEL": "Shift Y",  "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "fbBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/
void main(){
    if (PASSINDEX == 0){
        vec2 asp = vec2(RENDERSIZE.x/RENDERSIZE.y, 1.0);
        vec2 p = (isf_FragNormCoord - 0.5)*asp;
        float a = uRotate*0.15;
        p = vec2(cos(a)*p.x - sin(a)*p.y, sin(a)*p.x + cos(a)*p.y)/(1.0 + uZoom*0.08);
        vec2 uv = fract(p/asp + 0.5 - vec2(uShiftX, uShiftY)*0.03);
        vec3 prev = IMG_NORM_PIXEL(fbBuf, uv).rgb;
        vec3 cur = IMG_NORM_PIXEL(inputImage, isf_FragNormCoord).rgb;
        // A little of the input always gets in, so the loop never starves; above 1 the fed-back
        // picture is amplified, so it builds up until it burns out.
        float m = clamp(uMix, 0.0, 1.0)*0.97;
        vec3 v = mix(cur, prev*max(uMix, 1.0), m);
        gl_FragColor = vec4(clamp(v, 0.0, 4.0), 1.0);
        return;
    }
    gl_FragColor = vec4(clamp(IMG_NORM_PIXEL(fbBuf, isf_FragNormCoord).rgb, 0.0, 1.0), 1.0);
}
