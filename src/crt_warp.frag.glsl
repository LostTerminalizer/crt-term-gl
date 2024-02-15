#version 330 core

vec2 warp(vec2 uv) {

    float warp_amount = 0.3;

    vec2 delta = uv - 0.5;
    float delta2 = dot(delta.xy, delta.xy);
    float delta4 = delta2 * delta2;
    float delta_offset = delta4 * warp_amount;

    return uv + delta * delta_offset;
}

float map(float is, float ie, float os, float oe, float v) {
    float t = (v - is) / (ie - is);
    return os + t * (oe - os); 
}

in vec2 uv;
uniform sampler2D tex;


void main() {
    vec3 col;
    vec2 uv = warp(uv);

    
    vec2 dist_v = max(0.0 - uv, uv - 1.0);

    if(uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        float dist = max(dist_v.x, dist_v.y);

        col = vec3(dist * 6.0);
    } else {
        float border_shade = clamp(abs(dist_v.x * dist_v.y * 10), 0.0, 1.0);

        col = texture(tex, uv).rgb;
        float light = max(col.r, max(col.g, col.b));
        light = pow(light, 2);

        border_shade = 1 - border_shade;
        border_shade = border_shade * ((1 - light) * 0.002 + 0.998);
        border_shade = 1 - pow(border_shade, 300);

        col = col * border_shade;

        float dist = dist_v.x * dist_v.y;
        float warp_shade = clamp(pow(abs(dist), 0.12), 0.0, 1.0);
        warp_shade = 1 - warp_shade;
        light = pow(light, 2);
        warp_shade = warp_shade * (1 - light);
        warp_shade = 1 - warp_shade;
        col = col * warp_shade;

        // col = vec3(0);
        // col.r = border_shade;
    }

    gl_FragColor = vec4(col, 1.0);
}