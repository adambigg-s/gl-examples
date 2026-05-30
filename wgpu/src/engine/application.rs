use std::{env, sync};

use winit::{application, event, keyboard, window};

use crate::engine::render::{self, Renderer};

fn init() {
    unsafe { env::set_var("RUST_LOG", "info") };
    env_logger::init();
    log::info!("Application started");
}

fn cleanup() {
    log::info!("Application terminated successfully");
}

pub fn run<T>() -> anyhow::Result<()>
where
    T: Application,
{
    init();
    winit::event_loop::EventLoop::new()?.run_app(&mut App::<T> { inner: None })?;
    cleanup();
    Ok(())
}

pub trait Application {
    fn setup(context: &mut render::RenderContext, renderer: &mut render::Renderer) -> Self;

    fn frame(&mut self, context: &mut render::RenderContext, renderer: &mut render::Renderer);
}

#[derive(bon::Builder, Debug)]
struct State<T> {
    inner_state: T,
    window: sync::Arc<window::Window>,
    context: render::RenderContext,
    renderer: render::Renderer,
}

impl<T> State<T>
where
    T: Application,
{
    async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::debugging(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance.create_surface(sync::Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;
        log::info!("Adapter information: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some(&adapter.get_info().name),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;
        log::info!("Device name: {:?}", device.adapter_info().name);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|fmt| fmt.is_srgb()).unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes.first().copied().unwrap(),
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes.first().copied().unwrap(),
            view_formats: Vec::new(),
        };
        surface.configure(&device, &config);

        let mut context = render::RenderContext { device, queue, surface, config };
        let mut renderer = Renderer::new(&context);

        let inner_state = T::setup(&mut context, &mut renderer);

        Ok(Self { inner_state, window, context, renderer })
    }

    fn update(&mut self) {
        self.inner_state.frame(&mut self.context, &mut self.renderer);
    }

    fn config_changed(&mut self, width: u32, height: u32) {
        self.context.config.width = width.max(1);
        self.context.config.height = height.max(1);
        self.context.surface.configure(&self.context.device, &self.context.config);
        self.renderer.depth_buffer = render::Texture::depth_texture(&self.context)
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        let output = match self.context.surface.get_current_texture() {
            | wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Device lost"),
            | wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
        };

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.25, g: 0.25, b: 0.4, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer.render(&mut render_pass);
        }

        self.context.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}

#[derive(bon::Builder, Debug, Default)]
struct App<T> {
    inner: Option<State<T>>,
}

impl<T> application::ApplicationHandler for App<T>
where
    T: Application,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Application resumed");

        if self.inner.is_some() {
            return;
        }

        let window_attributes = window::WindowAttributes::default();
        let window = sync::Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.inner = Some(pollster::block_on(State::new(sync::Arc::clone(&window))).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.inner {
            | Some(state) => state,
            | None => return,
        };

        match event {
            | event::WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            | event::WindowEvent::Resized(size) => {
                log::info!("Resize requested: {:?}", size);
                state.config_changed(size.width, size.height);
            }
            | event::WindowEvent::KeyboardInput { event, .. } => {
                log::info!("Keyboard input: {:?}", event);
                if event.state != event::ElementState::Pressed {
                    return;
                }
                if let keyboard::PhysicalKey::Code(code) = event.physical_key
                    && code == keyboard::KeyCode::Escape
                {
                    event_loop.exit();
                }
            }
            | event::WindowEvent::RedrawRequested => {
                state.update();
                if let Err(err) = state.render() {
                    log::error!("Render error: {}", err);
                    event_loop.exit();
                }
            }
            | _ => {}
        }
    }
}
