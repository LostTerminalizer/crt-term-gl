#version 330 core

precision highp float;
//!defs

uniform sampler2D frame;
uniform vec2 pixelSize;
uniform float time;
in vec2 uv;

const vec3 BG_COLOR = vec3(0.03, 0.12, 0.04);
const vec3 FG_COLOR = vec3(0.23, 0.89, 0.29);

vec4 sample(vec2 uv) {
    return texture(frame, uv);
}

float rand(vec2 seed) {
    return fract(sin(dot(seed, vec2(12.9898, 78.233))) * 43758.5453);
}

vec3 lerp(vec3 a, vec3 b, vec3 t) {
    return a + t * (b - a);
}

const int BLUR_SIZE = 12;

void main() {

    int mid = BLUR_SIZE / 2;
    vec4 color = vec4(0);
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
            float idist = clamp(1 - dist, 0, 1);

            vec2 new_uv = uv + pixelSize * diff;
            color += sample(new_uv) * idist * rand_mod * 0.2;
        }
    }
    color /= BLUR_SIZE;

    color = max(color, color + sample(uv));

    float intensity = length(color.rgb);
    color.rgb = lerp(BG_COLOR, FG_COLOR, vec3(intensity));

    float scanline = 1 - fract(mod(time, 5) / 5 + uv.y);
    float scanline_start = 0.5;
    if (scanline < scanline_start) {
        scanline = 0;
    } else {
        scanline = -cos(1.57079 * ((scanline - scanline_start) / (1 - scanline_start))) + 1;
    }

    gl_FragColor = vec4(color.rgb * (1 + scanline * 0.6) * rand_mod, 1);
}