#version 420
// vi: ft=glsl

layout(location = 0) in vec2 texture_coords;
layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D image_texture;
layout(set = 1, binding = 1) uniform sampler image_sampler;

void main() {
	out_color = texture(sampler2D(image_texture, image_sampler), texture_coords);
}
