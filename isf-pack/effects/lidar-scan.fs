/*{
  "DESCRIPTION": "The picture becomes a 3D point cloud of shaded spheres: brightness is height. Orbit around it, fly over it, dive through it or stand inside it; a scan plane sweeps through, with ground grid, axes and telemetry.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["effect", "hud", "3d", "analysis"],
  "INPUTS": [
    { "NAME": "inputImage", "TYPE": "image" },
    { "NAME": "uDensity",  "LABEL": "Point density",        "TYPE": "float", "DEFAULT": 90.0, "MIN": 20.0, "MAX": 320.0 },
    { "NAME": "uPointSize","LABEL": "Sphere size",       "TYPE": "float", "DEFAULT": 0.38, "MIN": 0.05, "MAX": 0.5 },
    { "NAME": "uHeight",   "LABEL": "Height",              "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSmooth",   "LABEL": "Smoothing",          "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uJitter",   "LABEL": "Measurement noise",       "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uShade",    "LABEL": "Sphere shading", "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uSpecular", "LABEL": "Highlight",         "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uColorMode","LABEL": "Coloring (0 height, 1 picture)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "uCamMode",  "LABEL": "Camera (0 orbit, 1 fly-over, 2 dive, 3 tripod)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 },
    { "NAME": "uSpeed",    "LABEL": "Speed",              "TYPE": "float", "DEFAULT": 0.15, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uTilt",     "LABEL": "Tilt",        "TYPE": "float", "DEFAULT": 0.45, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uYaw",      "LABEL": "Heading",      "TYPE": "float", "DEFAULT": 0.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uZoom",     "LABEL": "Zoom",               "TYPE": "float", "DEFAULT": 0.75, "MIN": 0.4,  "MAX": 2.5 },
    { "NAME": "uFov",      "LABEL": "Wide angle",         "TYPE": "float", "DEFAULT": 0.4,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uFlyHeight","LABEL": "Flight height",          "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uWeave",    "LABEL": "Weaving",         "TYPE": "float", "DEFAULT": 0.3,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uCamX",     "LABEL": "Tripod X",           "TYPE": "float", "DEFAULT": 0.0,  "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uCamZ",     "LABEL": "Tripod Z",           "TYPE": "float", "DEFAULT": 0.0,  "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "uTile",     "LABEL": "Endless (tiled)", "TYPE": "bool",  "DEFAULT": true },
    { "NAME": "uFog",      "LABEL": "Depth fog",       "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uScan",     "LABEL": "Scan",               "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uGrid",     "LABEL": "Ground grid",        "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uLabels",   "LABEL": "Labels",             "TYPE": "float", "DEFAULT": 1.0,  "MIN": 0.0,  "MAX": 1.0 },
    { "NAME": "uTextSize", "LABEL": "Text size",        "TYPE": "float", "DEFAULT": 2.0,  "MIN": 1.0,  "MAX": 4.0 },
    { "NAME": "uHudHue",   "LABEL": "HUD color",          "TYPE": "float", "DEFAULT": 0.5,  "MIN": 0.0,  "MAX": 1.0 }
  ],
  "PASSES": [
    { "TARGET": "heightBuf", "FLOAT": true },
    {}
  ]
}*/

// LIDAR SCAN v2
//   pass 0 heightBuf: blurred brightness (the relief) in alpha, the picture's colour in rgb.
//   pass 1: the measured points sit on a square grid over the picture (world x in [-aspect, aspect],
//           z in [-1, 1], y = height). Each grid cell holds one sphere of radius <= half a cell,
//           so a sphere never leaves its cell column. The ray walks the cells front to back
//           (2D DDA in xz, clipped to the height slab and, untiled, to the picture's box) and
//           tests one sphere per cell. Spheres are anti-aliased by pixel-footprint coverage and
//           composited front to back, so the first opaque hit ends the walk.

#define TAU 6.2831853
#define MAXSTEPS 420

float luma(vec3 c){ return dot(c, vec3(0.299, 0.587, 0.114)); }
vec3 hsv(float h, float s, float v){
    return v*mix(vec3(1.0), clamp(abs(mod(h*6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0), s);
}
float hash21(vec2 p){ p = fract(p*vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x*p.y); }

// ---------------------------------------------------------------- 3x5 font
float glyph(float code, vec2 g){
    if (g.x < 0.0 || g.y < 0.0 || g.x >= 3.0 || g.y >= 5.0) return 0.0;
    float idx = (4.0 - floor(g.y))*3.0 + floor(g.x);
    return mod(floor(code/exp2(idx)), 2.0);
}
float digitCode(float d){
    if (d < 0.5) return 31599.0; if (d < 1.5) return 29850.0; if (d < 2.5) return 29671.0;
    if (d < 3.5) return 31207.0; if (d < 4.5) return 18925.0; if (d < 5.5) return 31183.0;
    if (d < 6.5) return 31695.0; if (d < 7.5) return 9383.0;  if (d < 8.5) return 31727.0;
    return 31215.0;
}
float number(vec2 p, vec2 o, float s, float v, float ints, float decs){
    vec2 g = (p - o)/s;
    float i = floor(g.x/4.0);
    if (g.y < 0.0 || g.y >= 5.0 || i < 0.0 || i >= ints + decs + (decs > 0.0 ? 1.0 : 0.0)) return 0.0;
    vec2 cell = g - vec2(i*4.0, 0.0);
    v = abs(v);
    if (decs > 0.0 && abs(i - ints) < 0.5) return glyph(8192.0, cell);
    float k = i < ints ? ints - 1.0 - i : -(i - ints);
    float d = mod(floor(v/pow(10.0, k) + 1e-4), 10.0);
    return glyph(digitCode(d), cell);
}

// Mirror-repeat so tiled copies meet without seams.
vec2 mirrorUV(vec2 uv){ return 1.0 - abs(1.0 - mod(uv, 2.0)); }

void main(){
    vec2 res = RENDERSIZE.xy;
    vec2 px = gl_FragCoord.xy;

    if (PASSINDEX == 0){
        vec2 uv = px/res;
        float r = (1.0 + uSmooth*12.0)/res.y;
        float h = 0.0;
        for (int j = -2; j <= 2; j++)
            for (int i = -2; i <= 2; i++)
                h += luma(IMG_NORM_PIXEL(inputImage, uv + vec2(float(i), float(j))*r*vec2(res.y/res.x, 1.0)).rgb);
        gl_FragColor = vec4(IMG_NORM_PIXEL(inputImage, uv).rgb, h/25.0);
        return;
    }

    vec3 hud = hsv(uHudHue, 0.7, 1.0);
    float A = res.x/res.y;
    bool tiled = uTile > 0.5;

    // ---------------------------------------------------------------- grid
    float cs = 2.0/floor(uDensity);                 // cell size in world units
    vec2 N = vec2(max(1.0, floor(2.0*A/cs)), floor(uDensity));
    vec2 org = vec2(-N.x*cs*0.5, -1.0);             // world xz of grid corner (0,0)
    float rad = uPointSize*cs;
    float jit = uJitter*max(0.0, 0.5*cs - rad);     // keeps every sphere inside its cell

    // ---------------------------------------------------------------- camera
    float mode = floor(uCamMode + 0.5);
    float T = TIME*uSpeed;
    float yaw0 = uYaw*TAU;
    float focal = mix(2.4, 0.6, uFov);
    float pitch = (uTilt - 0.2)*1.8;                // flight modes: look down (+) or up (-)
    float wv = T*0.9;
    float roll = 0.0;
    float camFade = 1.0;
    vec3 ro, fw;
    float flyY = 0.02 + uFlyHeight*1.4;
    float Lx = N.x*cs*0.5, Lz = 1.0;
    if (mode < 0.5){
        // Orbit around the centre (as v1).
        float ang = T*0.3 + 0.6 + yaw0;
        float tilt = mix(0.08, 1.45, uTilt);
        float dist = 3.2/uZoom;
        vec3 ta = vec3(0.0, uHeight*0.3, 0.0);
        ro = ta + vec3(sin(ang)*cos(tilt), sin(tilt), cos(ang)*cos(tilt))*dist;
        fw = normalize(ta - ro);
    } else {
        focal *= uZoom/0.75;
        vec3 d = vec3(sin(yaw0), 0.0, cos(yaw0));
        vec3 side = vec3(cos(yaw0), 0.0, -sin(yaw0));
        float yawW = cos(wv)*uWeave*0.35;
        roll = -cos(wv)*uWeave*0.25;
        if (mode < 1.5){
            // Fly over: straight line along the view direction, weaving sideways.
            // Untiled: start well before the cloud (so it comes into view) and end past it.
            float L = abs(d.x)*Lx + abs(d.z)*Lz;
            float span = 2.0*L + 1.5;
            float ph = T*0.6/span;
            float along = tiled ? T*0.6 : fract(ph)*span - L - 1.5;
            if (!tiled) camFade = smoothstep(0.0, 0.05, fract(ph))*smoothstep(1.0, 0.95, fract(ph));
            ro = d*along + side*sin(wv)*uWeave*0.6 + vec3(0.0, flyY, 0.0);
            float y = yaw0 + yawW;
            fw = vec3(sin(y)*cos(pitch), -sin(pitch), cos(y)*cos(pitch));
        } else if (mode < 2.5){
            // Dive: from far above and outside, straight into the cloud and through it, looping.
            float dp = mix(0.05, 1.4, uTilt);
            vec3 back = vec3(-d.x*cos(dp), sin(dp), -d.z*cos(dp));
            vec3 ta = vec3(0.0, uFlyHeight*uHeight, 0.0);
            float D = 3.2/uZoom + 0.5;
            // From D (far) to just under the point layer, then the loop starts over.
            float sEnd = -(ta.y + 0.08)/max(back.y, 0.05);
            float ph = fract(T*0.6/(D - sEnd));
            float s = D - ph*(D - sEnd);
            camFade = smoothstep(0.0, 0.05, ph)*smoothstep(1.0, 0.9, ph);
            ro = ta + back*s + side*sin(wv)*uWeave*0.3;
            fw = normalize(-back + side*yawW);
        } else {
            // Tripod: fixed position, panning with Tempo.
            ro = vec3(uCamX*Lx, flyY, uCamZ*1.2);
            float y = yaw0 + T*0.5;
            fw = vec3(sin(y)*cos(pitch), -sin(pitch), cos(y)*cos(pitch));
        }
    }
    vec3 rt = normalize(cross(fw, vec3(0.0, 1.0, 0.0)));
    vec3 up = cross(rt, fw);
    vec3 rt2 = rt*cos(roll) + up*sin(roll);
    up = cross(rt2, fw); rt = rt2;
    vec2 sp = (px - res*0.5)/res.y;
    vec3 rd = normalize(fw*focal + rt*sp.x + up*sp.y);
    float pixAng = 1.0/(res.y*focal);               // world size of a pixel per unit distance

    float fogK = mix(0.0, 0.9, uFog);
    float tFar = fogK > 0.0 ? min(40.0, 4.6/fogK + 2.0) : 40.0;

    // ---------------------------------------------------------------- ground
    vec3 col = vec3(0.0);
    float tg = (rd.y < 0.0 && ro.y > 0.0) ? -ro.y/rd.y : -1.0;

    // ---------------------------------------------------------------- spheres
    // Clip the ray to the slab the spheres live in, and (untiled) to the picture's box.
    float yLo = -rad - jit, yHi = uHeight + rad + jit;
    float t0 = 0.0, t1 = tFar;
    if (abs(rd.y) > 1e-5){
        float a = (yLo - ro.y)/rd.y, b = (yHi - ro.y)/rd.y;
        t0 = max(t0, min(a, b)); t1 = min(t1, max(a, b));
    } else if (ro.y < yLo || ro.y > yHi) t1 = -1.0;
    if (!tiled){
        vec2 bmin = org, bmax = org + N*cs;
        vec2 inv = 1.0/max(abs(rd.xz), vec2(1e-6))*sign(rd.xz + vec2(1e-12));
        vec2 a = (bmin - ro.xz)*inv, b = (bmax - ro.xz)*inv;
        vec2 lo = min(a, b), hi = max(a, b);
        t0 = max(t0, max(lo.x, lo.y)); t1 = min(t1, min(hi.x, hi.y));
    }

    vec3 acc = vec3(0.0);
    float alpha = 0.0;
    float hitT = -1.0;
    vec3 L1 = normalize(vec3(-0.45, 0.8, 0.35));
    float sx = (fract(TIME*mix(0.05, 0.5, uScan))*2.4 - 1.2)*A + (tiled ? ro.x : 0.0);

    if (t1 > t0){
        float t = t0 + 1e-4;
        vec2 g = (ro.xz + rd.xz*t - org)/cs;          // grid coordinates
        vec2 id = floor(g);
        vec2 dg = rd.xz/cs;
        vec2 st = vec2(dg.x >= 0.0 ? 1.0 : -1.0, dg.y >= 0.0 ? 1.0 : -1.0);
        vec2 adg = max(abs(dg), vec2(1e-6));
        vec2 tDelta = 1.0/adg;
        vec2 tMax = t + (st*(id - g) + max(st, 0.0))*tDelta;
        for (int i = 0; i < MAXSTEPS; i++){
            bool inside = tiled || (id.x >= 0.0 && id.y >= 0.0 && id.x < N.x && id.y < N.y);
            if (inside){
                vec2 uv = (id + 0.5)/N;
                vec2 suv = tiled ? mirrorUV(uv) : uv;
                vec4 hb = IMG_NORM_PIXEL(heightBuf, suv);
                vec3 jr = vec3(hash21(id), hash21(id + 17.3), hash21(id + 41.7)) - 0.5;
                vec3 c = vec3(org.x + (id.x + 0.5)*cs, hb.a*uHeight, org.y + (id.y + 0.5)*cs) + jr*2.0*jit;
                vec3 oc = c - ro;
                float b = dot(oc, rd);
                if (b - rad > 0.02){                  // near clip: the camera flies through spheres
                    float d = length(oc - rd*b);
                    float pw = b*pixAng;
                    float cov = clamp((rad - d)/max(pw, 1e-5) + 0.5, 0.0, 1.0);
                    if (cov > 0.0){
                        float th = sqrt(max(rad*rad - d*d, 0.0));
                        float tt = b - th;
                        vec3 p = ro + rd*max(tt, 0.0);
                        vec3 n = d < rad ? normalize(p - c) : normalize(ro + rd*b - c);
                        float hn = hb.a;
                        vec3 pc = mix(hsv(0.66 - 0.66*hn, 0.85, 1.0), hb.rgb*1.3, uColorMode);
                        float dif = max(dot(n, L1), 0.0);
                        float head = max(dot(n, -rd), 0.0);
                        vec3 hv = normalize(L1 - rd);
                        float spc = pow(max(dot(n, hv), 0.0), 40.0)*uSpecular*1.5;
                        float rim = pow(1.0 - head, 3.0)*0.35;
                        vec3 shaded = pc*(0.18 + 0.62*dif + 0.35*head) + vec3(spc) + hud*rim*0.4;
                        vec3 sc = mix(pc*(0.75 + 0.25*hn), shaded, uShade);
                        float scan = uScan > 0.0 ? exp(-abs(c.x - sx)*18.0) : 0.0;
                        sc += vec3(0.9)*scan + hud*scan*0.15;
                        sc *= exp(-tt*fogK*0.5);
                        acc += (1.0 - alpha)*cov*sc;
                        if (hitT < 0.0) hitT = tt;
                        alpha += (1.0 - alpha)*cov;
                        if (alpha > 0.985) break;
                    }
                }
            }
            float tn = min(tMax.x, tMax.y);
            if (tn > t1) break;
            if (tMax.x < tMax.y){ tMax.x += tDelta.x; id.x += st.x; }
            else                { tMax.y += tDelta.y; id.y += st.y; }
        }
    }

    // Ground grid behind the spheres.
    if (uGrid > 0.0 && tg > 0.0 && tg < tFar){
        vec3 gp = ro + rd*tg;
        vec2 gl = abs(fract(gp.xz*4.0 + 0.5) - 0.5)/max(fwidth(gp.xz*4.0), vec2(1e-4));
        float line = 1.0 - clamp(min(gl.x, gl.y) - 0.5, 0.0, 1.0);
        float fade = exp(-tg*max(fogK, 0.12)*0.6);
        col += hud*line*0.35*uGrid*fade;
        vec2 ax = abs(gp.xz)/max(fwidth(gp.xz), vec2(1e-4));
        col += vec3(1.0, 0.3, 0.3)*(1.0 - clamp(ax.y - 0.8, 0.0, 1.0))*uGrid*fade;
        col += vec3(0.3, 0.5, 1.0)*(1.0 - clamp(ax.x - 0.8, 0.0, 1.0))*uGrid*fade;
    }
    col = (acc + (1.0 - alpha)*col)*camFade;

    // ---------------------------------------------------------------- telemetry
    if (uLabels > 0.5){
        float lw = max(1.0, floor(res.y/540.0 + 0.5));
        float s = floor(uTextSize*max(1.0, res.y/540.0));
        vec2 o = vec2(10.0*lw, res.y - 10.0*lw - 5.0*s);
        float pts = N.x*N.y;
        float az = mod(degrees(atan(fw.x, fw.z)) + 360.0, 360.0);
        float el = degrees(asin(clamp(-fw.y, -1.0, 1.0)));
        float txt = (glyph(29257.0, (px - (o))/s - vec2(0.0, 0.0)) + glyph(29847.0, (px - (o))/s - vec2(4.0, 0.0)) + glyph(15211.0, (px - (o))/s - vec2(8.0, 0.0)) + glyph(23530.0, (px - (o))/s - vec2(12.0, 0.0)) + glyph(23275.0, (px - (o))/s - vec2(16.0, 0.0)) + glyph(14478.0, (px - (o))/s - vec2(24.0, 0.0)) + glyph(25166.0, (px - (o))/s - vec2(28.0, 0.0)) + glyph(23530.0, (px - (o))/s - vec2(32.0, 0.0)) + glyph(23403.0, (px - (o))/s - vec2(36.0, 0.0)))
                  + (glyph(4843.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(0.0, 0.0)) + glyph(9367.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(4.0, 0.0)) + glyph(14478.0, (px - (o - vec2(0.0, 8.0*s)))/s - vec2(8.0, 0.0))) + number(px, o + vec2(16.0*s, -8.0*s), s, pts, 6.0, 0.0)
                  + (glyph(23530.0, (px - (o - vec2(0.0, 16.0*s)))/s - vec2(0.0, 0.0)) + glyph(29351.0, (px - (o - vec2(0.0, 16.0*s)))/s - vec2(4.0, 0.0))) + number(px, o + vec2(12.0*s, -16.0*s), s, az, 3.0, 1.0)
                  + (glyph(29391.0, (px - (o - vec2(0.0, 24.0*s)))/s - vec2(0.0, 0.0)) + glyph(29257.0, (px - (o - vec2(0.0, 24.0*s)))/s - vec2(4.0, 0.0))) + (el < 0.0 ? (glyph(448.0, (px - (o + vec2(8.0*s, -24.0*s)))/s - vec2(0.0, 0.0))) : 0.0)
                  + number(px, o + vec2(12.0*s, -24.0*s), s, el, 2.0, 1.0)
                  + (glyph(23530.0, (px - (o - vec2(0.0, 32.0*s)))/s - vec2(0.0, 0.0)) + glyph(29257.0, (px - (o - vec2(0.0, 32.0*s)))/s - vec2(4.0, 0.0)) + glyph(9367.0, (px - (o - vec2(0.0, 32.0*s)))/s - vec2(8.0, 0.0))) + (ro.y < 0.0 ? (glyph(448.0, (px - (o + vec2(12.0*s, -32.0*s)))/s - vec2(0.0, 0.0))) : 0.0)
                  + number(px, o + vec2(16.0*s, -32.0*s), s, ro.y*100.0, 3.0, 1.0)
                  + (glyph(4815.0, (px - (o - vec2(0.0, 40.0*s)))/s - vec2(0.0, 0.0)) + glyph(23275.0, (px - (o - vec2(0.0, 40.0*s)))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (o - vec2(0.0, 40.0*s)))/s - vec2(8.0, 0.0))) + number(px, o + vec2(16.0*s, -40.0*s), s, mod(float(FRAMEINDEX), 1e6), 6.0, 0.0);
        // Camera mode tag.
        vec2 om = o - vec2(0.0, 50.0*s);
        txt += mode < 0.5 ? (glyph(25166.0, (px - (om))/s - vec2(0.0, 0.0)) + glyph(23530.0, (px - (om))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (om))/s - vec2(8.0, 0.0)) + glyph(11114.0, (px - (om))/s - vec2(16.0, 0.0)) + glyph(23275.0, (px - (om))/s - vec2(20.0, 0.0)) + glyph(15083.0, (px - (om))/s - vec2(24.0, 0.0)) + glyph(29847.0, (px - (om))/s - vec2(28.0, 0.0)) + glyph(9367.0, (px - (om))/s - vec2(32.0, 0.0)))
             : mode < 1.5 ? (glyph(25166.0, (px - (om))/s - vec2(0.0, 0.0)) + glyph(23530.0, (px - (om))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (om))/s - vec2(8.0, 0.0)) + glyph(4815.0, (px - (om))/s - vec2(16.0, 0.0)) + glyph(29257.0, (px - (om))/s - vec2(20.0, 0.0)) + glyph(9389.0, (px - (om))/s - vec2(24.0, 0.0)) + glyph(11114.0, (px - (om))/s - vec2(28.0, 0.0)) + glyph(11117.0, (px - (om))/s - vec2(32.0, 0.0)) + glyph(29391.0, (px - (om))/s - vec2(36.0, 0.0)) + glyph(23275.0, (px - (om))/s - vec2(40.0, 0.0)))
             : mode < 2.5 ? (glyph(25166.0, (px - (om))/s - vec2(0.0, 0.0)) + glyph(23530.0, (px - (om))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (om))/s - vec2(8.0, 0.0)) + glyph(15211.0, (px - (om))/s - vec2(16.0, 0.0)) + glyph(29847.0, (px - (om))/s - vec2(20.0, 0.0)) + glyph(11117.0, (px - (om))/s - vec2(24.0, 0.0)) + glyph(29391.0, (px - (om))/s - vec2(28.0, 0.0)))
             :              (glyph(25166.0, (px - (om))/s - vec2(0.0, 0.0)) + glyph(23530.0, (px - (om))/s - vec2(4.0, 0.0)) + glyph(23549.0, (px - (om))/s - vec2(8.0, 0.0)) + glyph(9367.0, (px - (om))/s - vec2(16.0, 0.0)) + glyph(23275.0, (px - (om))/s - vec2(20.0, 0.0)) + glyph(29847.0, (px - (om))/s - vec2(24.0, 0.0)) + glyph(4843.0, (px - (om))/s - vec2(28.0, 0.0)) + glyph(11114.0, (px - (om))/s - vec2(32.0, 0.0)) + glyph(15211.0, (px - (om))/s - vec2(36.0, 0.0)));
        col = mix(col, hud, clamp(txt, 0.0, 1.0));
    }

    gl_FragColor = vec4(clamp(col, 0.0, 1.0), 1.0);
}
