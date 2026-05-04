use std::marker::PhantomData;
use bytemuck::Pod;
use wgpu::util::DeviceExt;
use crate::core::context::GpuContext;

/// Type-safe wrapper nad `wgpu::Buffer`.
/// `T` musi być `Pod` (plain old data) — działa z bytemuck.
pub struct TypedBuffer<T: Pod> {
    pub buf: wgpu::Buffer,
    pub len: u64,       // liczba elementów T
    pub usage: wgpu::BufferUsages,
    _phantom: PhantomData<T>,
}

impl<T: Pod> TypedBuffer<T> {
    /// Tworzy buffer i od razu wgrywa dane.
    pub fn new(ctx: &GpuContext, data: &[T], usage: wgpu::BufferUsages, label: &str) -> Self {
        let buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self {
            buf,
            len: data.len() as u64,
            usage,
            _phantom: PhantomData,
        }
    }

    /// Tworzy pusty buffer o podanej liczbie elementów (bez danych).
    pub fn empty(ctx: &GpuContext, count: u64, usage: wgpu::BufferUsages, label: &str) -> Self {
        let size = count * std::mem::size_of::<T>() as u64;
        let buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buf,
            len: count,
            usage,
            _phantom: PhantomData,
        }
    }

    /// Wgrywa nowe dane przez queue (COPY_DST musi być w usage).
    pub fn write(&self, ctx: &GpuContext, data: &[T]) {
        ctx.queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(data));
    }

    /// Wgrywa dane z offsetem (offset w elementach, nie bajtach).
    pub fn write_at(&self, ctx: &GpuContext, offset_elements: u64, data: &[T]) {
        let offset_bytes = offset_elements * std::mem::size_of::<T>() as u64;
        ctx.queue.write_buffer(&self.buf, offset_bytes, bytemuck::cast_slice(data));
    }

    /// Rozmiar w bajtach.
    #[inline]
    pub fn size_bytes(&self) -> u64 {
        self.len * std::mem::size_of::<T>() as u64
    }

    /// Binding descriptor — wygoda przy tworzeniu BindGroup.
    #[inline]
    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.buf.as_entire_binding()
    }
}

// ─── Wygodne konstruktory dla typowych użyć ──────────────────────────────────

impl<T: Pod> TypedBuffer<T> {
    /// Vertex buffer.
    pub fn vertex(ctx: &GpuContext, data: &[T], label: &str) -> Self {
        Self::new(
            ctx, data,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            label,
        )
    }

    /// Index buffer (u16 lub u32).
    pub fn index(ctx: &GpuContext, data: &[T], label: &str) -> Self {
        Self::new(
            ctx, data,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            label,
        )
    }

    /// Uniform buffer.
    pub fn uniform(ctx: &GpuContext, data: &[T], label: &str) -> Self {
        Self::new(
            ctx, data,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label,
        )
    }

    /// Storage buffer (read/write).
    pub fn storage(ctx: &GpuContext, data: &[T], label: &str) -> Self {
        Self::new(
            ctx, data,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            label,
        )
    }
}
