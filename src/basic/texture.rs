use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use crate::core::{context::GpuContext, error::{GpuError, GpuResult}};

/// Konfiguracja tekstury przy tworzeniu.
#[derive(Debug, Clone)]
pub struct TextureConfig {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub mip_levels: u32,
    pub label: String,
}

impl TextureConfig {
    pub fn color_target(width: u32, height: u32, label: impl Into<String>) -> Self {
        Self {
            width,
            height,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            mip_levels: 1,
            label: label.into(),
        }
    }

    pub fn depth(width: u32, height: u32, label: impl Into<String>) -> Self {
        Self {
            width,
            height,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            mip_levels: 1,
            label: label.into(),
        }
    }

    pub fn with_format(mut self, fmt: TextureFormat) -> Self {
        self.format = fmt;
        self
    }

    pub fn with_usage(mut self, usage: TextureUsages) -> Self {
        self.usage = usage;
        self
    }
}

/// Wrapper nad wgpu::Texture z gotowym widokiem i samplerem.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: TextureFormat,
    pub size: (u32, u32),
}

impl Texture {
    /// Tworzy pustą teksturę wg konfiguracji.
    pub fn new(ctx: &GpuContext, cfg: &TextureConfig) -> Self {
        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some(&cfg.label),
            size: Extent3d {
                width: cfg.width,
                height: cfg.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: cfg.mip_levels,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: cfg.format,
            usage: cfg.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::default_sampler(ctx, cfg.format);

        Self {
            texture,
            view,
            sampler,
            format: cfg.format,
            size: (cfg.width, cfg.height),
        }
    }

    /// Tworzy teksturę z surowych RGBA bajtów.
    pub fn from_rgba(
        ctx: &GpuContext,
        width: u32,
        height: u32,
        data: &[u8],
        label: &str,
    ) -> GpuResult<Self> {
        if data.len() as u32 != width * height * 4 {
            return Err(GpuError::TextureLoad(format!(
                "Nieprawidłowy rozmiar danych: oczekiwano {} bajtów, dostałem {}",
                width * height * 4,
                data.len()
            )));
        }

        let cfg = TextureConfig::color_target(width, height, label);
        let tex = Self::new(ctx, &cfg);

        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );

        Ok(tex)
    }

    /// Tworzy 1x1 teksturę z podanym kolorem RGBA.
    pub fn solid_color(ctx: &GpuContext, r: u8, g: u8, b: u8, a: u8, label: &str) -> GpuResult<Self> {
        Self::from_rgba(ctx, 1, 1, &[r, g, b, a], label)
    }

    /// Tworzy depth texture dopasowany do rozmiaru.
    pub fn depth_texture(ctx: &GpuContext, width: u32, height: u32, label: &str) -> Self {
        Self::new(ctx, &TextureConfig::depth(width, height, label))
    }

    /// Binding descriptor do BindGroupEntry.
    pub fn binding(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.view)
    }

    /// Sampler binding descriptor.
    pub fn sampler_binding(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.sampler)
    }

    fn default_sampler(ctx: &GpuContext, format: TextureFormat) -> wgpu::Sampler {
        let is_depth = matches!(format, TextureFormat::Depth32Float | TextureFormat::Depth24Plus);
        ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("default_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: if is_depth { Some(wgpu::CompareFunction::LessEqual) } else { None },
            ..Default::default()
        })
    }
}
