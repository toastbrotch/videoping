/*{
  "DESCRIPTION": "Inverts the picture: the colors as a negative, only the brightness (colors stay), only the hue, or single channels. Amount fades through gray in between, which is a look of its own.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "mode", "LABEL": "Mode", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4, 5], "LABELS": ["negative", "brightness", "hue", "red", "green", "blue"] },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "solarize", "LABEL": "Solarize (only above)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

vec3 hueRotate(vec3 c, float turns) {
    float a = turns * 6.2831853;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c * ca + cross(k, c) * sin(a) + k * dot(k, c) * (1.0 - ca);
}

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    vec3 c = s.rgb;
    vec3 inv;
    if (mode == 0) inv = 1.0 - c;
    else if (mode == 1) {
        // brightness flips, the color stays: bright red becomes dark red
        float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
        inv = clamp(c + (1.0 - 2.0 * l), 0.0, 1.0);
    }
    else if (mode == 2) inv = clamp(hueRotate(c, 0.5), 0.0, 1.0);
    else if (mode == 3) inv = vec3(1.0 - c.r, c.g, c.b);
    else if (mode == 4) inv = vec3(c.r, 1.0 - c.g, c.b);
    else inv = vec3(c.r, c.g, 1.0 - c.b);
    // solarize: only the parts brighter than the threshold flip (Sabattier)
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    float where = solarize > 0.0 ? smoothstep(solarize - 0.02, solarize + 0.02, l) : 1.0;
    gl_FragColor = vec4(mix(c, inv, amount * where), s.a);
}
