//! Simple polygon rendering.
//!

use std::borrow::Cow;

use crate::RenderContext;

pub(crate) struct WithColor {
    render_pipeline: wgpu::RenderPipeline,
    /// The hexagon vertex buffer.
    shapes_vertex_buffer: wgpu::Buffer,
    /// The hexagon index buffer.
    hexagon_index_buffer: wgpu::Buffer,
    /// The shape instance buffer.
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl WithColor {
    pub(crate) fn new(context: &RenderContext) -> Self {
        //
        // Pipeline setup
        //

        // Compile the shaders from source.
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Polygon Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        // Define the pipeline layout.
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Polygon Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        // Create the render pipeline.
        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Polygon Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[PolygonVertex::desc(), PolygonInstance::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(context.swapchain_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        //
        // Shape setup
        //

        let shapes_vertex_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Shapes Vertex Buffer"),
                    contents: bytemuck::cast_slice(HEXAGON_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let hexagon_index_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Hexagon Index Buffer"),
                    contents: bytemuck::cast_slice(HEXAGON_INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        const INSTANCE_PER_SIDE: usize = 4;
        const INSTANCE_DISPLACEMENT: f32 = 0.1;
        const SCALE: f32 = 0.09;
        // A square of shape instances.
        let instance_data = (0..INSTANCE_PER_SIDE)
            .flat_map(|x| {
                (0..INSTANCE_PER_SIDE).map(move |y| PolygonInstance {
                    transform: [
                        [SCALE, 0.0, 0.0, 0.0],
                        [0.0, SCALE, 0.0, 0.0],
                        [0.0, 0.0, SCALE, 0.0],
                        [
                            x as f32 * INSTANCE_DISPLACEMENT,
                            y as f32 * INSTANCE_DISPLACEMENT,
                            0.0,
                            1.0,
                        ],
                    ],
                })
            })
            .collect::<Vec<_>>();
        let instance_count = instance_data.len() as u32;

        let instance_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Shape Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    // The buffer will be used as a vertex buffer and is updatable.
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            render_pipeline,
            shapes_vertex_buffer,
            hexagon_index_buffer,
            instance_buffer,
            instance_count,
        }
    }

    pub(crate) fn render<'a, 'b, 'c: 'a>(
        &'a self,
        pass: &'b mut wgpu::RenderPass<'a>,
        context: &'c RenderContext,
    ) {
        pass.set_pipeline(&self.render_pipeline);

        // Set normal vertex buffer.
        pass.set_vertex_buffer(0, self.shapes_vertex_buffer.slice(..));
        // Set per-instance vertex buffer.
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // Set index buffer.
        pass.set_index_buffer(
            self.hexagon_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        // Draw the hexagon.
        pass.draw_indexed(0..HEXAGON_INDICES.len() as u32, 0, 0..self.instance_count);
    }
}

/// The vertex for the triangle shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PolygonVertex {
    position: [f32; 3],
    color: [f32; 3],
}
impl PolygonVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTR: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PolygonVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTR,
        }
    }
}

/// GPU representation of a polygon instance.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PolygonInstance {
    /// Transformation matrix.
    transform: [[f32; 4]; 4],
}
impl PolygonInstance {
    /// Returns the buffer descriptor for the shape instance buffer.
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTR: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PolygonInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTR,
        }
    }
}

use wgpu::util::DeviceExt;

/// The radius of the hexagon.
const HEXAGON_RADIUS: f32 = 0.5;

const SQRT_3: f32 = 1.73205080757;

/// The vertices of the hexagon.
///
/// The vertices are in counter-clockwise order.
/// The radius of the hexagon is given by `HEXAGON_RADIUS`.
/// The center of the hexagon is at the origin.
const HEXAGON_VERTICES: &[PolygonVertex] = &[
    // Right vertex
    PolygonVertex {
        position: [HEXAGON_RADIUS, 0.0, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    // Bottom right vertex
    PolygonVertex {
        position: [HEXAGON_RADIUS / 2.0, HEXAGON_RADIUS * SQRT_3 / 2.0, 0.0],
        color: [0.667, 0.667, -0.333],
    },
    // Bottom left vertex
    PolygonVertex {
        position: [-HEXAGON_RADIUS / 2.0, HEXAGON_RADIUS * SQRT_3 / 2.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    // Left vertex
    PolygonVertex {
        position: [-HEXAGON_RADIUS, 0.0, 0.0],
        color: [-0.333, 0.667, 0.667],
    },
    // Top left vertex
    PolygonVertex {
        position: [-HEXAGON_RADIUS / 2.0, -HEXAGON_RADIUS * SQRT_3 / 2.0, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    // Top right vertex
    PolygonVertex {
        position: [HEXAGON_RADIUS / 2.0, -HEXAGON_RADIUS * SQRT_3 / 2.0, 0.0],
        color: [0.667, -0.333, 0.667],
    },
];

/// The indices of a hexagon.
///
/// The indices are in counter-clockwise order.
const HEXAGON_INDICES: &[u16] = &[
    0, 1, 2, // First triangle
    0, 2, 3, // Second triangle
    0, 3, 4, // Third triangle
    0, 4, 5, // Fourth triangle
];
