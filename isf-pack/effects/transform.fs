/*{
  "DESCRIPTION": "Zoom, turn, stretch and move the picture, with a continuous spin and drift; outside the picture it stretches the border, tiles, mirrors or turns transparent.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "geometry"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "zoom", "LABEL": "Zoom", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.02, "MAX": 30.0 },
    { "NAME": "stretch", "LABEL": "Stretch (wide .. tall)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "angle", "LABEL": "Angle", "TYPE": "float", "DEFAULT": 0.0, "MIN": -0.5, "MAX": 0.5 },
    { "NAME": "spin", "LABEL": "Spin (turns/s)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -4.0, "MAX": 4.0 },
    { "NAME": "position", "LABEL": "Position", "TYPE": "point2D", "DEFAULT": [0.0, 0.0], "MIN": [-1.0, -1.0], "MAX": [1.0, 1.0] },
    { "NAME": "drift", "LABEL": "Drift (screens/s)", "TYPE": "point2D", "DEFAULT": [0.0, 0.0], "MIN": [-2.0, -2.0], "MAX": [2.0, 2.0] },
    { "NAME": "edge", "LABEL": "Outside the picture", "TYPE": "long", "DEFAULT": 3, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["stretch", "repeat", "mirror", "black", "transparent"] }
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
    vec2 p = (isf_FragNormCoord - 0.5 - position * 0.5) * asp;
    float a = -(angle + TIME * spin) * 6.2831853;
    p = mat2(cos(a), sin(a), -sin(a), cos(a)) * p;
    p /= zoom * vec2(exp2(-stretch * 2.0), exp2(stretch * 2.0));
    vec2 uv = p / asp + 0.5 - drift * TIME;
    gl_FragColor = sampleEdge(uv, edge);
}
