/*{
  "DESCRIPTION": "Darkens the edges of the picture like an old lens, from a gentle frame to a small spotlight in the dark. The color can be anything, and the shape round or following the frame.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 0.75, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "roundness", "LABEL": "Round (0 = frame shape)", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] },
    { "NAME": "color", "LABEL": "Color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "edgeAlpha", "LABEL": "Edge (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    vec2 p = (isf_FragNormCoord - center) * 2.0;
    vec2 asp = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    // round: a circle; 0: an ellipse that follows the frame
    p *= mix(vec2(1.0), asp, roundness);
    float d = length(p);
    float v = smoothstep(size, size + softness + 1e-3, d) * amount;
    // straight alpha: the edge color over the picture
    vec4 e = vec4(color.rgb, edgeAlpha);
    float oa = mix(s.a, e.a, v);
    vec3 c = (mix(s.rgb * s.a, e.rgb * e.a, v)) / max(oa, 1e-4);
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), oa);
}
