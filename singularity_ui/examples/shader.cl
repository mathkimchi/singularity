__kernel void draw_circle(
            __private uint4 const color,
            __private float2 const center,
            __private float const radius,
            // read_only image2d_t src_image,
            write_only image2d_t dst_image)
{
    int2 coord = (int2)(get_global_id(0), get_global_id(1));

    if (distance((float2) ((float) coord.x, (float) coord.y), center)<=radius) {
        write_imageui(dst_image, coord, color);
    }
}
// assume it starts at (0, 0)
__kernel void draw_rectangle(
            __private uint4 const color,
            __private uint const width,
            __private uint const height,
            // read_only image2d_t src_image,
            write_only image2d_t dst_image)
{
    int2 coord = (int2)(get_global_id(0), get_global_id(1));

    if ((coord.x < width) && (coord.y < height)) {
        write_imageui(dst_image, coord, color);
    }
}
