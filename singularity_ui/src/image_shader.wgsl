// From:
// https://github.com/sotrh/learn-wgpu/blob/master/code/beginner/tutorial3-pipeline/src/shader.wgsl

// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
};

// Just assumes the image is at the corners
struct InstanceInput {
    // top left corner
    @location(1) origin: vec2<f32>,
    @location(2) size: vec2<f32>,
}

// is also the input of the fragment shader
struct VertexOutput {
    // the x and y of the builtin(position) are in pixel space
    @builtin(position) clip_position: vec4<f32>,
    // the uv coordinates (in terms of the texture)
    // goes from Top left (0, 0) to bottom right (1, 1)
    @location(0) tex_coords: vec2<f32>,
};

fn remap_range(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return out_min + (x - in_min) / (in_max - in_min) * (out_max - out_min);
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = vec2<f32>(
        remap_range(model.position.x, instance.origin.x, instance.origin.x + instance.size.x, 0., 1.),
        remap_range(model.position.y, instance.origin.y, instance.origin.y + instance.size.y, 0., 1.),
    );
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // There's gotta be a way to check all at once
    if (0. < in.tex_coords.x && in.tex_coords.x < 1.) &&
        (0. < in.tex_coords.y && in.tex_coords.y < 1.) {
        return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
