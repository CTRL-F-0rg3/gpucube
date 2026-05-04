use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("Nie można znaleźć odpowiedniego adaptera GPU")]
    AdapterNotFound,

    #[error("Błąd tworzenia urządzenia: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),

    #[error("Błąd surface: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),

    #[error("Błąd winit: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error("Błąd tworzenia okna: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    #[error("Surface nie jest skonfigurowany — wywołaj configure() najpierw")]
    SurfaceNotConfigured,

    #[error("Błąd odczytu tekstury: {0}")]
    TextureLoad(String),

    #[error("Shader error: {0}")]
    Shader(String),

    #[error("Buffer za mały: potrzeba {needed} bajtów, dostępne {available}")]
    BufferTooSmall { needed: u64, available: u64 },

    #[error("Błąd wewnętrzny: {0}")]
    Internal(String),
}

pub type GpuResult<T> = Result<T, GpuError>;
