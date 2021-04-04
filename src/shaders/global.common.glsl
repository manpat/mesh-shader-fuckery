

layout(std140, binding = 0) uniform UniformData {
	layout(row_major) mat4 u_projection_view;
	vec4 u_camera_up;
	vec4 u_camera_right;
	vec2 u_world_size;
};

