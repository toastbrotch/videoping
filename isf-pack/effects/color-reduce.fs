/*{
  "DESCRIPTION": "Reduces the picture to its N most common colors, like the ColorReduce FreeFrame plugin: the colors of the frame are counted on a coarse 6 x 6 x 6 grid, the N most frequent become the palette, and every pixel takes the nearest of them. The palette comes from the picture itself and changes with it. Palette rotate swaps the colors while the shapes stay.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "color", "retro"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "colors", "LABEL": "Colors (64 = off)", "TYPE": "float", "DEFAULT": 8.0, "MIN": 2.0, "MAX": 64.0 },
    { "NAME": "rotate", "LABEL": "Palette rotate", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "dither", "LABEL": "Dithering", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "pixel", "LABEL": "Pixel size (px)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 1.0, "MAX": 64.0 }
  ],
  "PASSES": [
    { "TARGET": "rowBuf" },
    { "TARGET": "histBuf" },
    { "TARGET": "rankBuf" },
    { "TARGET": "palBuf" },
    { }
  ]
}*/

// The plugin's CPU steps as GPU passes. Bin b of the 6x6x6 color grid is b = 36 r + 6 g + b_
// (each channel quantized to 0..5). The tables keep one entry per pixel at the bottom of their
// buffer (the output is always wider than 216 pixels and higher than 72).
// videoping's buffers have 8 bits per channel, so:
//   pass 0 rowBuf:  pixel (b, y) = how many of the 128 sample points of sample row y fall into
//                   bin b (0..128, one byte); 216 x 72 small counts run in parallel
//   pass 1 histBuf: pixel b = the sum over the 72 rows (up to 9216), as a 16-bit number in red
//                   (high byte) and green (low byte)
//   pass 2 rankBuf: pixel b = its rank (how many bins are more common; ties by index), 0..215 in
//                   red as a whole byte
//   pass 3 palBuf:  pixel j = the color of the bin with rank j (alpha 0 if that bin is empty)
//   pass 4:         every pixel takes the nearest of the first N palette colors, shifted by the
//                   rotation, as the plugin's remap shader

const float BINS = 216.0;
const int SX = 128;
const int SY = 72;

float byteOf(float v) { return floor(v * 255.0 + 0.5); }

float countAt(float b) {
    vec4 h = IMG_PIXEL(histBuf, vec2(b + 0.5, 0.5));
    return byteOf(h.r) * 256.0 + byteOf(h.g);
}

float binOf(vec3 c) {
    vec3 q = min(floor(clamp(c, 0.0, 1.0) * 6.0), 5.0);
    return q.r * 36.0 + q.g * 6.0 + q.b;
}

// 4x4 ordered dither (Bayer), 0..1
float bayer4(vec2 p) {
    vec2 a = mod(p, 2.0), b = mod(floor(p / 2.0), 2.0);
    float m1 = a.x * 2.0 + a.y * 3.0 - a.x * a.y * 4.0;
    float m2 = b.x * 2.0 + b.y * 3.0 - b.x * b.y * 4.0;
    return (m1 * 4.0 + m2 + 0.5) / 16.0;
}

void main() {
    vec2 px = floor(gl_FragCoord.xy);
    bool table = px.y < 0.5 && px.x < BINS;

    if (PASSINDEX == 0) {
        if (!(px.x < BINS && px.y < float(SY))) { gl_FragColor = vec4(0.0); return; }
        float n = 0.0;
        for (int i = 0; i < SX; i++) {
            vec2 uv = (vec2(float(i), px.y) + 0.5) / vec2(float(SX), float(SY));
            if (abs(binOf(IMG_NORM_PIXEL(inputImage, uv).rgb) - px.x) < 0.5) n += 1.0;
        }
        gl_FragColor = vec4(n / 255.0, 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 1) {
        if (!table) { gl_FragColor = vec4(0.0); return; }
        float n = 0.0;
        for (int j = 0; j < SY; j++) n += byteOf(IMG_PIXEL(rowBuf, vec2(px.x + 0.5, float(j) + 0.5)).r);
        gl_FragColor = vec4(floor(n / 256.0) / 255.0, mod(n, 256.0) / 255.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 2) {
        if (!table) { gl_FragColor = vec4(0.0); return; }
        float mine = countAt(px.x);
        float rank = 0.0;
        for (int i = 0; i < 216; i++) {
            float fi = float(i);
            float h = countAt(fi);
            if (h > mine || (h == mine && fi < px.x)) rank += 1.0;
        }
        gl_FragColor = vec4(rank / 255.0, 0.0, 0.0, 1.0);
        return;
    }

    if (PASSINDEX == 3) {
        if (!(px.y < 0.5 && px.x < 64.0)) { gl_FragColor = vec4(0.0); return; }
        vec4 entry = vec4(0.0);
        for (int i = 0; i < 216; i++) {
            float fi = float(i);
            if (abs(byteOf(IMG_PIXEL(rankBuf, vec2(fi + 0.5, 0.5)).r) - px.x) < 0.5) {
                // the bin's center color, like the plugin's levelCenter()
                vec3 q = vec3(floor(fi / 36.0), mod(floor(fi / 6.0), 6.0), mod(fi, 6.0));
                entry = vec4((q + 0.5) / 6.0, countAt(fi) > 0.0 ? 1.0 : 0.0);
            }
        }
        gl_FragColor = entry;
        return;
    }

    // output: big pixels read the color at their center
    float ps = max(floor(pixel), 1.0);
    vec2 cell = floor(gl_FragCoord.xy / ps);
    vec4 src = IMG_NORM_PIXEL(inputImage, clamp((cell + 0.5) * ps / RENDERSIZE, 0.0, 1.0));
    float N = floor(clamp(colors, 2.0, 64.0));
    if (N >= 64.0) { gl_FragColor = src; return; }          // "full", as in the plugin
    vec3 c = src.rgb + (bayer4(cell) - 0.5) * dither * 0.35;

    // nearest of the first N non-empty palette entries (fewer if the frame has fewer colors)
    float used = 0.0, best = 0.0, bestD = 1e9;
    for (int j = 0; j < 64; j++) {
        float fj = float(j);
        if (fj >= N) break;
        vec4 p = IMG_PIXEL(palBuf, vec2(fj + 0.5, 0.5));
        if (p.a < 0.5) break;
        used += 1.0;
        vec3 d = c - p.rgb;
        float dist = dot(d, d);
        if (dist < bestD) { bestD = dist; best = fj; }
    }
    used = max(used, 1.0);
    float shift = mod(floor(rotate * used), used);
    float idx = mod(best + shift, used);
    gl_FragColor = vec4(IMG_PIXEL(palBuf, vec2(idx + 0.5, 0.5)).rgb, src.a);
}
