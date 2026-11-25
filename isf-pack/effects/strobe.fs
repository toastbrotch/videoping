/*{
  "DESCRIPTION": "Flashes the picture: to black, to white, inverted, to a color, or freezes it. Free running in flashes per second, or exactly on the beat by animating Phase with the bpm.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "time"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "rate", "LABEL": "Flashes per second (0 = by phase)", "TYPE": "float", "DEFAULT": 2.0, "MIN": 0.0, "MAX": 30.0 },
    { "NAME": "phase", "LABEL": "Phase", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "duty", "LABEL": "Flash length", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "mode", "LABEL": "Flash to", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4, 5], "LABELS": ["black", "white", "inverted", "color", "freeze", "transparent"] },
    { "NAME": "fade", "LABEL": "Fade (soft flash)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "random", "LABEL": "Random", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "flashColor", "LABEL": "Flash color", "TYPE": "color", "DEFAULT": [1.0, 1.0, 1.0, 1.0] }
  ],
  "PASSES": [
    { "TARGET": "held", "PERSISTENT": true },
    { }
  ]
}*/

float hash(float n) { return fract(sin(n * 12.9898) * 43758.5453); }

// 0..1: how much of the flash is on right now
float flashAmount() {
    float t = rate > 0.0 ? TIME * rate + phase : phase;
    float cycle = floor(t);
    float f = fract(t);
    // random: some flashes are skipped
    if (random > 0.0 && hash(cycle + 0.37) < random * 0.8) return 0.0;
    float d = clamp(duty, 0.0, 1.0);
    if (d <= 0.0) return 0.0;
    if (fade <= 0.0) return f < d ? 1.0 : 0.0;
    // soft: rises and falls inside the flash
    float x = f / d;
    if (x > 1.0) return 0.0;
    return pow(sin(x * 3.14159265), mix(0.1, 2.0, fade));
}

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    float on = flashAmount();
    // freeze: the buffer keeps the picture from when the flash started
    if (PASSINDEX == 0) {
        gl_FragColor = (mode == 4 && on > 0.0 && FRAMEINDEX > 1) ? IMG_THIS_PIXEL(held) : s;
        return;
    }
    vec4 f;
    if (mode == 0) f = vec4(0.0, 0.0, 0.0, s.a);
    else if (mode == 1) f = vec4(1.0, 1.0, 1.0, s.a);
    else if (mode == 2) f = vec4(1.0 - s.rgb, s.a);
    else if (mode == 3) f = vec4(flashColor.rgb, s.a);
    else if (mode == 4) f = IMG_THIS_PIXEL(held);
    else f = vec4(s.rgb, 0.0);
    gl_FragColor = mix(s, f, on);
}
