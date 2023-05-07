// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_index: u32,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coord = model.tex_coord;
    out.tex_index = model.tex_index;
    out.color = model.color;
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
        texture_array[in.tex_index],
        text_sampler,
        in.tex_coord,
    ) * in.color;
    return color;
}