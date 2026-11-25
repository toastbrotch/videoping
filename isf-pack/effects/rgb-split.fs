/*{
  "DESCRIPTION": "Pulls the red, green and blue parts of the picture apart: a slight lens fringe, a 3D-glasses look, or three ghost copies flying apart. Sideways, zooming out from the center, or turning.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color", "glitch"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "dist", "LABEL": "Distance (px)", "TYPE": "float", "DEFAULT": 6.0, "MIN": 0.0, "MAX": 500.0 },
    { "NAME": "mode", "LABEL": "Mode", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["sideways", "from the center", "turning"] },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "wobble", "LABEL": "Wobble", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "wobbleSpeed", "LABEL": "Wobble speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 20.0 },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] }
  ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    float d = dist;
    // wobble: the distance breathes and jumps a little
    if (wobble > 0.0) {
        float t = TIME * wobbleSpeed;
        d *= 1.0 + wobble * (sin(t * 2.1) * 0.6 + sin(t * 5.3 + 1.0) * 0.4);
    }
    vec2 asp = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    vec2 offR, offB;
    if (mode == 0) {
        float a = angle * 6.2831853;
        offR = vec2(cos(a), sin(a)) * d / RENDERSIZE;
        offB = -offR;
    } else if (mode == 1) {
        // radial: red zooms out, blue in; more towards the edges
        vec2 p = (uv - center) * asp;
        offR = p * (d / RENDERSIZE.y) / asp;
        offB = -offR;
    } else {
        // turning: red and blue turned around the center against each other
        vec2 p = (uv - center) * asp;
        float a = d / RENDERSIZE.y * 0.5 + angle * 0.3;
        vec2 pr = mat2(cos(a), sin(a), -sin(a), cos(a)) * p;
        vec2 pb = mat2(cos(-a), sin(-a), -sin(-a), cos(-a)) * p;
        offR = (pr - p) / asp;
        offB = (pb - p) / asp;
    }
    vec4 r = texture(inputImage, uv + offR);
    vec4 g = texture(inputImage, uv);
    vec4 b = texture(inputImage, uv + offB);
    float a = max(r.a, max(g.a, b.a));
    gl_FragColor = vec4(r.r * r.a, g.g * g.a, b.b * b.a, 1.0) / max(a, 1e-4) * vec4(1.0, 1.0, 1.0, a);
}
