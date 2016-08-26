#version 150
in vec3 position;
in vec3 normal;
in vec3 color;

out vec3 v_normal;
out vec3 v_position;
out vec3 v_color;

uniform mat4 perspective;
uniform mat4 model_view;

void main() {
	vec4 world_position = model_view * vec4(position, 1.0);

	v_position = vec3(world_position) / world_position.w;
	v_color = color;
	gl_Position = perspective * world_position;

	v_normal = transpose(inverse(mat3(model_view))) * normal;
}
