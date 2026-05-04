//! # core
//!
//! Fundamenty GPU — inicjalizacja, okno, surface, błędy.
//! Wszystkie typy z tego modułu są re-eksportowane na poziomie `gpucube::`.
//!
//! ## Zależności między modułami
//!
//! ```text
//! error   ← używany przez wszystkich
//! context ← device + queue + instance (niezależny od okna)
//! surface ← wymaga context, zarządza swap chain
//! window  ← wymaga context + surface, integracja z winit
//! ```

pub mod context;
pub mod error;
pub mod surface;
pub mod window;

// Re-eksporty wewnątrz core:: — żebyś mógł pisać core::GpuContext
// zamiast core::context::GpuContext
pub use context::{GpuContext, GpuContextConfig};
pub use error::{GpuError, GpuResult};
pub use surface::{Frame, GpuSurface, SurfaceConfig};
pub use window::{run, AppHandler, WindowConfig};
