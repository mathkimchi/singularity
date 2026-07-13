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

fn median(v: vec3<f32>) -> f32 {
    return clamp(v.x, min(v.y, v.z), max(v.y, v.z));
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
            if median(msdf_values.xyz) < 0.5 {
                // idk bro, msdfgen crate inverts the sign for some reason
                return bg;
            } else {
                // yeah, I'm just ignoring the border (==0.5) case
                return fg;
            }
        }
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
