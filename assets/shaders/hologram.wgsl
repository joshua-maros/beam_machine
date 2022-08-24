#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var regular_blocks: texture_2d<f32>;
@group(1) @binding(1)
var regular_blocks_sampler: sampler;

@group(1) @binding(2)
var holo_blocks: texture_2d<f32>;
@group(1) @binding(3)
var holo_blocks_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // Get screen position with coordinates from 0 to 1
    let uv = position.xy / vec2<f32>(view.width, view.height);
    let strength = select(0.5, 0.3, uv.y % 0.01 < 0.005);

    var output_color = textureSample(regular_blocks, regular_blocks_sampler, uv)
        + strength * textureSample(holo_blocks, holo_blocks_sampler, uv);

    return output_color;
}
