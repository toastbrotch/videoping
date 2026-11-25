/*{
  "DESCRIPTION": "A kaleidoscope: the picture is cut into mirrored wedges around a center and turned. From a calm six-fold mandala to a hundred flickering slivers.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "geometry", "kaleidoscope"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "segments", "LABEL": "Segments", "TYPE": "float", "DEFAULT": 6.0, "MIN": 1.0, "MAX": 64.0 },
    { "NAME": "spin", "LABEL": "Spin (turns/s)", "TYPE": "float", "DEFAULT": 0.03, "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "zoom", "LABEL": "Zoom", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.05, "MAX": 20.0 },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] },
    { "NAME": "source", "LABEL": "Source offset", "TYPE": "point2D", "DEFAULT": [0.0, 0.0], "MIN": [-1.0, -1.0], "MAX": [1.0, 1.0] },
    { "NAME": "rings", "LABEL": "Rings", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 30.0 },
    { "NAME": "edge", "LABEL": "Outside the picture", "TYPE": "long", "DEFAULT": 2, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["stretch", "repeat", "mirror", "black", "transparent"] }
  ]
}*/

// outside the picture: 0 clamp (stretch the border), 1 repeat, 2 mirror, 3 black, 4 transparent
vec4 sampleEdge(vec2 uv, int mode) {
    bool out_ = uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0;
    if (mode == 1) uv = fract(uv);
    else if (mode == 2) uv = 1.0 - abs(1.0 - mod(uv, 2.0));
    else if (mode == 3 && out_) return vec4(0.0, 0.0, 0.0, 1.0);
    else if (mode == 4 && out_) return vec4(0.0);
    return texture(inputImage, clamp(uv, 0.0, 1.0));
}

void main() {
    vec2 asp = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    vec2 p = (isf_FragNormCoord - center) * asp;
    float r = length(p);
    float a = atan(p.y, p.x) + (angle + TIME * spin) * 6.2831853;
    float n = max(floor(segments + 0.5), 1.0);
    float seg = 6.2831853 / n;
    a = mod(a, seg);
    a = abs(a - seg * 0.5);                      // every second wedge mirrored: no seams
    // rings: the distance folds too, so the pattern repeats outwards
    if (rings > 0.0) { float k = 1.0 / rings; r = abs(mod(r + k, 2.0 * k) - k) + 0.0; }
    vec2 q = vec2(cos(a), sin(a)) * r / zoom;
    gl_FragColor = sampleEdge(q / asp + 0.5 + source * 0.5, edge);
}
