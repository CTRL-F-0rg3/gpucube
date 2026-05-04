// Derive macros z bytemuck — działają gdy Cargo.toml ma:
// bytemuck = { version = "1.21", features = ["derive"] }
use bytemuck::{Pod, Zeroable};
use crate::{basic::buffer::TypedBuffer, core::context::GpuContext};

// ─── Standardowy Vertex ───────────────────────────────────────────────────────

/// Standardowy wierzchołek: pozycja + normalna + UV.
/// Możesz zdefiniować własny typ — Mesh<V> jest generyczne.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub uv:       [f32; 2],
}

impl Vertex {
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self { position, normal, uv }
    }

    /// Layout dla wgpu — przekaż do `vertex_buffers` przy budowaniu pipeline.
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute { offset: 0,  shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
            wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
            wgpu::VertexAttribute { offset: 24, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
        ],
    };
}

// ─── Mesh<V> ──────────────────────────────────────────────────────────────────

/// Mesh z własnym typem wierzchołka V i indeksami u32.
pub struct Mesh<V: Pod = Vertex> {
    pub vertices: TypedBuffer<V>,
    pub indices: TypedBuffer<u32>,
    pub index_count: u32,
}

impl<V: Pod> Mesh<V> {
    pub fn new(ctx: &GpuContext, vertices: &[V], indices: &[u32], label: &str) -> Self {
        Self {
            vertices: TypedBuffer::vertex(ctx, vertices, &format!("{label}:vb")),
            indices:  TypedBuffer::index(ctx, indices,  &format!("{label}:ib")),
            index_count: indices.len() as u32,
        }
    }

    /// Bind vertex + index buffer do render pass.
    pub fn bind<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>) {
        pass.set_vertex_buffer(0, self.vertices.buf.slice(..));
        pass.set_index_buffer(self.indices.buf.slice(..), wgpu::IndexFormat::Uint32);
    }

    /// bind() + draw_indexed() w jednym.
    pub fn draw<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>) {
        self.bind(pass);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    /// Draw z instancingiem.
    pub fn draw_instanced<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>, instances: u32) {
        self.bind(pass);
        pass.draw_indexed(0..self.index_count, 0, 0..instances);
    }
}

// ─── MeshBuilder ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices:  Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn vertex(mut self, v: Vertex) -> Self {
        self.vertices.push(v); self
    }

    pub fn vertices(mut self, vs: impl IntoIterator<Item = Vertex>) -> Self {
        self.vertices.extend(vs); self
    }

    pub fn triangle(mut self, a: u32, b: u32, c: u32) -> Self {
        self.indices.extend([a, b, c]); self
    }

    /// Quad jako dwa trójkąty: ABC + ACD.
    pub fn quad(mut self, a: u32, b: u32, c: u32, d: u32) -> Self {
        self.indices.extend([a, b, c, a, c, d]); self
    }

    pub fn build(self, ctx: &GpuContext, label: &str) -> Mesh<Vertex> {
        Mesh::new(ctx, &self.vertices, &self.indices, label)
    }
}

// ─── Gotowe prymitywy ─────────────────────────────────────────────────────────

/// Quad w płaszczyźnie XY, rozmiar 1×1, wyśrodkowany w (0,0,0).
pub fn quad_mesh(ctx: &GpuContext) -> Mesh {
    let verts = [
        Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        Vertex::new([ 0.5, -0.5, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
        Vertex::new([ 0.5,  0.5, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
        Vertex::new([-0.5,  0.5, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
    ];
    Mesh::new(ctx, &verts, &[0u32, 1, 2, 0, 2, 3], "quad")
}

/// Sześcian 1×1×1 — 24 wierzchołki (4 na ścianę), poprawne normalne i UV.
pub fn cube_mesh(ctx: &GpuContext) -> Mesh {
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([ 0., 0., 1.], [[-0.5,-0.5, 0.5],[ 0.5,-0.5, 0.5],[ 0.5, 0.5, 0.5],[-0.5, 0.5, 0.5]]),
        ([ 0., 0.,-1.], [[ 0.5,-0.5,-0.5],[-0.5,-0.5,-0.5],[-0.5, 0.5,-0.5],[ 0.5, 0.5,-0.5]]),
        ([ 1., 0., 0.], [[ 0.5,-0.5, 0.5],[ 0.5,-0.5,-0.5],[ 0.5, 0.5,-0.5],[ 0.5, 0.5, 0.5]]),
        ([-1., 0., 0.], [[-0.5,-0.5,-0.5],[-0.5,-0.5, 0.5],[-0.5, 0.5, 0.5],[-0.5, 0.5,-0.5]]),
        ([ 0., 1., 0.], [[-0.5, 0.5, 0.5],[ 0.5, 0.5, 0.5],[ 0.5, 0.5,-0.5],[-0.5, 0.5,-0.5]]),
        ([ 0.,-1., 0.], [[-0.5,-0.5,-0.5],[ 0.5,-0.5,-0.5],[ 0.5,-0.5, 0.5],[-0.5,-0.5, 0.5]]),
    ];
    let uvs = [[0.0f32, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    let mut vertices = Vec::with_capacity(24);
    let mut indices  = Vec::with_capacity(36);

    for (i, (normal, positions)) in faces.iter().enumerate() {
        let base = (i * 4) as u32;
        for (j, pos) in positions.iter().enumerate() {
            vertices.push(Vertex::new(*pos, *normal, uvs[j]));
        }
        indices.extend([base, base+1, base+2, base, base+2, base+3]);
    }

    Mesh::new(ctx, &vertices, &indices, "cube")
}