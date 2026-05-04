//! # gpucube
//!
//! Cienki wrapper nad wgpu. Mniej boilerplate, pełna kontrola.
//!
//! ## Minimalna aplikacja
//!
//! ```rust,no_run
//! use gpucube::prelude::*;
//!
//! struct MyApp;
//!
//! impl AppHandler for MyApp {
//!     fn render(&mut self, ctx: &GpuContext, surface: &mut GpuSurface) {
//!         let frame = surface.current_frame().unwrap();
//!         let mut enc = ctx.encoder("frame");
//!         // ... render pass ...
//!         ctx.submit(enc);
//!         frame.present();
//!     }
//! }
//!
//! fn main() {
//!     gpucube::run(MyApp, WindowConfig::default(), GpuContextConfig::default()).unwrap();
//! }
//! ```

pub mod core;
pub mod basic;

// ─── Re-eksporty modułów ─────────────────────────────────────────────────────

pub use core::{
    context::{GpuContext, GpuContextConfig},
    surface::{GpuSurface, SurfaceConfig, Frame},
    window::{WindowConfig, AppHandler, run},
    error::{GpuError, GpuResult},
};

pub use basic::{
    buffer::TypedBuffer,
    mesh::{Mesh, MeshBuilder, Vertex, quad_mesh, cube_mesh},
    texture::{Texture, TextureConfig},
    pipeline::{RenderPipelineBuilder, PipelineDesc},
    shader::{ShaderSource, wgsl, wgsl_file},
};

// ─── Prelude — import jedną linią ────────────────────────────────────────────

/// Wszystko czego zazwyczaj potrzebujesz w jednym `use gpucube::prelude::*`.
pub mod prelude {
    pub use crate::{
        // core
        GpuContext, GpuContextConfig,
        GpuSurface, SurfaceConfig, Frame,
        WindowConfig, AppHandler, run,
        GpuError, GpuResult,
        // basic
        TypedBuffer,
        Mesh, MeshBuilder, Vertex, quad_mesh, cube_mesh,
        Texture, TextureConfig,
        RenderPipelineBuilder, PipelineDesc,
        ShaderSource, wgsl, wgsl_file,
    };

    // wgpu re-export — żebyś nie musiał dodawać wgpu osobno do Cargo.toml
    // dla typowych operacji render pass, blend state, itd.
    pub use wgpu::{
        Color, LoadOp, StoreOp, Operations,
        RenderPassDescriptor, RenderPassColorAttachment,
        RenderPassDepthStencilAttachment,
        BlendState, Face, PrimitiveTopology,
        BufferUsages, TextureUsages, TextureFormat,
        BindGroupLayoutEntry, BindGroupEntry, BindingType,
        ShaderStages, BindGroupDescriptor, BindGroupLayoutDescriptor,
        SamplerBindingType, TextureSampleType, TextureViewDimension,
        BufferBindingType,
    };
}

// ─── Skrót do pollster::block_on dla async init ──────────────────────────────

/// `block_on` do jednorazowego async init (np. `GpuContext::new`).
/// Nie używaj w render loop.
#[inline]
pub fn block_on<F: std::future::Future>(f: F) -> F::Output {
    pollster::block_on(f)
}

/// Wersja biblioteki.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");