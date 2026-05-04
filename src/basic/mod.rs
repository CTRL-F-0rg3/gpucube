//! # basic
//!
//! Helpery własne — mesh, buffery, shadery, tekstury, pipeline.
//! Tu trafiają twoje własne abstrakcje ponad surowym wgpu.
//!
//! ## Moduły
//!
//! | Moduł        | Co robi                                                  |
//! |--------------|----------------------------------------------------------|
//! | `buffer`     | `TypedBuffer<T>` — type-safe vertex/index/uniform/storage |
//! | `mesh`       | `Mesh<V>`, `MeshBuilder`, gotowe prymitywy (cube, quad)  |
//! | `shader`     | Kompilacja WGSL inline lub z pliku, error scope          |
//! | `texture`    | `Texture` z widokiem + samplerem, depth, solid color     |
//! | `pipeline`   | `RenderPipelineBuilder` z sensownymi defaults            |

pub mod buffer;
pub mod mesh;
pub mod pipeline;
pub mod shader;
pub mod texture;

// Re-eksporty wewnątrz basic:: — basic::Mesh zamiast basic::mesh::Mesh
pub use buffer::TypedBuffer;
pub use mesh::{cube_mesh, quad_mesh, Mesh, MeshBuilder, Vertex};
pub use pipeline::{PipelineDesc, RenderPipelineBuilder};
pub use shader::{wgsl, wgsl_file, ShaderSource};
pub use texture::{Texture, TextureConfig};