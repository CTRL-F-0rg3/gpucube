use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue};
use crate::core::error::{GpuError, GpuResult};

/// Konfiguracja kontekstu GPU przy inicjalizacji.
#[derive(Debug, Clone)]
pub struct GpuContextConfig {
    pub backends: wgpu::Backends,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
    pub label: Option<&'static str>,
}

impl Default for GpuContextConfig {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::all(),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("gpucube"),
        }
    }
}

/// Centralny kontekst GPU. Trzyma Arc<Device> i Arc<Queue>,
/// można go klonować i przekazywać między systemami.
#[derive(Clone, Debug)]
pub struct GpuContext {
    pub instance: Arc<Instance>,
    pub adapter: Arc<Adapter>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl GpuContext {
    /// Tworzy kontekst bez okna (headless lub przed stworzeniem surface).
    pub async fn new(config: GpuContextConfig) -> GpuResult<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: config.backends,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::AdapterNotFound)?;

        Self::from_adapter(Arc::new(instance), Arc::new(adapter), config).await
    }

    /// Tworzy kontekst kompatybilny z podanym surface (wymagane dla swap chain).
    pub async fn with_surface(
        surface: &wgpu::Surface<'_>,
        config: GpuContextConfig,
    ) -> GpuResult<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: config.backends,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::AdapterNotFound)?;

        Self::from_adapter(Arc::new(instance), Arc::new(adapter), config).await
    }

    async fn from_adapter(
        instance: Arc<Instance>,
        adapter: Arc<Adapter>,
        config: GpuContextConfig,
    ) -> GpuResult<Self> {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: config.label,
                    required_features: config.required_features,
                    required_limits: config.required_limits,
                    // memory_hints — dodane w wgpu 0.20, nie istnieje w 0.19
                },
                None,
            )
            .await?;

        log::info!(
            "GPU: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        Ok(Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    /// Tworzy command encoder z etykietą.
    #[inline]
    pub fn encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(label),
        })
    }

    /// finish() + queue.submit() w jednym.
    #[inline]
    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Informacje o adapterze (nazwa GPU, backend, itd.).
    #[inline]
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Dostęp do surowego `wgpu::Device` — gdy wrapper nie wystarcza.
    #[inline]
    pub fn raw_device(&self) -> &Device { &self.device }

    /// Dostęp do surowej `wgpu::Queue`.
    #[inline]
    pub fn raw_queue(&self) -> &Queue { &self.queue }
}