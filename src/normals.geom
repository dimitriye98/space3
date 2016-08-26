#version 150
layout(triangles) in;

// Three lines will be generated: 6 vertices
layout(line_strip, max_vertices=6) out;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

in Vertex
{
	vec3 normal;
	vec3 color;
} vertex[];

out vec4 v_color;

const float normal_length = 0.5;

void main()
{
	mat4 modelviewperspective = perspective * view * model;
	int i;
	for(i=0; i < gl_in.length(); i++)
	{
		vec3 P = gl_in[i].gl_Position.xyz;
		vec3 N = vertex[i].normal;

		gl_Position = modelviewperspective * vec4(P, 1.0);
		v_color = vec4(vertex[i].color, 1.0);
		EmitVertex();

		gl_Position = modelviewperspective * vec4(P + N * normal_length, 1.0);
		v_color = vec4(vertex[i].color, 1.0);
		EmitVertex();

		EndPrimitive();
	}
}
