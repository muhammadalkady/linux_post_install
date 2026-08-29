// PaperShell-style static grayscale grain for Hyprland.
#version 300 es

precision highp float;
precision highp int;

in vec2 v_texcoord;
layout(location = 0) out vec4 fragColor;

uniform sampler2D tex;

const float TILE_SIZE = 256.0;
const float NOISE_BRIGHTNESS = 0.4;
const float NOISE_SPREAD = 0.35;
const float BLEND_STRENGTH = 0.15;

uint hashPixel(uint value) {
    value = (value ^ 61u) ^ (value >> 16u);
    value *= 9u;
    value ^= value >> 4u;
    value *= 0x27d4eb2du;
    value ^= value >> 15u;
    return value;
}

void main() {
    vec4 screen = texture(tex, v_texcoord);

    // Repeat the same deterministic noise tile instead of animating it. This
    // keeps Hyprland's damage tracking effective and mirrors PaperShell's
    // fixed Cairo tile.
    uvec2 tilePixel = uvec2(mod(floor(gl_FragCoord.xy), TILE_SIZE));
    uint seed = tilePixel.x + tilePixel.y * uint(TILE_SIZE);
    float randomValue = float(hashPixel(seed) & 0x00ffffffu) / 16777215.0;
    float shade = clamp(
        NOISE_BRIGHTNESS + (randomValue - 0.5) * NOISE_SPREAD,
        0.0,
        1.0
    );

    fragColor = vec4(
        mix(screen.rgb, vec3(shade), BLEND_STRENGTH),
        screen.a
    );
}
