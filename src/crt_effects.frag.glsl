#version 330 core

uniform sampler2D frame;
uniform vec2 pixelSize;
uniform float time;

uniform vec3 bgColor;
uniform vec3 fgColor;

in vec2 uv;

// const vec3 BG_COLOR = vec3(0.03, 0.12, 0.04);
// const vec3 FG_COLOR = vec3(0.23, 0.89, 0.29);

vec4 sample(vec2 uv) {
    return texture(frame, uv);
}

float rand(vec2 seed) {
    return fract(sin(dot(seed, vec2(12.9898, 78.233))) * 43758.5453);
}

vec3 lerp(vec3 a, vec3 b, vec3 t) {
    return a + t * (b - a);
}

vec3 clamp01(vec3 i) {
    return vec3(
        clamp(i.x, 0.0, 1.0),
        clamp(i.y, 0.0, 1.0),
        clamp(i.z, 0.0, 1.0)
    );
}

const int BLUR_SIZE = 12;
const float BLUR_QUALITY = 0.75;

void main() {

    int mid = BLUR_SIZE / 2;
    vec3 color = vec3(0);
    float rand_mod = rand(uv) * 0.08 + 0.92;
    for (int x = 0; x < BLUR_SIZE; x++) {
        for (int y = 0; y < BLUR_SIZE; y++) {
            vec2 xy = vec2(x, y);
            vec2 diff = xy - mid;

            vec2 diff_norm = diff / mid;
            float dist = sqrt(diff_norm.x * diff_norm.x + diff_norm.y * diff_norm.y);
            if (dist > 1) {
                continue;
            }
            float idist = clamp(1 - dist, 0.0, 1.0);

            vec2 new_uv = uv + pixelSize * diff / BLUR_QUALITY * rand_mod;
            color += sample(new_uv).rgb * idist * rand_mod * 0.2;
        }
    }
    color /= BLUR_SIZE;

    color = max(color, clamp01(color + sample(uv).rgb));

    float light = (color.r + color.g + color.b) / 3;
    color = lerp(bgColor, fgColor, vec3(light));

    float scanline = 1 - fract(mod(time, 5.0) / 5 + uv.y);
    float scanline_start = 0.75;
    if (scanline < scanline_start) {
        scanline = 0;
    } else {
        scanline = -cos(1.57079 * ((scanline - scanline_start) / (1 - scanline_start))) + 1;
    }

    gl_FragColor = vec4(color * (1 + scanline * 0.3) * rand_mod, 1);
}