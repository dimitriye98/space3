#version 150
in vec3 position;
in vec3 normal;
in vec3 color;

out Vertex
{
	vec3 normal;
	vec3 color;
} vertex;

void main()
{
	gl_Position = vec4(position, 1.0);
	vertex.normal = normal;
	vertex.color = vec3(1.0, 1.0, 0.0);
}
