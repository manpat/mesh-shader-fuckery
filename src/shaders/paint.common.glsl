
layout(std140, binding = 1) uniform PaintData {
	vec2 world_size;
} u_paint;


layout(std140, binding = 2) uniform BrushData {
	vec2 center;
	vec2 size;
} u_brush;



