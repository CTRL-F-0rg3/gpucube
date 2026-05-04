/// Minimalny przykład: trójkąt na ekranie.
/// cargo run --example triangle

use gpucube::{
    basic::{
        mesh::{Mesh, Vertex},
        pipeline::{PipelineDesc, RenderPipelineBuilder},
        shader,
    },
    core::{
        context::{GpuContext, GpuContextConfig},
        surface::GpuSurface,
        window::{AppHandler, WindowConfig, run},
    },
};

const SHADER: &str = r#"
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.7, 1.0, 1.0);
}
"#;

struct App {
    pipeline: Option<wgpu::RenderPipeline>,
    mesh: Option<Mesh>,
}

impl AppHandler for App {
    fn init(&mut self, ctx: &GpuContext, surface: &GpuSurface) {
        let shader = shader::wgsl(ctx, SHADER, "triangle_shader").unwrap();

        let pipeline = RenderPipelineBuilder::new(
            PipelineDesc::standard_3d(
                "triangle",
                &shader,
                &[Vertex::LAYOUT],
                &[],
                surface.format,
            )
        )
        .build(ctx);

        let verts = [
            Vertex::new([ 0.0,  0.5, 0.0], [0.0, 0.0, 1.0], [0.5, 0.0]),
            Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
            Vertex::new([ 0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
        ];
        let idx = [0u32, 1, 2];
        let mesh = Mesh::new(ctx, &verts, &idx, "triangle");

        self.pipeline = Some(pipeline);
        self.mesh = Some(mesh);
    }

    fn render(&mut self, ctx: &GpuContext, surface: &mut GpuSurface) {
        let frame = surface.current_frame().unwrap();
        let mut encoder = ctx.encoder("triangle_frame");

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            if let (Some(pipeline), Some(mesh)) = (&self.pipeline, &self.mesh) {
                pass.set_pipeline(pipeline);
                mesh.draw(&mut pass);
            }
        }

        ctx.submit(encoder);
        frame.present();
    }
}

fn main() {
    env_logger::init();

    run(
        App { pipeline: None, mesh: None },
        WindowConfig::new("gpucube — triangle", 1280, 720),
        GpuContextConfig::default(),
    )
    .unwrap();
}
