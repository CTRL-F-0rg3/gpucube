use wgpu::{Surface, SurfaceConfiguration, SurfaceTexture, TextureView};
use crate::core::{context::GpuContext, error::{GpuError, GpuResult}};

/// Konfiguracja surface przy tworzeniu / resize.
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub present_mode: wgpu::PresentMode,
    pub format: Option<wgpu::TextureFormat>, // None = preferowany przez adapter
    pub alpha_mode: wgpu::CompositeAlphaMode,
}

impl SurfaceConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            format: None,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        }
    }

    pub fn with_present_mode(mut self, mode: wgpu::PresentMode) -> Self {
        self.present_mode = mode;
        self
    }

    pub fn with_format(mut self, fmt: wgpu::TextureFormat) -> Self {
        self.format = Some(fmt);
        self
    }
}

/// Wrapper nad `wgpu::Surface` z automatycznym zarządzaniem konfiguracją.
pub struct GpuSurface<'window> {
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub format: wgpu::TextureFormat,
}

impl<'window> GpuSurface<'window> {
    /// Konfiguruje surface. Musi być wywołane przed pierwszą ramką i po każdym resize.
    pub fn configure(
        surface: Surface<'window>,
        ctx: &GpuContext,
        cfg: SurfaceConfig,
    ) -> GpuResult<Self> {
        let caps = surface.get_capabilities(&ctx.adapter);

        let format = cfg.format.unwrap_or_else(|| {
            caps.formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(caps.formats[0])
        });

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: cfg.width.max(1),
            height: cfg.height.max(1),
            present_mode: cfg.present_mode,
            alpha_mode: cfg.alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&ctx.device, &config);

        Ok(Self { surface, config, format })
    }

    /// Rekonfiguruje po resize okna.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&ctx.device, &self.config);
        log::debug!("Surface resized: {}x{}", width, height);
    }

    /// Pobiera następną ramkę do renderowania.
    pub fn current_frame(&self) -> GpuResult<Frame> {
        let texture = self
            .surface
            .get_current_texture()
            .map_err(|e| GpuError::Internal(format!("get_current_texture: {e}")))?;

        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Frame { texture, view })
    }

    /// Rozmiar surface jako (width, height).
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Aspect ratio (width / height).
    #[inline]
    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height.max(1) as f32
    }
}

/// Pojedyncza ramka gotowa do renderowania. Prezentowana po drop lub `.present()`.
pub struct Frame {
    pub texture: SurfaceTexture,
    pub view: TextureView,
}

impl Frame {
    /// Prezentuje ramkę na ekran.
    #[inline]
    pub fn present(self) {
        self.texture.present();
    }
}
