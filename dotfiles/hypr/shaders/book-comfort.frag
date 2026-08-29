// Book-like comfort preset: restrained sharpening, warm paper whites,
// softened highlights, reduced saturation, and subtle static grain.
#version 300 es

precision highp float;
precision highp int;

in vec2 v_texcoord;
layout(location = 0) out vec4 fragColor;

uniform sampler2D tex;

const float TILE_SIZE = 256.0;
const float NOISE_BRIGHTNESS = 0.4;
const float NOISE_SPREAD = 0.35;
const float GRAIN_STRENGTH = 0.07;
const float SHARPEN_STRENGTH = 0.34;
const float SATURATION = 0.88;

float luminance(vec3 color) {
	return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

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
	vec2 texel = 1.0 / vec2(textureSize(tex, 0));

	// A limited five-tap unsharp mask restores a little edge definition after
	// fractional output scaling without creating conspicuous text halos.
	vec3 neighbors = (
		texture(tex, v_texcoord + vec2(texel.x, 0.0)).rgb +
		texture(tex, v_texcoord - vec2(texel.x, 0.0)).rgb +
		texture(tex, v_texcoord + vec2(0.0, texel.y)).rgb +
		texture(tex, v_texcoord - vec2(0.0, texel.y)).rgb
	) * 0.25;
	vec3 detail = clamp(screen.rgb - neighbors, vec3(-0.05), vec3(0.05));
	vec3 color = clamp(screen.rgb + detail * SHARPEN_STRENGTH, 0.0, 1.0);

	// Calm intense colors while retaining luminance contrast.
	float luma = luminance(color);
	color = mix(vec3(luma), color, SATURATION);

	// Roll bright backgrounds gently toward a warm, non-glaring paper white.
	float highlight = smoothstep(0.52, 1.0, luma);
	color *= mix(1.0, 0.90, highlight);
	color *= mix(vec3(1.0), vec3(1.0, 0.965, 0.86), highlight);

	// Use a lighter grain blend than the dedicated Paper mode so small text
	// keeps more of its original foreground/background contrast.
	uvec2 tilePixel = uvec2(mod(floor(gl_FragCoord.xy), TILE_SIZE));
	uint seed = tilePixel.x + tilePixel.y * uint(TILE_SIZE);
	float randomValue = float(hashPixel(seed) & 0x00ffffffu) / 16777215.0;
	float shade = clamp(
		NOISE_BRIGHTNESS + (randomValue - 0.5) * NOISE_SPREAD,
		0.0,
		1.0
	);
	color = mix(color, vec3(shade), GRAIN_STRENGTH);

	fragColor = vec4(color, screen.a);
}
