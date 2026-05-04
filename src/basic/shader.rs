use crate::core::{context::GpuContext, error::{GpuError, GpuResult}};

/// Źródło shadera — inline string lub ścieżka do pliku.
pub enum ShaderSource<'a> {
    Wgsl(&'a str),
    WgslFile(&'a str),
}

impl<'a> ShaderSource<'a> {
    /// Kompiluje shader do `wgpu::ShaderModule`.
    pub fn compile(&self, ctx: &GpuContext, label: &str) -> GpuResult<wgpu::ShaderModule> {
        let source = match self {
            ShaderSource::Wgsl(src) => src.to_string(),
            ShaderSource::WgslFile(path) => {
                std::fs::read_to_string(path)
                    .map_err(|e| GpuError::Shader(format!("Nie można otworzyć {path}: {e}")))?
            }
        };

        // wgpu::Device::create_shader_module nie zwraca błędu synchronicznie —
        // błędy kompilacji pojawią się przy pierwszym użyciu pipeline.
        // Dodajemy push_error_scope żeby złapać je wcześniej.
        ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);

        let module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        // Sprawdź błędy synchronicznie (blokujące)
        if let Some(err) = pollster::block_on(ctx.device.pop_error_scope()) {
            return Err(GpuError::Shader(err.to_string()));
        }

        Ok(module)
    }
}

/// Helper — kompiluj WGSL inline.
pub fn wgsl(ctx: &GpuContext, source: &str, label: &str) -> GpuResult<wgpu::ShaderModule> {
    ShaderSource::Wgsl(source).compile(ctx, label)
}

/// Helper — kompiluj WGSL z pliku.
pub fn wgsl_file(ctx: &GpuContext, path: &str, label: &str) -> GpuResult<wgpu::ShaderModule> {
    ShaderSource::WgslFile(path).compile(ctx, label)
}
