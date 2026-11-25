/*{
  "DESCRIPTION": "Turns the picture into big pixels: squares, round dots like an LED wall, or bars. From a slight retro grain to a few giant blocks.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "retro", "geometry"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "size", "LABEL": "Pixel size (px)", "TYPE": "float", "DEFAULT": 12.0, "MIN": 1.0, "MAX": 600.0 },
    { "NAME": "shape", "LABEL": "Shape", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["squares", "round dots", "diamonds", "horizontal bars", "vertical bars"] },
    { "NAME": "gap", "LABEL": "Gap", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 0.95 },
    { "NAME": "sizeByBrightness", "LABEL": "Size by brightness", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "gapColor", "LABEL": "Gap color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "gapAlpha", "LABEL": "Gap (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

void main() {
    vec2 frag = gl_FragCoord.xy;
    vec2 cs = vec2(size);
    if (shape == 3) cs = vec2(RENDERSIZE.x, size);
    if (shape == 4) cs = vec2(size, RENDERSIZE.y);
    // cells are laid out from the center of the picture, so zooming the size stays centered
    vec2 rel = frag - RENDERSIZE * 0.5;
    vec2 cell = floor(rel / cs);
    vec2 cc = (cell + 0.5) * cs + RENDERSIZE * 0.5;
    vec4 c = texture(inputImage, clamp(cc / RENDERSIZE, 0.0, 1.0));
    vec2 f = (rel - cell * cs) / cs - 0.5;      // -0.5 .. 0.5 inside the cell
    float l = dot(c.rgb, vec3(0.2126, 0.7152, 0.0722));
    float k = (1.0 - gap) * mix(1.0, sqrt(l), sizeByBrightness);
    float d;
    if (shape == 1) d = length(f) * 2.0;
    else if (shape == 2) d = abs(f.x) + abs(f.y);
    else if (shape == 3) d = abs(f.y) * 2.0;
    else if (shape == 4) d = abs(f.x) * 2.0;
    else d = max(abs(f.x), abs(f.y)) * 2.0;
    float aa = 1.5 / min(cs.x, cs.y);
    float inside = 1.0 - smoothstep(k - aa, k + aa, d);
    if (size < 1.5) inside = 1.0;
    // straight alpha: the cell over the gap
    vec4 g = vec4(gapColor.rgb, gapAlpha * c.a);
    float a = inside * c.a + g.a * (1.0 - inside * c.a);
    vec3 col = (c.rgb * inside * c.a + g.rgb * g.a * (1.0 - inside * c.a)) / max(a, 1e-4);
    gl_FragColor = vec4(col, a);
}
