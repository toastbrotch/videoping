/*{
  "DESCRIPTION": "Shows what changed: moving parts light up, still areas go black. Exact mode compares with the frame 1 to 3 frames ago; memory mode compares with a slowly updating average, so motion glows and things that leave leave ghosts.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "stylize", "motion"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uMode",     "LABEL": "Mode (0 exact, 1 memory)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uDelay",    "LABEL": "Delay (frames)",          "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 3.0 },
    { "NAME": "uMemory",   "LABEL": "Memory",                    "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uHardness", "LABEL": "Hardness",                       "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uColor",    "LABEL": "Color",                          "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uUnder",    "LABEL": "Original underneath",              "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "f3", "PERSISTENT": true, "FLOAT": true },
    { "TARGET": "f2", "PERSISTENT": true, "FLOAT": true },
    { "TARGET": "f1", "PERSISTENT": true, "FLOAT": true },
    { "TARGET": "f0", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// FRAMEDIFF — four persistent buffers hand frames down a chain, oldest first, so each reads its
// neighbour before that one is overwritten: after the four passes f0 = this frame, f1 = 1 frame
// ago, f2 = 2, f3 = 3. (Only four, because the FreeFrame 1 ISF Effect allows four buffers.)
// The memory mode's running average lives in the alpha channels of f0, f1 and f2 - one colour
// channel each - so it fits in the same four buffers. Its time constant is in seconds (via
// TIMEDELTA), so it behaves the same at any frame rate.

vec3 inputRGB(){
    vec4 c = IMG_THIS_NORM_PIXEL(inputImage);
    if (c.a > 0.0) c.rgb /= c.a;          // FFGL hands effects premultiplied input
    return c.rgb;
}

// Running average of one channel: last average (this buffer's own alpha) moved towards `v`.
float average(float last, float v){
    float tau = mix(0.03, 10.0, uMemory*uMemory);               // seconds
    float k = 1.0 - exp(-max(TIMEDELTA, 0.0)/tau);
    return FRAMEINDEX < 2 ? v : mix(last, v, k);
}

void main(){
    vec3 cur = inputRGB();

    if (PASSINDEX == 0){                  // f3 <- f2
        gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(f2).rgb, 0.0);
        return;
    }
    if (PASSINDEX == 1){                  // f2 <- f1, alpha: average of blue
        gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(f1).rgb, average(IMG_THIS_NORM_PIXEL(f2).a, cur.b));
        return;
    }
    if (PASSINDEX == 2){                  // f1 <- f0, alpha: average of green
        gl_FragColor = vec4(IMG_THIS_NORM_PIXEL(f0).rgb, average(IMG_THIS_NORM_PIXEL(f1).a, cur.g));
        return;
    }
    if (PASSINDEX == 3){                  // f0 <- input, alpha: average of red
        gl_FragColor = vec4(cur, average(IMG_THIS_NORM_PIXEL(f0).a, cur.r));
        return;
    }

    // Output: difference against the delayed frame or the running average.
    vec3 past;
    if (uMode >= 0.5){
        past = vec3(IMG_THIS_NORM_PIXEL(f0).a, IMG_THIS_NORM_PIXEL(f1).a, IMG_THIS_NORM_PIXEL(f2).a);
    } else {
        float d = floor(clamp(uDelay, 1.0, 3.0) + 0.5);
        past = d < 1.5 ? IMG_THIS_NORM_PIXEL(f1).rgb : (d < 2.5 ? IMG_THIS_NORM_PIXEL(f2).rgb : IMG_THIS_NORM_PIXEL(f3).rgb);
    }
    if (FRAMEINDEX < 4 && uMode < 0.5) past = cur;   // the chain isn't filled yet

    vec3 diff = abs(cur - past);
    // Same contrast curve as the plugin: scale 1 at hardness 0, 256 at hardness 1.
    float scale = 1.0/(1.0 - uHardness + 1.0/256.0);
    float grey = clamp(dot(diff, vec3(0.299, 0.587, 0.114))*scale, 0.0, 1.0);
    vec3 col = clamp(diff*scale, 0.0, 1.0);
    vec3 v = mix(vec3(grey), col, uColor);
    gl_FragColor = vec4(clamp(cur*uUnder + v, 0.0, 1.0), 1.0);
}
