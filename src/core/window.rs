use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use crate::core::{
    context::{GpuContext, GpuContextConfig},
    error::GpuResult,
    surface::{GpuSurface, SurfaceConfig},
};

/// Konfiguracja okna.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "gpucube".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            vsync: true,
        }
    }
}

impl WindowConfig {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }
}

/// Callback trait — implementuj to we własnej aplikacji.
/// Wszystkie metody mają domyślne puste implementacje.
pub trait AppHandler {
    /// Wywołane raz po inicjalizacji GPU i okna.
    fn init(&mut self, _ctx: &GpuContext, _surface: &GpuSurface) {}

    /// Wywołane co ramkę. Tutaj renderujesz.
    fn render(&mut self, ctx: &GpuContext, surface: &mut GpuSurface);

    /// Wywołane po resize okna.
    fn resized(&mut self, _ctx: &GpuContext, _surface: &GpuSurface, _width: u32, _height: u32) {}

    /// Zdarzenia klawiatury, myszy, itd.
    fn event(&mut self, _ctx: &GpuContext, _event: &WindowEvent) -> bool {
        false // false = nie konsumuj eventu
    }

    /// Wywołane przed zamknięciem.
    fn exit(&mut self, _ctx: &GpuContext) {}
}

/// Gotowy, skonfigurowany state okna — można użyć bez event loop
/// jeśli integrujesz z własnym systemem.
pub struct WindowState {
    pub window: Window,
    pub ctx: GpuContext,
}

// ─── Event Loop Runner ───────────────────────────────────────────────────────

/// Uruchamia główną pętlę okna. Blokuje do zamknięcia.
pub fn run<H: AppHandler + 'static>(
    handler: H,
    win_cfg: WindowConfig,
    gpu_cfg: GpuContextConfig,
) -> GpuResult<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppRunner {
        handler,
        win_cfg,
        gpu_cfg,
        state: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

// ─── Wewnętrzne ──────────────────────────────────────────────────────────────

struct RunnerState<'w> {
    window: &'w Window,
    ctx: GpuContext,
    surface: GpuSurface<'w>,
}

struct AppRunner<H: AppHandler> {
    handler: H,
    win_cfg: WindowConfig,
    gpu_cfg: GpuContextConfig,
    state: Option<RunnerState<'static>>, // SAFETY: okno żyje tak długo jak state
}

impl<H: AppHandler> ApplicationHandler for AppRunner<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title(&self.win_cfg.title)
            .with_inner_size(PhysicalSize::new(self.win_cfg.width, self.win_cfg.height))
            .with_resizable(self.win_cfg.resizable);

        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Nie można stworzyć okna: {e}");
                event_loop.exit();
                return;
            }
        };

        // Box::leak jest bezpieczne — window żyje tak długo jak AppRunner
        let window: &'static Window = Box::leak(Box::new(window));

        let surface = match self
            .gpu_cfg
            .backends // tylko potrzebujemy instance
            .try_into_wgpu_surface(&self.gpu_cfg, window)
        {
            Ok(s) => s,
            Err(e) => {
                log::error!("Nie można stworzyć surface/GPU: {e}");
                event_loop.exit();
                return;
            }
        };

        let (ctx, gpu_surface) = surface;

        self.handler.init(&ctx, &gpu_surface);

        self.state = Some(RunnerState {
            window,
            ctx,
            surface: gpu_surface,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };

        // Przekaż event do handlera — jeśli skonsumował, nie rób nic więcej
        if self.handler.event(&state.ctx, &event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.handler.exit(&state.ctx);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                state.surface.resize(&state.ctx, size.width, size.height);
                self.handler
                    .resized(&state.ctx, &state.surface, size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                self.handler.render(&state.ctx, &mut state.surface);
                state.window.request_redraw();
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

// Helper trait do budowania surface + context razem
trait BuildSurface {
    fn try_into_wgpu_surface(
        &self,
        gpu_cfg: &GpuContextConfig,
        window: &'static Window,
    ) -> GpuResult<(GpuContext, GpuSurface<'static>)>;
}

impl BuildSurface for wgpu::Backends {
    fn try_into_wgpu_surface(
        &self,
        gpu_cfg: &GpuContextConfig,
        window: &'static Window,
    ) -> GpuResult<(GpuContext, GpuSurface<'static>)> {
        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: *self,
                ..Default::default()
            });

            let surface = instance.create_surface(window)?;

            let ctx = GpuContext::with_surface(&surface, gpu_cfg.clone()).await?;

            let size = window.inner_size();
            let present_mode = if gpu_cfg.required_features.is_empty() {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            };

            let gpu_surface = GpuSurface::configure(
                surface,
                &ctx,
                SurfaceConfig::new(size.width, size.height)
                    .with_present_mode(present_mode),
            )?;

            Ok((ctx, gpu_surface))
        })
    }
}