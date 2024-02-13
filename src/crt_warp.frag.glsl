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
uniform float time;
uniform float scanlineSize;

uniform float weight[7] = float[](0.1, 0.2, 0.6, 0.8, 0.6, 0.2, 0.1);

const int BLOOM_START = -4;
const int BLOOM_END = 4;

void main() {
    vec3 col;
    vec2 uv = warp(uv);

    vec2 dist_v = max(0.0 - uv, uv - 1.0);

    if(uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        float dist = max(dist_v.x, dist_v.y);

        col = vec3(dist * 6.0);
    } else {
        // float fw = 0.0015;

        // vec3 c = vec3(0.0);
        // for(int x = BLOOM_START; x <= BLOOM_END; x++) {
        //     for(int y = BLOOM_START; y <= BLOOM_END; y++) {
        //         float yo = fw * y;
        //         float xo = fw * x;
        //         c += texture(tex, uv + vec2(xo, yo)).rgb * weight[x - BLOOM_START] * weight[y - BLOOM_START];
        //     }
        // }
        // const int BLOOM_SIZE = BLOOM_END - BLOOM_START;
        // c /= BLOOM_SIZE;
        // c += texture(tex, uv).rgb;
        col = texture(tex, uv).rgb;

        // float intensity = length(c);//max(c.r, max(c.g, c.b));
        // float scanlineIntensity = intensity;

        //float scanlineDistance = mod(uv.y, scanlineSize) * scanlineSize;

        // float closestScanlineEdge = round(uv.y / scanlineSize) * scanlineSize;
        // float scanlineDistance = clamp(abs(closestScanlineEdge - uv.y) / (scanlineSize * 0.5), 0.0, 1.0);
        // float scanline = clamp(scanlineIntensity * scanlineDistance, 0.0, 1.0);

        float dist = dist_v.x * dist_v.y;
        //col = lerp(col, vec3(0), vec3(scanline));
        col = col * ( vec3(clamp(pow(abs(dist), 0.1), 0.0, 1.0)));
    }

    gl_FragColor = vec4(col, 1.0);
}