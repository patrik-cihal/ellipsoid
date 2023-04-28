// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) index: u32,
    @location(2) tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) index: u32,
    @location(1) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0., 1.0);
    out.tex_coord = model.tex_coord;
    out.index = model.index;
    return out;
}

@group(0) @binding(0)
var texture_array: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var text_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    color = textureSample(
        texture_array[in.index],
        text_sampler,
        in.tex_coord,
    );
    return color;
}