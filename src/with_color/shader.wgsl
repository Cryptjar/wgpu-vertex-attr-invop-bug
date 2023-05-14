
/// Vertex input data
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};
/// Instance (vertex) input data
struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
};


/// Output of the vertex shader and input of the fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) vertex_color: vec4<f32>,
};

fn rgb_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
    let shifted_color = (rgb + 0.055) / 1.055;
    let power_color = pow(shifted_color, vec3<f32>(2.4));
    return power_color;
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput,) -> VertexOutput {
	let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var output: VertexOutput;

    let wpos = (model_matrix * vec4<f32>(vertex.position, 1.0)).xyz;
    let clip_pos = vec4<f32>(wpos, 1.0);

    output.clip_position = clip_pos;
    output.world_position = wpos;

    let color = vertex.color;
    output.vertex_color = vec4<f32>(color, 1.0);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(rgb_to_srgb(input.vertex_color.rgb), 1.0);
}
