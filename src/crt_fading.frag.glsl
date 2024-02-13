#version 330 core

uniform sampler2D frame;
// uniform float time;
// uniform vec2 pixelSize;
in vec2 uv;

vec3 lerp(vec3 a, vec3 b, vec3 t) {
    return a + t * (b - a);
}

layout (location = 0) out vec4 color;

// float rand(vec2 seed) {
//     float m = mod(time, 123.0);

// 	// https://stackoverflow.com/a/10625698/13645088
//     vec2 K1 = vec2(
//         23.14069263277926, // e^pi (Gelfond's constant)
//          2.665144142690225 // 2^sqrt(2) (Gelfond Schneider constant)
//     );
//     return fract(cos(dot(seed, K1) + m * 0.01) * 123.456);
// }

vec4 sample(vec2 uv) {
    return texture(frame,uv);// uv + (rand(uv) - 0.5) * 0.1);// * rand(uv * 21.66 + 1.35);
}

void main() {

    float fade = 0.1;

    vec3 bg = sample(uv).rgb * 0.95 - 0.05;

    // vec2 pixel = pixelSize * 1;

    // vec3 blur = vec3(0.0);
    // blur += sample(uv + vec2(0, -pixel.y)).rgb;
    // blur += sample(uv + vec2(0, pixel.y) ).rgb;
    // blur += sample(uv + vec2(pixel.x, 0) ).rgb;
    // blur += sample(uv + vec2(-pixel.x, 0)).rgb;
    // blur /= 4;

    // blur = blur * 0.07;// + rand(uv) * 0.05;

    bg = clamp(bg, 0.0, 1.0);
    gl_FragColor = vec4(bg,1.0);
    // gl_FragColor = vec4(0);
    // gl_FragColor = sample(uv);
    //color = vec4(0.1, 0.4, 0.9, 1.0);
}
