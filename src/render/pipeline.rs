use bevy::{
    prelude::{Component, FromWorld, Resource, World},
    render::{
        globals::GlobalsUniform,
        mesh::PrimitiveTopology,
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
            ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState, PolygonMode,
            PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            SpecializedRenderPipeline, TextureFormat, TextureSampleType, TextureViewDimension,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::{ViewTarget, ViewUniform},
    },
};

use super::BILLBOARD_SHADER_HANDLE;

#[derive(Clone, Resource)]
pub struct BillboardPipeline {
    pub view_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
}

impl FromWorld for BillboardPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let view_layout = render_device.create_bind_group_layout(
            "billboard_view_layout",
            &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                /* TODO: I do not think I need this actually */
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
            ],
        );

        let material_layout = render_device.create_bind_group_layout(
            "billboard_material_layout",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2, /* TODO: D2Array? */
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        );

        Self {
            view_layout,
            material_layout,
        }
    }
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BillboardPipelineKey {
    pub msaa: u32,
    pub hdr: bool,
}

impl SpecializedRenderPipeline for BillboardPipeline {
    type Key = BillboardPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        /* TODO: shader_defs */
        let shader_defs = Vec::new();

        /* TODO: vertex layout */
        let formats = vec![
            // Position
            VertexFormat::Float32x4,
            // Uv
            VertexFormat::Float32x4,
            // Color
            VertexFormat::Float32x4,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        /* TODO: HDR Texture format */
        let format = match key.hdr {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: BILLBOARD_SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: BILLBOARD_SHADER_HANDLE,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![self.view_layout.clone(), self.material_layout.clone()],
            primitive: PrimitiveState {
                conservative: false,
                cull_mode: None, /* TODO: Culling */
                front_face: FrontFace::Ccw,
                polygon_mode: PolygonMode::Fill,
                strip_index_format: None,
                topology: PrimitiveTopology::TriangleList,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("billboard_pipeline".into()),
            push_constant_ranges: vec![],
        }
    }
}
