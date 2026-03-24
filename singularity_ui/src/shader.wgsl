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
    @location(1) origin: vec2<f32>,
    @location(2) size: vec2<f32>,
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

/// Logic from [Zed's blog](https://zed.dev/blog/videogame)
/// ported from Metal to WGSL.
fn rect_sdf(
    absolute_pixel_position: vec2<f32>,
    origin: vec2<f32>,
    size: vec2<f32>,
    corner_radius: f32,
) -> f32 {
    let half_size = size / 2.;
    let rect_center = origin + half_size;
 
    // Change coordinate space so that the rectangle's center is at the origin,
    // taking advantage of the problem's symmetry.
    let pixel_position = abs(absolute_pixel_position - rect_center);
 
    // Shrink rectangle by the corner radius.
    let shrunk_corner_position = half_size - corner_radius;
 
    // Determine the distance vector from the pixel to the rectangle corner,
    // disallowing negative components to simplify the three cases.
    let pixel_to_shrunk_corner = max(vec2<f32>(0., 0.), pixel_position - shrunk_corner_position);

    let distance_to_shrunk_corner = length(pixel_to_shrunk_corner);
 
    // Subtract the corner radius from the calculated distance to produce a
    // rectangle having the desired size.
    let distance = distance_to_shrunk_corner - corner_radius;

    return distance;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = rect_sdf(
        in.clip_position.xy,
        in.origin,
        in.size,
        in.corner_radius,
    );

    if distance < 0.0 {
        return in.main_color;
    } else if distance < in.border_dist {
        return in.border_color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
