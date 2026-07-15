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
    // character width and height; technically I can just say `num char / width = height` but as they say, KISS
    // u16 would work, but wgsl doesn't have it. I could use one u32 then split it later but again, KISS
    @location(3) grid_size: vec2<u32>,
}

// is also the input of the fragment shader
struct VertexOutput {
    // builtin(position) is in clip space (-1 to 1) when outputted by vertex shader,
    // then the rasterizer transforms it to pixel space and gives it to frag shader
    @builtin(position) clip_position: vec4<f32>,
    // // the uv coordinates (in terms of the texture)
    // // goes from Top left (0, 0) to bottom right (1, 1)
    // @location(0) tex_coords: vec2<f32>,
    // top left corner
    @location(0) origin: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) grid_size: vec2<u32>,
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
    // out.tex_coords = vec2<f32>(
    //     remap_range(model.position.x, instance.origin.x, instance.origin.x + instance.size.x, 0., 1.),
    //     remap_range(model.position.y, instance.origin.y, instance.origin.y + instance.size.y, 0., 1.),
    // );
    out.origin = instance.origin;
    out.size = instance.size;
    out.grid_size = instance.grid_size;
    return out;
}

// Fragment shader

// I think each group changes together, and groups that change least should get lowest number
@group(0) @binding(0)
var sdf_atlas: texture_2d_array<f32>;
@group(0) @binding(1)
var sdf_sampler: sampler;
// Yeah, this is kinda jank, but I guess it makes it quirky *mews shyly*
@group(1) @binding(0)
var characters: texture_storage_2d<rgba32uint, read>;

// Pixel Range used when generrating distance field
const PX_RANGE: f32 = 4.0;

fn median(v: vec3<f32>) -> f32 {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 0 to 1 placing this pixel relative to the whole char grid
    let tex_coords = vec2<f32>(
        remap_range(in.clip_position.x, in.origin.x, in.origin.x + in.size.x, 0., 1.),
        remap_range(in.clip_position.y, in.origin.y, in.origin.y + in.size.y, 0., 1.),
    );

    // There's gotta be a vector-magic way to check all at once; ig it could be done by seeing if clamping changes it
    // NOTE: I don't want to deal with the literal edge case of tex coord.x or y = 1
    // TODO: ^^^--- deal with edge case later (probably important for bg or full-char fg)
    if (0. <= tex_coords.x && tex_coords.x < 1.) &&
        (0. <= tex_coords.y && tex_coords.y < 1.) {
        // TODO: find tex_coords relative to character
        // integer index of character
        let char_idx = vec2<u32>(tex_coords * vec2<f32>(in.grid_size));
        // where in the glyph this pixel is, from 0 to 1
        let glyph_pos_uv = (tex_coords * vec2<f32>(in.grid_size)) % 1.;
        // // row * width + col
        // let char_idx = char_coord.y * grid_size.x + char_coord.x;
        // // 0 is mipLevel, which lets you render more roughly at higher levels
        // let char = textureLoad(characters, char_coord, 0);
        // ^-- never mind, https://www.w3.org/TR/WGSL/#textureload for texture_storage_2d, you don't specify mip level.
        let char = textureLoad(characters, char_idx);
        let char_type= char.x;
        let fg = unpack4x8unorm(char.y);
        let bg = unpack4x8unorm(char.z);
        let style = char.w;

        // // just make sure characters change
        // return vec4<f32>((f32(char_type) * 1.618033 * 1000000.) % 1., 0., 0., 1.);

        if char_type == 32 {
            // space
            return bg;
        } else if char_type < 32 || char_type > 126 {
            // Non-printable character or beyond ascii, just draw red box
            // https://www.ascii-code.com/
            return vec4<f32>(0.5, 0.0, 0.0, 0.5);
        } else {
            let msdf_values = textureSample(sdf_atlas, sdf_sampler, glyph_pos_uv, char_type - 33);
            // The gpu maps this to [0, 1]
            // if median(msdf_values.xyz) < 0.5 {
            //     // idk bro, msdfgen crate inverts the sign for some reason
            //     return bg;
            // } else {
            //     // yeah, I'm just ignoring the border (==0.5) case
            //     return fg;
            // }
            // The gpu maps all distances to [0, 1] since these operations are meant for rgb
            // I think the 0 and 1 bounds mean that it is PX_RANGE far from boundary
            // I think PX_RANGE was in terms of the texture grid (like the 32x64 or 64x64 grid the dists were stored inside), not the actual pixels
            let signed_dist = median(msdf_values.xyz) - 0.5;
            // now in terms of normalized texture units, where unit length 1 is the dist of 1 char
            // (it's impossible to know the actual distance bc we got stretched dist, so I'll assume it was diagonal)
            // NOTE: This math doesn't actually come from logic, I am pretty sure it doesn't actually represent what I want it to.
            // Read 2026-07-15 devlog for more info.
            let screen_uv_offset = PX_RANGE / (vec2<f32>(32.0, 64.0) * 0.70710678118);
            let screen_px_offset = screen_uv_offset * vec2<f32>(textureDimensions(sdf_atlas));
            let screen_px_dist = 0.1 * signed_dist * length(screen_px_offset);
            let opacity = clamp(screen_px_dist + 0.5, 0.0, 1.0);
            return mix(bg, fg, opacity);
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
