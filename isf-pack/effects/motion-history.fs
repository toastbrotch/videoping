/*{
  "DESCRIPTION": "Every pixel remembers how long ago it last moved, shown as a color gradient: fresh motion hot, older motion cooling through the palette, then black. Movement leaves colored time trails and, with time rings, contour rings like growth rings in time.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "motion", "analysis", "stylize"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uThresh",   "LABEL": "Threshold",          "TYPE": "float", "DEFAULT": 0.08, "MIN": 0.01, "MAX": 0.5 },
    { "NAME": "uDuration", "LABEL": "Duration (s)",         "TYPE": "float", "DEFAULT": 2.0,  "MIN": 0.2,  "MAX": 10.0 },
    { "NAME": "uPalette",  "LABEL": "Palette (0-5)",     "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 5.0 },
    { "NAME": "uRings",    "LABEL": "Time rings (0 = smooth)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 24.0 },
    { "NAME": "uCycle",    "LABEL": "Color cycle",        "TYPE": "float", "DEFAULT": 0.0,  "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uUnder",    "LABEL": "Original underneath", "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "ageBuf",  "PERSISTENT": true, "FLOAT": true },
    { "TARGET": "prevBuf", "PERSISTENT": true, "FLOAT": true },
    {}
  ]
}*/

// MOTION HISTORY
//   pass 0 ageBuf:  seconds since this pixel last changed by more than the threshold (reads
//                   prevBuf while it still holds the last frame); 0 where it moves right now.
//   pass 1 prevBuf: this frame.
//   pass 2:         age -> 0..1 over the duration -> palette colour, fading towards black.

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 ramp(float t, vec3 a, vec3 b, vec3 c, vec3 d){
    t = clamp(t, 0.0, 1.0)*3.0;
    return t < 1.0 ? mix(a, b, t) : (t < 2.0 ? mix(b, c, t - 1.0) : mix(c, d, t - 2.0));
}

// t = 1 fresh, 0 old.
vec3 palette(int p, float t){
    if (p == 0) return ramp(t, vec3(0.05, 0.0, 0.25), vec3(0.65, 0.0, 0.55), vec3(1.0, 0.45, 0.0), vec3(1.0, 1.0, 0.85));
    if (p == 1) return clamp(abs(mod((1.0 - t)*4.5 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
    if (p == 2) return ramp(t, vec3(0.1, 0.0, 0.3), vec3(0.5, 0.0, 0.6), vec3(1.0, 0.2, 0.6), vec3(0.4, 1.0, 1.0));
    if (p == 3) return ramp(t, vec3(0.0, 0.03, 0.15), vec3(0.05, 0.3, 0.6), vec3(0.4, 0.85, 1.0), vec3(1.0));
    if (p == 4) return vec3(t);
    return ramp(t, vec3(0.1, 0.0, 0.15), vec3(0.3, 0.0, 0.4), vec3(0.4, 0.9, 0.0), vec3(0.95, 1.0, 0.5));
}

void main(){
    vec3 cur = IMG_THIS_NORM_PIXEL(inputImage).rgb;

    if (PASSINDEX == 0){
        float last = IMG_THIS_NORM_PIXEL(ageBuf).r;
        float moved = step(uThresh, abs(luma(cur) - luma(IMG_THIS_NORM_PIXEL(prevBuf).rgb)));
        // Before the first frames there is no history: everything starts "long ago".
        float age = FRAMEINDEX < 2 ? 1000.0 : (moved > 0.5 ? 0.0 : min(last + max(TIMEDELTA, 0.0), 1000.0));
        gl_FragColor = vec4(age, 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 1){
        gl_FragColor = vec4(cur, 1.0);
        return;
    }

    float age = IMG_THIS_NORM_PIXEL(ageBuf).r;
    float t = 1.0 - age/max(uDuration, 0.05);              // 1 = moving now, 0 = forgotten
    vec3 col = cur*uUnder;
    if (t > 0.0){
        float rings = floor(uRings);
        float tq = rings >= 1.0 ? ceil(t*rings)/rings : t; // contour steps: rings in time
        float pt = fract(tq + TIME*uCycle*0.25);
        if (uCycle == 0.0) pt = tq;
        int p = int(floor(clamp(uPalette, 0.0, 5.0) + 0.5));
        vec3 trail = palette(p, pt)*(0.25 + 0.75*sqrt(t));
        col = max(col, trail);
    }
    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
