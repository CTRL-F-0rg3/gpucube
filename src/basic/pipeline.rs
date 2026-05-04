use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Face, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, StencilState, TextureFormat,
    VertexBufferLayout, VertexState,
};
use crate::core::context::GpuContext;

/// Opis pipeline — wygodna struktura zamiast ogromnego inline descriptor.
pub struct PipelineDesc<'a> {
    pub label: &'a str,
    pub shader: &'a wgpu::ShaderModule,
    pub vs_entry: &'a str,
    pub fs_entry: &'a str,
    pub vertex_layouts: &'a [VertexBufferLayout<'a>],
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub color_format: TextureFormat,
    pub depth_format: Option<TextureFormat>,
    pub topology: PrimitiveTopology,
    pub cull_mode: Option<Face>,
    pub blend: Option<BlendState>,
    pub polygon_mode: PolygonMode,
    pub depth_write: bool,
}

impl<'a> PipelineDesc<'a> {
    /// Solidne defaults dla 3D — depth test, back-face culling, brak blendingu.
    pub fn standard_3d(
        label: &'a str,
        shader: &'a wgpu::ShaderModule,
        vertex_layouts: &'a [VertexBufferLayout<'a>],
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        color_format: TextureFormat,
    ) -> Self {
        Self {
            label,
            shader,
            vs_entry: "vs_main",
            fs_entry: "fs_main",
            vertex_layouts,
            bind_group_layouts,
            color_format,
            depth_format: Some(TextureFormat::Depth32Float),
            topology: PrimitiveTopology::TriangleList,
            cull_mode: Some(Face::Back),
            blend: None,
            polygon_mode: PolygonMode::Fill,
            depth_write: true,
        }
    }

    /// Defaults dla UI/2D — bez depth, bez cullingu, z alpha blendingiem.
    pub fn standard_2d(
        label: &'a str,
        shader: &'a wgpu::ShaderModule,
        vertex_layouts: &'a [VertexBufferLayout<'a>],
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        color_format: TextureFormat,
    ) -> Self {
        Self {
            label,
            shader,
            vs_entry: "vs_main",
            fs_entry: "fs_main",
            vertex_layouts,
            bind_group_layouts,
            color_format,
            depth_format: None,
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            blend: Some(BlendState::ALPHA_BLENDING),
            polygon_mode: PolygonMode::Fill,
            depth_write: false,
        }
    }
}

/// Builder pattern dla `wgpu::RenderPipeline`.
pub struct RenderPipelineBuilder<'a> {
    desc: PipelineDesc<'a>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(desc: PipelineDesc<'a>) -> Self { Self { desc } }

    pub fn with_cull(mut self, face: Option<Face>) -> Self {
        self.desc.cull_mode = face; self
    }

    pub fn with_blend(mut self, blend: BlendState) -> Self {
        self.desc.blend = Some(blend); self
    }

    pub fn with_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.desc.topology = topology; self
    }

    pub fn wireframe(mut self) -> Self {
        self.desc.polygon_mode = PolygonMode::Line; self
    }

    pub fn no_depth_write(mut self) -> Self {
        self.desc.depth_write = false; self
    }

    pub fn vs_entry(mut self, entry: &'a str) -> Self {
        self.desc.vs_entry = entry; self
    }

    pub fn fs_entry(mut self, entry: &'a str) -> Self {
        self.desc.fs_entry = entry; self
    }

    /// Buduje i zwraca gotowy `RenderPipeline`.
    pub fn build(self, ctx: &GpuContext) -> RenderPipeline {
        let d = &self.desc;

        let layout = ctx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", d.label)),
            bind_group_layouts: d.bind_group_layouts,
            push_constant_ranges: &[],
        });

        let depth_stencil = d.depth_format.map(|format| DepthStencilState {
            format,
            depth_write_enabled: d.depth_write,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });

        ctx.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(d.label),
            layout: Some(&layout),
            vertex: VertexState {
                module: d.shader,
                entry_point: d.vs_entry,  // wgpu 0.19: &str, nie Option<&str>
                buffers: d.vertex_layouts,
            },
            fragment: Some(FragmentState {
                module: d.shader,
                entry_point: d.fs_entry,  // wgpu 0.19: &str, nie Option<&str>
                targets: &[Some(ColorTargetState {
                    format: d.color_format,
                    blend: d.blend,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: d.topology,
                front_face: FrontFace::Ccw,
                cull_mode: d.cull_mode,
                polygon_mode: d.polygon_mode,
                ..Default::default()
            },
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
            // cache: None  — dodane w wgpu 0.20, nie istnieje w 0.19
        })
    }
}