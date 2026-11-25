/*{
  "DESCRIPTION": "Composite video with the faults of tired analog gear: a real NTSC / PAL encode and decode (dot crawl, rainbows, smeared color) plus random events: line jitter and tape wobble, sync tears, vertical roll, the color carrier losing lock, level errors, dropouts, head-switching noise and interference bursts.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "analog", "glitch", "stylize"],
  "INPUTS": [
    { "NAME": "inputImage",   "TYPE": "image" },
    { "NAME": "uChaos",       "LABEL": "Chaos (all faults)",              "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 2.0 },
    { "NAME": "uRate",        "LABEL": "Glitch rate",               "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSeed",        "LABEL": "Random seed",               "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 100.0 },
    { "NAME": "uJitter",      "LABEL": "Line jitter",              "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uWobble",      "LABEL": "Tape wobble",                "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uTearing",     "LABEL": "Sync tears",               "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uRoll",        "LABEL": "Vertical roll",                "TYPE": "float", "DEFAULT": 0.2,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uColorLock",   "LABEL": "Color unlock",          "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uLevelErr",    "LABEL": "Level errors",               "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uDropouts",    "LABEL": "Dropouts",                   "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uHeadSwitch",  "LABEL": "Head switching",            "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uBursts",      "LABEL": "Interference bursts",           "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uStandard",    "LABEL": "Standard (0 NTSC, 1 PAL)",       "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSamples",     "LABEL": "Signal resolution",          "TYPE": "float", "DEFAULT": 480.0,"MIN": 160.0,"MAX": 1200.0 },
    { "NAME": "uNotch",       "LABEL": "Color carrier filter",         "TYPE": "float", "DEFAULT": 0.6,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uComb",        "LABEL": "Comb filter / delay line", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uChromaBlur",  "LABEL": "Color blur",            "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uChromaDelay", "LABEL": "Color delay",               "TYPE": "float", "DEFAULT": 0.25, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uSaturation",  "LABEL": "Saturation",                 "TYPE": "float", "DEFAULT": 1.1,  "MIN": 0.0,  "MAX": 2.5 },
    { "NAME": "uSharpen",     "LABEL": "Sharpening (peaking)",       "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 2.0 },
    { "NAME": "uSnow",        "LABEL": "Antenna noise",          "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uMix",         "LABEL": "Mix",                        "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 1.0 }
  ]
}*/

// COMPOSITE VIDEO GLITCH — composite-video.fs's encode/decode (see there) with analog faults.
//
// Faults act where they would in real gear:
//   per line, before encoding: timing (jitter, wobble, sync tears, head switching) shifts where
//     the line's picture is read from; dropouts replace a line by noise or the previous line;
//     vertical roll re-maps lines and runs a black blanking bar through the picture;
//   on the signal: black-level (DC) errors per band of lines, pumping gain, bursts of snow and
//     interference;
//   in the decoder: the colour carrier loses lock - its phase drifts, and in unlock events
//     slides along the line (rainbow bands); the colour killer flickers; the colour delay jumps.
// Random events are rolled on a glitch clock (gTick, uRate) and scaled by uChaos.

#define PI  3.14159265
#define TAU 6.2831853

float gLines;     // virtual line count
bool  gPAL;
float gTick;      // glitch clock: events re-roll at every tick
float gFrame;     // per-frame randomness
float gChaos;

float hash(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }
float hash1(float n){ return fract(sin(n*12.9898 + 4.1)*43758.5453); }

// An event of kind `id` fires this tick with probability `p` (already scaled by uChaos).
bool event(float id, float p){ return hash(vec2(gTick, id)) < p*gChaos; }

vec3 rgb2yuv(vec3 c){
    float y = dot(c, vec3(0.299, 0.587, 0.114));
    return vec3(y, 0.492*(c.b - y), 0.877*(c.r - y));
}
vec3 yuv2rgb(vec3 c){
    return vec3(c.x + 1.140*c.z, c.x - 0.395*c.y - 0.581*c.z, c.x + 2.032*c.y);
}

float carrierPhase(float s, float L){
    float perLine  = gPAL ? 1.5*PI : PI;
    float perFrame = gPAL ? 0.5*PI : PI;
    return s*0.5*PI + mod(L, 4.0)*perLine + mod(gFrame, 4.0)*perFrame;
}
float vSwitch(float L){
    return gPAL ? (mod(L, 2.0) < 1.0 ? 1.0 : -1.0) : 1.0;
}

// ---------------------------------------------------------------- line faults

// Vertical roll: while V-hold is lost the picture slides vertically; the region where it wraps
// is the vertical blanking interval, shown as a black bar. Returns the source line (or -1 = bar).
float rollLine(float L){
    float slow = floor(gTick*0.25);
    if (hash(vec2(slow, 71.0)) >= uRoll*0.45*gChaos) return L;
    float speed = mix(0.2, 1.5, hash(vec2(slow, 72.0)))*(hash(vec2(slow, 73.0)) < 0.5 ? -1.0 : 1.0);
    float off = fract(TIME*speed*0.5 + hash(vec2(slow, 74.0)))*gLines;
    float Ls = mod(L + off, gLines*1.08);
    return Ls >= gLines ? -1.0 : Ls;
}

// Horizontal timing error of line L, in samples.
float lineShift(float L){
    float sh = 0.0;
    // Jitter: every line, every frame, slightly off.
    sh += (hash(vec2(L, gFrame)) - 0.5)*uJitter*5.0;
    // Tape wobble: slow wavy bend of the lines.
    sh += uWobble*(7.0*sin(L*0.045 + TIME*2.3) + 3.0*sin(L*0.19 - TIME*6.1));
    // Sync tears: a block of lines loses horizontal sync and rips sideways, slanting.
    float blockH = floor(mix(6.0, 70.0, hash(vec2(gTick, 3.0))));
    float block = floor(L/blockH);
    float hb = hash(vec2(block, gTick + 11.0));
    if (hb < uTearing*0.22*gChaos){
        float within = L - block*blockH;
        sh += (hash(vec2(block, gTick + 12.0)) - 0.5)*uTearing*uSamples*0.5
            + within*(hash(vec2(block, gTick + 13.0)) - 0.5)*uTearing*4.0;
    }
    // Head switching: the bottom lines bend away.
    float hsLines = gLines*0.035;
    if (L < hsLines)
        sh += uHeadSwitch*(hsLines - L)*(2.5 + 2.0*hash(vec2(L, gFrame)));
    return sh;
}

// 0 = normal, 1 = noise line, 2 = repeat of the line above (dropout compensator).
float dropoutMode(float L, float s){
    float h = hash(vec2(L, gFrame + 0.37));
    if (h >= uDropouts*0.03*gChaos) return 0.0;
    float x0 = hash(vec2(L, gFrame + 1.7))*uSamples;
    float len = hash(vec2(L, gFrame + 2.9))*uSamples*0.6;
    if (s < x0 || s > x0 + len) return 0.0;
    return hash(vec2(L, gFrame + 4.3)) < 0.5 ? 1.0 : 2.0;
}

// Black-level clamp error of the band of lines L is in.
float levelError(float L){
    float bandH = floor(mix(8.0, 50.0, hash(vec2(gTick, 21.0))));
    float band = floor(L/bandH);
    if (hash(vec2(band, gTick + 22.0)) >= uLevelErr*0.35*gChaos) return 0.0;
    return (hash(vec2(band, gTick + 23.0)) - 0.5)*uLevelErr*0.6;
}

// ---------------------------------------------------------------- signal

// Composite of the clean picture at sample s of line L, with the line faults applied.
float clean(float s, float L){
    float Ls = rollLine(L);
    if (Ls < 0.0) return -0.05;                       // blanking bar (blacker than black)
    float mode = dropoutMode(L, s);
    if (mode > 0.5 && mode < 1.5) return hash(vec2(s, L + gFrame*3.1))*1.2 - 0.1;
    if (mode > 1.5) Ls = max(Ls - 1.0, 0.0);
    vec2 uv = vec2((s + lineShift(L) + 0.5)/uSamples, (Ls + 0.5)/gLines);
    if (uv.x < 0.0 || uv.x > 1.0) return 0.0;         // torn off the line: black
    vec3 yuv = rgb2yuv(IMG_NORM_PIXEL(inputImage, clamp(uv, 0.0, 1.0)).rgb);
    float ph = carrierPhase(s, L);
    return yuv.x + yuv.y*sin(ph) + yuv.z*cos(ph)*vSwitch(L);
}

float composite(float s, float L){
    float c = clean(s, L) + levelError(L);
    // Pumping gain (AGC hunting).
    c *= 1.0 + uLevelErr*0.25*sin(TIME*1.7 + 2.0*sin(TIME*0.31));
    // Bursts: snow and interference, plus the always-on antenna snow.
    float snow = uSnow + (event(31.0, uBursts*0.35) ? uBursts*0.6 : 0.0);
    if (snow > 0.0)
        c += (hash(vec2(s, L) + gFrame*1.37) - 0.5)*snow*1.2;
    if (event(32.0, uBursts*0.3)){
        float f = mix(0.05, 0.4, hash(vec2(gTick, 33.0)));
        c += uBursts*0.35*sin(TAU*(s*f + L*hash(vec2(gTick, 34.0))*0.5) + TIME*9.0);
    }
    return c;
}

float lumaAt(float s, float L){
    float raw = composite(s, L);
    float notched = 0.125*composite(s - 2.0, L) + 0.25*composite(s - 1.0, L) + 0.25*raw
                  + 0.25*composite(s + 1.0, L) + 0.125*composite(s + 2.0, L);
    float y = mix(raw, notched, uNotch);
    if (uComb > 0.0){
        float Lc = gPAL ? L - 2.0 : L - 1.0;
        y = mix(y, 0.5*(raw + composite(s, Lc)), uComb);
    }
    return y;
}

// Decoder phase error of line L at sample s: slow drift, plus unlock events where the phase
// slides along the line (bands of cycling hue).
float phaseError(float s, float L){
    float drift = uColorLock*0.6*sin(TIME*0.37 + gChaos*hash1(uSeed));
    float noise = (hash(vec2(L, gFrame)) - 0.5)*uColorLock*0.6;
    float slide = 0.0;
    if (event(41.0, uColorLock*0.4))
        slide = s*(hash(vec2(gTick, 42.0)) - 0.5)*uColorLock*0.08;
    return drift + noise + slide;
}

vec2 chromaAt(float s, float L){
    float sigma = mix(1.0, 7.0, uChromaBlur);
    vec2 acc = vec2(0.0);
    float wsum = 0.0;
    for (int i = -10; i <= 10; i++){
        float k = float(i);
        float w = exp(-k*k/(2.0*sigma*sigma));
        float ph = carrierPhase(s + k, L) + phaseError(s + k, L);
        acc += w*composite(s + k, L)*vec2(sin(ph), cos(ph));
        wsum += w;
    }
    vec2 uv = 2.0*acc/max(wsum, 1e-4);
    uv.y *= vSwitch(L);
    return uv;
}

void main(){
    vec2 res = RENDERSIZE.xy;
    gPAL = uStandard >= 0.5;
    gLines = floor(uSamples*res.y/res.x + 0.5);
    gChaos = uChaos;
    gFrame = float(FRAMEINDEX);
    gTick = floor(TIME*mix(1.5, 25.0, uRate)) + floor(uSeed)*137.0;

    vec2 uvOut = gl_FragCoord.xy/res;
    float s = floor(uvOut.x*uSamples);
    float L = floor(uvOut.y*gLines);

    float y = lumaAt(s, L);
    if (uSharpen > 0.0)
        y += uSharpen*(y - 0.5*(lumaAt(s - 2.0, L) + lumaAt(s + 2.0, L)));

    // Colour delay, which jumps in colour-lock events.
    float delay = uChromaDelay;
    if (event(51.0, uColorLock*0.3))
        delay += (hash(vec2(gTick, 52.0)) - 0.5)*uColorLock*3.0;
    float cs = s - floor(delay*6.0 + 0.5);
    vec2 uv = chromaAt(cs, L);
    if (gPAL && uComb > 0.0)
        uv = mix(uv, 0.5*(uv + chromaAt(cs, L - 1.0)), uComb);

    // Colour killer: a snowy signal loses colour; in lock events it flickers off.
    float killer = 1.0 - smoothstep(0.35, 0.8, uSnow);
    if (event(61.0, uColorLock*0.12)) killer = 0.0;
    uv *= uSaturation*killer;

    vec3 col = clamp(yuv2rgb(vec3(y, uv)), 0.0, 1.0);
    vec3 orig = IMG_NORM_PIXEL(inputImage, uvOut).rgb;
    gl_FragColor = vec4(mix(orig, col, uMix), 1.0);
}
