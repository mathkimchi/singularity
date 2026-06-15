// From:
// https://github.com/sotrh/learn-wgpu/blob/master/code/beginner/tutorial3-pipeline/src/shader.wgsl

// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    // @location(1) color: vec3<f32>,
};
/// RoundRectInstance:
/// origin: [f32; 2],
/// size: [f32; 2],
/// corner_radius: f32,
/// border_dist: f32,
/// main_color: [f32; 4],
/// border_color: [f32; 4],
struct InstanceInput {
    // Should be top left
    @location(1) origin: vec2<f32>,
    @location(2) size: vec2<f32>,
    // NOTE: I'm ignoring this for now
    @location(3) corner_radius: f32,
    @location(4) border_dist: f32,
    @location(5) main_color: vec4<f32>,
    @location(6) border_color: vec4<f32>,
}

// is also the input of the fragment shader
struct VertexOutput {
    // the x and y of the builtin(position) are in pixel space
    @builtin(position) clip_position: vec4<f32>,
    @location(0) main_color: vec4<f32>,
    @location(1) origin: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) corner_radius: f32,
    @location(4) border_dist: f32,
    @location(5) border_color: vec4<f32>,
};

// @vertex
// fn vs_main(
//     model: VertexInput,
//     @builtin(vertex_index) index: u32,
// ) -> VertexOutput {
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    // out.color = model.color;
    out.main_color = instance.main_color;
    out.origin = instance.origin;
    out.size = instance.size;
    out.corner_radius = instance.corner_radius;
    out.border_dist = instance.border_dist;
    out.border_color = instance.border_color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.origin.x + in.border_dist <= in.clip_position.x) && (in.clip_position.x <= in.origin.x + in.size.x - in.border_dist) && (in.origin.y + in.border_dist <= in.clip_position.y) && (in.clip_position.y <= in.origin.y + in.size.y - in.border_dist) {
        return in.main_color;
    } else if (in.origin.x <= in.clip_position.x) && (in.clip_position.x <= in.origin.x + in.size.x) && (in.origin.y <= in.clip_position.y) && (in.clip_position.y <= in.origin.y + in.size.y) {
        return in.border_color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
