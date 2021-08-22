#version 460

uniform vec4 translation;
uniform mat4 rotation;

in vec3 position;
in vec3 color;

out vec3 vertex_color;

void main()
{
	gl_Position  = vec4(position, 1.0) * rotation + translation;
	vertex_color = vec3(0.0, 0.0, 0.0);
}