/*{
  "DESCRIPTION": "Video feedback: every frame is laid over the last ones, so movement leaves trails. With zoom, turn and drift the trails become tunnels, spirals and smoke; hue drift colors each echo.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "feedback", "time"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "trailLength", "LABEL": "Trail length", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "mode", "LABEL": "Mode", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["lighten (bright parts trail)", "blend (motion blur)", "add (burns out)"] },
    { "NAME": "zoom", "LABEL": "Zoom per frame", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "turn", "LABEL": "Turn per frame", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "drift", "LABEL": "Drift per frame", "TYPE": "point2D", "DEFAULT": [0.0, 0.0], "MIN": [-1.0, -1.0], "MAX": [1.0, 1.0] },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] },
    { "NAME": "hueDrift", "LABEL": "Hue drift", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "smear", "LABEL": "Smear (blurs the echoes)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "clear", "LABEL": "Clear", "TYPE": "event" }
  ],
  "PASSES": [
    { "TARGET": "trail", "PERSISTENT": true },
    { }
  ]
}*/

vec3 hueRotate(vec3 c, float turns) {
    float a = turns * 6.2831853;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c * ca + cross(k, c) * sin(a) + k * dot(k, c) * (1.0 - ca);
}

void main() {
    vec2 uv = isf_FragNormCoord;
    if (PASSINDEX == 1) { gl_FragColor = IMG_THIS_PIXEL(trail); return; }
    vec4 cur = IMG_THIS_PIXEL(inputImage);
    if (clear || FRAMEINDEX < 2) { gl_FragColor = cur; return; }
    // the last frame, moved: scaled to 60 fps so the speed stays the same at any frame rate
    float fs = clamp(TIMEDELTA * 60.0, 0.1, 4.0);
    vec2 asp = vec2(RENDERSIZE.x / RENDERSIZE.y, 1.0);
    vec2 p = (uv - center) * asp;
    float a = -turn * 0.05 * fs;
    p = mat2(cos(a), sin(a), -sin(a), cos(a)) * p;
    p /= 1.0 + zoom * 0.05 * fs;
    vec2 q = p / asp + center - drift * 0.01 * fs;
    vec4 prev = vec4(0.0);
    if (q.x >= 0.0 && q.y >= 0.0 && q.x <= 1.0 && q.y <= 1.0) {
        prev = texture(trail, q);
        if (smear > 0.0) {
            vec2 o = smear * 3.0 / RENDERSIZE;
            prev = (prev * 2.0 + texture(trail, q + vec2(o.x, 0.0)) + texture(trail, q - vec2(o.x, 0.0))
                    + texture(trail, q + vec2(0.0, o.y)) + texture(trail, q - vec2(0.0, o.y))) / 6.0;
        }
    }
    if (hueDrift != 0.0) prev.rgb = clamp(hueRotate(prev.rgb, hueDrift * 0.02 * fs), 0.0, 1.0);
    // how much of the last frame stays per frame; the 8-bit buffer needs at least one step down,
    // or faint echoes would stay for ever
    float keep = pow(clamp(trailLength, 0.0, 1.0), 0.25);
    keep = pow(keep, fs);
    prev = max(prev * keep - (1.0 / 255.0), 0.0);
    vec4 o;
    if (mode == 0) o = max(cur, prev);
    else if (mode == 1) o = mix(cur, prev, keep * 0.97);
    else o = min(cur + prev, 1.0);
    gl_FragColor = o;
}
