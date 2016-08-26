#version 140

in vec3 v_normal;
in vec3 v_position;
in vec3 v_color;

out vec4 color;

uniform vec3 u_light;
uniform mat4 model_view;

const float ambient_coefficient = 0.33;
const vec3 specular_color = vec3(0.7, 0.7, 0.7);

void main() {
	vec3 light_dir = normalize(u_light - v_position);
	float diffuse = max(dot(normalize(v_normal), light_dir), 0.0);

	float specular = 0.0;

	if (diffuse > 0.0) {
		vec3 camera_dir = normalize(-v_position);
		vec3 half_direction = normalize(light_dir + camera_dir);
		specular = diffuse * pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
	}

	color = vec4(ambient_coefficient * v_color + diffuse * v_color + specular * specular_color, 1.0);
}
