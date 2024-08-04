use bevy::{
    prelude::{FromWorld, Resource, World},
    render::{
        globals::GlobalsUniform,
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType,
            SamplerBindingType, ShaderStages, ShaderType, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

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

/* TODO: Spezialize BillboardPipeline, with/without direction maybe? */
/* TODO: Is this the place to put the shader I want to use? */
/*
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilemapPipelineKey {
    pub msaa: u32,
    pub map_type: TilemapType,
    pub hdr: bool,
}

impl SpecializedRenderPipeline for TilemapPipeline {
    type Key = TilemapPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();

        #[cfg(feature = "atlas")]
        shader_defs.push("ATLAS".into());

        let mesh_string = match key.map_type {
            TilemapType::Square { .. } => "SQUARE",
            TilemapType::Isometric(coord_system) => match coord_system {
                IsoCoordSystem::Diamond => "ISO_DIAMOND",
                IsoCoordSystem::Staggered => "ISO_STAGGERED",
            },
            TilemapType::Hexagon(coord_system) => match coord_system {
                HexCoordSystem::Column => "COLUMN_HEX",
                HexCoordSystem::ColumnEven => "COLUMN_EVEN_HEX",
                HexCoordSystem::ColumnOdd => "COLUMN_ODD_HEX",
                HexCoordSystem::Row => "ROW_HEX",
                HexCoordSystem::RowEven => "ROW_EVEN_HEX",
                HexCoordSystem::RowOdd => "ROW_ODD_HEX",
            },
        };
        shader_defs.push(mesh_string.into());

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

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: TILEMAP_SHADER_VERTEX,
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: TILEMAP_SHADER_FRAGMENT,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                self.view_layout.clone(),
                self.mesh_layout.clone(),
                self.material_layout.clone(),
            ],
            primitive: PrimitiveState {
                conservative: false,
                cull_mode: Some(Face::Back),
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
            label: Some("tilemap_pipeline".into()),
            push_constant_ranges: vec![],
        }
    }
}
*/
