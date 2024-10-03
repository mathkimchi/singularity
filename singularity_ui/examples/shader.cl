// assume it starts at (0, 0)
__kernel void draw_rectangle(
            __private uint4 const color,
            uint const width,
            uint const height,
            // read_only image2d_t src_image,
            write_only image2d_t dst_image)
{
    int2 coord = (int2)(get_global_id(0), get_global_id(1));

    if ((coord.x < width) && (coord.y < height)) {
        write_imageui(dst_image, coord, color);
    }
}
// __kernel void draw_triangle(
//             __private float const coeff,
//             read_only image2d_t src_image,
//             write_only image2d_t dst_image)
// {
//     int2 coord = (int2)(get_global_id(0), get_global_id(1));

//     float4 pixel = read_imagef(src_image, sampler_host, coord);

//     pixel += (float4)(0.0, 0.0, 0.5, 0.0);

//     write_imagef(dst_image, coord, pixel);
// }