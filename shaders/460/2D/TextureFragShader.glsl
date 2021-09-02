#version 460

uniform sampler2D texture2d;

in vec2 vertex_texture_coords;

out vec4 fragment_color;

void main()
{
	vec4 tex_color = texture(texture2d, vertex_texture_coords);

	if(tex_color.a < 0.1) discard;

	fragment_color = tex_color;
}