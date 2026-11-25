/*{
  "DESCRIPTION": "Brightness, contrast, saturation, hue, gamma and a color gain in one place. At the defaults the picture is unchanged; pushed to the ends it burns out, crushes, turns neon or goes gray.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "brightness", "LABEL": "Brightness", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "contrast", "LABEL": "Contrast", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 8.0 },
    { "NAME": "saturation", "LABEL": "Saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 8.0 },
    { "NAME": "hue", "LABEL": "Hue shift", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "hueSpeed", "LABEL": "Hue cycle (turns/s)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -4.0, "MAX": 4.0 },
    { "NAME": "gamma", "LABEL": "Gamma", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0 },
    { "NAME": "exposure", "LABEL": "Exposure (stops)", "TYPE": "float", "DEFAULT": 0.0, "MIN": -6.0, "MAX": 6.0 },
    { "NAME": "gain", "LABEL": "Color gain", "TYPE": "color", "DEFAULT": [1.0, 1.0, 1.0, 1.0] }
  ]
}*/

// hue rotation around the gray axis (keeps the brightness)
vec3 hueRotate(vec3 c, float turns) {
    float a = turns * 6.2831853;
    vec3 k = vec3(0.57735);
    float ca = cos(a);
    return c * ca + cross(k, c) * sin(a) + k * dot(k, c) * (1.0 - ca);
}

void main() {
    vec4 s = IMG_THIS_PIXEL(inputImage);
    vec3 c = s.rgb * exp2(exposure) * gain.rgb;
    c = hueRotate(c, hue + TIME * hueSpeed);
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(vec3(l), c, saturation);
    c = (c - 0.5) * contrast + 0.5 + brightness;
    c = pow(max(c, 0.0), vec3(1.0 / gamma));
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), s.a);
}
