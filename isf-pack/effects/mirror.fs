/*{
  "DESCRIPTION": "Mirrors one half of the picture onto the other: left-right, top-bottom or all four quarters. The mirror line can move and turn, so faces and buildings fold into symmetric creatures.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "geometry"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "mode", "LABEL": "Mirror", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["left-right", "top-bottom", "four quarters"] },
    { "NAME": "flip", "LABEL": "Other half", "TYPE": "bool", "DEFAULT": false },
    { "NAME": "center", "LABEL": "Mirror line", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 0.0, "MIN": -0.5, "MAX": 0.5 },
    { "NAME": "spin", "LABEL": "Spin (turns/s)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -2.0, "MAX": 2.0 },
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
    float a = (angle + TIME * spin) * 6.2831853;
    mat2 r = mat2(cos(a), sin(a), -sin(a), cos(a));
    p = r * p;                                   // into the frame of the mirror line
    float sgn = flip ? -1.0 : 1.0;
    if (mode == 0 || mode == 2) p.x = sgn * abs(p.x) * -1.0;
    if (mode == 1 || mode == 2) p.y = sgn * abs(p.y);
    p = transpose(r) * p;
    gl_FragColor = sampleEdge(p / asp + center, edge);
}
