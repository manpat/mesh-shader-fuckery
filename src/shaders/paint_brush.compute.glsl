#version 450

#import paint

layout(local_size_x=8, local_size_y=8, local_size_z=1) in;


layout(binding = 0, r32f) uniform image2D u_image;

void main() {
	const ivec2 image_size = imageSize(u_image);

	const vec2 world_bl = -u_paint.world_size / 2.0;
	const vec2 brush_center_normalised = (u_brush.center - world_bl) / u_paint.world_size;
	const vec2 brush_extent_normalised = u_brush.size / u_paint.world_size / 2.0;

	vec2 brush_min_image_f = floor((brush_center_normalised - brush_extent_normalised) * vec2(image_size));
	vec2 brush_max_image_f = ceil((brush_center_normalised + brush_extent_normalised) * vec2(image_size));

	ivec2 brush_min_image = max(ivec2(0), ivec2(brush_min_image_f));
	ivec2 brush_max_image = min(image_size, ivec2(brush_max_image_f));

	ivec2 sample_pos = ivec2(gl_GlobalInvocationID.xy) + brush_min_image;

	if (all(lessThan(sample_pos, brush_max_image))) {
		sample_pos = clamp(sample_pos, ivec2(0), image_size);

		float dist = length(vec2(0.5) - vec2(gl_GlobalInvocationID.xy) / (brush_max_image_f - brush_min_image_f)) / 0.5;

		if (dist < 1.0) {
			float interpolant = (1.0 - dist);
			interpolant = interpolant * interpolant;
			interpolant = interpolant * interpolant;

			float value = imageLoad(u_image, sample_pos).r;
			// float target_value = min(value + interpolant * 0.1, 1.0);
			float target_value = value + interpolant * 0.3;

			value = mix(value, target_value, interpolant);

			imageStore(u_image, sample_pos, vec4(value));
		}
	}
}