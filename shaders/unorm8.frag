#version 430
// vi: ft=glsl

layout(location = 0) in vec2 texture_coords;
layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform InfoBlock {
	uint format;
	uint width;
	uint height;
	uint stride_x;
	uint stride_y;
};

layout(set = 1, binding = 1) buffer readonly Data {
	uint data[];
};

uint extract_u8(uint i) {
	uint word = data[i / 4];
	uint offset = (i % 4) * 8;
	return word >> offset & 0xFF;
}

float extract_unorm8(uint i) {
	return float(extract_u8(i)) / 255.0;
}

vec4 get_pixel(uint x, uint y) {
	uint i = x * stride_x + y * stride_y;

	// Mono8
	if (format == 0) {
		float mono = extract_unorm8(i);
		return vec4(mono, mono, mono, 1.0);

	// MonoAlpha8(Unpremultiplied)
	} else if (format == 1) {
		float mono = extract_unorm8(i);
		float a    = extract_unorm8(i + 1);
		return vec4(mono, mono, mono, a);

	// MonoAlpha8(Premultiplied)
	} else if (format == 2) {
		float a    = float(extract_u8(i + 1));
		float mono = float(extract_u8(i)) / a;
		return vec4(mono, mono, mono, a);

	// Bgr8
	} else if (format == 3) {
		float b = extract_unorm8(i + 0);
		float g = extract_unorm8(i + 1);
		float r = extract_unorm8(i + 2);
		return vec4(r, g, b, 1.0);

	// Bgra8(Unpremultiplied)
	} else if (format == 4) {
		float b = extract_unorm8(i + 0);
		float g = extract_unorm8(i + 1);
		float r = extract_unorm8(i + 2);
		float a = extract_unorm8(i + 3);
		return vec4(r, g, b, a);

	// Bgra8(Premultiplied)
	} else if (format == 5) {
		float a = float(extract_u8(i + 3));
		float b = float(extract_u8(i + 0)) / a;
		float g = float(extract_u8(i + 1)) / a;
		float r = float(extract_u8(i + 2)) / a;
		return vec4(r, g, b, a / 255.0);

	// Rgb8
	} else if (format == 6) {
		float r = extract_unorm8(i + 0);
		float g = extract_unorm8(i + 1);
		float b = extract_unorm8(i + 2);
		return vec4(r, g, b, 1.0);

	// Rgba8(Unpremultiplied)
	} else if (format == 7) {
		float r = extract_unorm8(i + 0);
		float g = extract_unorm8(i + 1);
		float b = extract_unorm8(i + 2);
		float a = extract_unorm8(i + 3);
		return vec4(r, g, b, a);

	// Rgba8(Premultiplied)
	} else if (format == 8) {
		float a = float(extract_u8(i + 3));
		float r = float(extract_u8(i + 0)) / a;
		float g = float(extract_u8(i + 1)) / a;
		float b = float(extract_u8(i + 2)) / a;
		return vec4(r, g, b, a / 255.0);

	} else {
		return vec4(1.0, 0.0, 1.0, 1.0);
	}
}

void main() {
	if (texture_coords.x < 0.0 || texture_coords.x >= float(width) ||
		texture_coords.y < 0.0 || texture_coords.y >= float(height)) {
		out_color = vec4(0.0, 0.0, 0.0, 0.0);
		return;
	}

	// Offset so pixel centers are at integers
	float fx = texture_coords.x - 0.5;
	float fy = texture_coords.y - 0.5;

	float floor_x = floor(fx);
	float floor_y = floor(fy);
	float frac_x = fx - floor_x;
	float frac_y = fy - floor_y;

	// Clamp to valid pixel range
	uint x0 = uint(clamp(floor_x, 0.0, float(width - 1)));
	uint y0 = uint(clamp(floor_y, 0.0, float(height - 1)));
	uint x1 = uint(clamp(floor_x + 1.0, 0.0, float(width - 1)));
	uint y1 = uint(clamp(floor_y + 1.0, 0.0, float(height - 1)));

	// Bilinear interpolation
	vec4 p00 = get_pixel(x0, y0);
	vec4 p10 = get_pixel(x1, y0);
	vec4 p01 = get_pixel(x0, y1);
	vec4 p11 = get_pixel(x1, y1);

	out_color = mix(mix(p00, p10, frac_x), mix(p01, p11, frac_x), frac_y);
}
