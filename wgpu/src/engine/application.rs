use std::{env, sync};

use winit::{application, event, keyboard, window};

pub const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.25, g: 0.25, b: 0.4, a: 1.0 };

use crate::engine::{
    inputs,
    render::{self, Renderer},
    texture,
};

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

    fn physics_frame(
        &mut self,
        inputs: &mut inputs::Inputs,
        context: &render::RenderContext,
        renderer: &render::Renderer,
    );

    fn render_frame(
        &self,
        inputs: &inputs::Inputs,
        context: &mut render::RenderContext,
        renderer: &mut render::Renderer,
    );
}

#[derive(bon::Builder, Debug)]
struct State<T> {
    inner_state: T,
    window: sync::Arc<window::Window>,
    inputs: inputs::Inputs,
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

        let inputs = inputs::Inputs::default();

        Ok(Self { inner_state, window, context, renderer, inputs })
    }

    fn update(&mut self) {
        self.inner_state.physics_frame(&mut self.inputs, &self.context, &self.renderer);
        self.inner_state.render_frame(&self.inputs, &mut self.context, &mut self.renderer);
    }

    fn config_changed(&mut self, width: u32, height: u32) {
        self.context.config.width = width.max(1);
        self.context.config.height = height.max(1);
        self.context.surface.configure(&self.context.device, &self.context.config);
        self.renderer.depth_buffer = texture::Texture::depth_texture(&self.context)
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
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_buffer.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
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

    fn device_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        let state = match &mut self.inner {
            | Some(state) => state,
            | None => return,
        };

        if let event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let (x, y) = &mut state.inputs.mouse_delta;
            *x += dx as f32;
            *y += dy as f32;
        }
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
            | event::WindowEvent::PinchGesture { .. } => todo!(),
            | event::WindowEvent::PanGesture { .. } => todo!(),
            | event::WindowEvent::DoubleTapGesture { .. } => todo!(),
            | event::WindowEvent::RotationGesture { .. } => todo!(),
            | event::WindowEvent::TouchpadPressure { .. } => todo!(),
            | event::WindowEvent::AxisMotion { .. } => todo!(),
            | event::WindowEvent::ActivationTokenDone { .. } => todo!(),
            | event::WindowEvent::ScaleFactorChanged { .. } => todo!(),
            | event::WindowEvent::DroppedFile(_) => todo!(),
            | event::WindowEvent::HoveredFile(_) => todo!(),
            | event::WindowEvent::HoveredFileCancelled => todo!(),
            | event::WindowEvent::Ime(_) => todo!(),
            | event::WindowEvent::Touch(_) => todo!(),
            | event::WindowEvent::ThemeChanged(_) => todo!(),
            | event::WindowEvent::Occluded(_) => todo!(),

            | event::WindowEvent::CursorMoved { .. } => {}
            | event::WindowEvent::CursorEntered { .. } => {}
            | event::WindowEvent::CursorLeft { .. } => {}
            | event::WindowEvent::MouseWheel { .. } => {}
            | event::WindowEvent::Moved(_) => {}
            | event::WindowEvent::Focused(_) => {}
            | event::WindowEvent::ModifiersChanged(_) => {}

            | event::WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            | event::WindowEvent::Resized(size) => {
                log::info!("Resize requested: {:?}", size);
                state.config_changed(size.width, size.height);
            }
            | event::WindowEvent::MouseInput { state: ele_state, button, .. } => {
                log::info!("Mouse pressed: {:?}", button);
                match ele_state {
                    | event::ElementState::Pressed => {
                        let (left, right) = &mut state.inputs.mouse_pressed;
                        match button {
                            | event::MouseButton::Left => {
                                *left = true;
                            }
                            | event::MouseButton::Right => {
                                *right = true;
                            }
                            | event::MouseButton::Middle => todo!(),
                            | event::MouseButton::Back => todo!(),
                            | event::MouseButton::Forward => todo!(),
                            | event::MouseButton::Other(_) => todo!(),
                        }
                    }
                    | event::ElementState::Released => {
                        let (left, right) = &mut state.inputs.mouse_released;
                        match button {
                            | event::MouseButton::Left => {
                                *left = true;
                            }
                            | event::MouseButton::Right => {
                                *right = true;
                            }
                            | event::MouseButton::Middle => todo!(),
                            | event::MouseButton::Back => todo!(),
                            | event::MouseButton::Forward => todo!(),
                            | event::MouseButton::Other(_) => todo!(),
                        }
                    }
                }
            }
            | event::WindowEvent::KeyboardInput { event, .. } => {
                log::info!("Keyboard input: {:?}", event);
                let keyboard::PhysicalKey::Code(keycode) = event.physical_key
                else {
                    return;
                };
                let name = keycode_name(keycode);
                match event.state {
                    | event::ElementState::Pressed => {
                        state.inputs.key_pressed.insert(name);
                        state.inputs.key_released.remove(name);
                    }
                    | event::ElementState::Released => {
                        state.inputs.key_pressed.remove(name);
                        state.inputs.key_released.insert(name);
                    }
                };
            }
            | event::WindowEvent::RedrawRequested => {
                state.update();

                if state.inputs.request_quit {
                    log::info!("Quit requested");
                    event_loop.exit()
                }
                if state.inputs.toggle_grab {
                    log::info!("Cursor grab requested");
                }

                if let Err(err) = state.render() {
                    log::error!("Render error: {}", err);
                    event_loop.exit();
                }
            }
            | _ => {}
        }
    }
}

fn keycode_name(keycode: keyboard::KeyCode) -> &'static str {
    match keycode {
        | keyboard::KeyCode::Backquote => "backquote",
        | keyboard::KeyCode::Backslash => "backslash",
        | keyboard::KeyCode::BracketLeft => "bracketleft",
        | keyboard::KeyCode::BracketRight => "bracketright",
        | keyboard::KeyCode::Comma => "comma",
        | keyboard::KeyCode::Digit0 => "digit0",
        | keyboard::KeyCode::Digit1 => "digit1",
        | keyboard::KeyCode::Digit2 => "digit2",
        | keyboard::KeyCode::Digit3 => "digit3",
        | keyboard::KeyCode::Digit4 => "digit4",
        | keyboard::KeyCode::Digit5 => "digit5",
        | keyboard::KeyCode::Digit6 => "digit6",
        | keyboard::KeyCode::Digit7 => "digit7",
        | keyboard::KeyCode::Digit8 => "digit8",
        | keyboard::KeyCode::Digit9 => "digit9",
        | keyboard::KeyCode::Equal => "equal",
        | keyboard::KeyCode::IntlBackslash => "intlbackslash",
        | keyboard::KeyCode::IntlRo => "intlro",
        | keyboard::KeyCode::IntlYen => "intlyen",
        | keyboard::KeyCode::KeyA => "keya",
        | keyboard::KeyCode::KeyB => "keyb",
        | keyboard::KeyCode::KeyC => "keyc",
        | keyboard::KeyCode::KeyD => "keyd",
        | keyboard::KeyCode::KeyE => "keye",
        | keyboard::KeyCode::KeyF => "keyf",
        | keyboard::KeyCode::KeyG => "keyg",
        | keyboard::KeyCode::KeyH => "keyh",
        | keyboard::KeyCode::KeyI => "keyi",
        | keyboard::KeyCode::KeyJ => "keyj",
        | keyboard::KeyCode::KeyK => "keyk",
        | keyboard::KeyCode::KeyL => "keyl",
        | keyboard::KeyCode::KeyM => "keym",
        | keyboard::KeyCode::KeyN => "keyn",
        | keyboard::KeyCode::KeyO => "keyo",
        | keyboard::KeyCode::KeyP => "keyp",
        | keyboard::KeyCode::KeyQ => "keyq",
        | keyboard::KeyCode::KeyR => "keyr",
        | keyboard::KeyCode::KeyS => "keys",
        | keyboard::KeyCode::KeyT => "keyt",
        | keyboard::KeyCode::KeyU => "keyu",
        | keyboard::KeyCode::KeyV => "keyv",
        | keyboard::KeyCode::KeyW => "keyw",
        | keyboard::KeyCode::KeyX => "keyx",
        | keyboard::KeyCode::KeyY => "keyy",
        | keyboard::KeyCode::KeyZ => "keyz",
        | keyboard::KeyCode::Minus => "minus",
        | keyboard::KeyCode::Period => "period",
        | keyboard::KeyCode::Quote => "quote",
        | keyboard::KeyCode::Semicolon => "semicolon",
        | keyboard::KeyCode::Slash => "slash",
        | keyboard::KeyCode::AltLeft => "altleft",
        | keyboard::KeyCode::AltRight => "altright",
        | keyboard::KeyCode::Backspace => "backspace",
        | keyboard::KeyCode::CapsLock => "capslock",
        | keyboard::KeyCode::ContextMenu => "contextmenu",
        | keyboard::KeyCode::ControlLeft => "controlleft",
        | keyboard::KeyCode::ControlRight => "controlright",
        | keyboard::KeyCode::Enter => "enter",
        | keyboard::KeyCode::SuperLeft => "superleft",
        | keyboard::KeyCode::SuperRight => "superright",
        | keyboard::KeyCode::ShiftLeft => "shiftleft",
        | keyboard::KeyCode::ShiftRight => "shiftright",
        | keyboard::KeyCode::Space => "space",
        | keyboard::KeyCode::Tab => "tab",
        | keyboard::KeyCode::Convert => "convert",
        | keyboard::KeyCode::KanaMode => "kanamode",
        | keyboard::KeyCode::Lang1 => "lang1",
        | keyboard::KeyCode::Lang2 => "lang2",
        | keyboard::KeyCode::Lang3 => "lang3",
        | keyboard::KeyCode::Lang4 => "lang4",
        | keyboard::KeyCode::Lang5 => "lang5",
        | keyboard::KeyCode::NonConvert => "nonconvert",
        | keyboard::KeyCode::Delete => "delete",
        | keyboard::KeyCode::End => "end",
        | keyboard::KeyCode::Help => "help",
        | keyboard::KeyCode::Home => "home",
        | keyboard::KeyCode::Insert => "insert",
        | keyboard::KeyCode::PageDown => "pagedown",
        | keyboard::KeyCode::PageUp => "pageup",
        | keyboard::KeyCode::ArrowDown => "arrowdown",
        | keyboard::KeyCode::ArrowLeft => "arrowleft",
        | keyboard::KeyCode::ArrowRight => "arrowright",
        | keyboard::KeyCode::ArrowUp => "arrowup",
        | keyboard::KeyCode::NumLock => "numlock",
        | keyboard::KeyCode::Numpad0 => "numpad0",
        | keyboard::KeyCode::Numpad1 => "numpad1",
        | keyboard::KeyCode::Numpad2 => "numpad2",
        | keyboard::KeyCode::Numpad3 => "numpad3",
        | keyboard::KeyCode::Numpad4 => "numpad4",
        | keyboard::KeyCode::Numpad5 => "numpad5",
        | keyboard::KeyCode::Numpad6 => "numpad6",
        | keyboard::KeyCode::Numpad7 => "numpad7",
        | keyboard::KeyCode::Numpad8 => "numpad8",
        | keyboard::KeyCode::Numpad9 => "numpad9",
        | keyboard::KeyCode::NumpadAdd => "numpadadd",
        | keyboard::KeyCode::NumpadBackspace => "numpadbackspace",
        | keyboard::KeyCode::NumpadClear => "numpadclear",
        | keyboard::KeyCode::NumpadClearEntry => "numpadclearentry",
        | keyboard::KeyCode::NumpadComma => "numpadcomma",
        | keyboard::KeyCode::NumpadDecimal => "numpaddecimal",
        | keyboard::KeyCode::NumpadDivide => "numpaddivide",
        | keyboard::KeyCode::NumpadEnter => "numpadenter",
        | keyboard::KeyCode::NumpadEqual => "numpadequal",
        | keyboard::KeyCode::NumpadHash => "numpadhash",
        | keyboard::KeyCode::NumpadMemoryAdd => "numpadmemoryadd",
        | keyboard::KeyCode::NumpadMemoryClear => "numpadmemoryclear",
        | keyboard::KeyCode::NumpadMemoryRecall => "numpadmemoryrecall",
        | keyboard::KeyCode::NumpadMemoryStore => "numpadmemorystore",
        | keyboard::KeyCode::NumpadMemorySubtract => "numpadmemorysubtract",
        | keyboard::KeyCode::NumpadMultiply => "numpadmultiply",
        | keyboard::KeyCode::NumpadParenLeft => "numpadparenleft",
        | keyboard::KeyCode::NumpadParenRight => "numpadparenright",
        | keyboard::KeyCode::NumpadStar => "numpadstar",
        | keyboard::KeyCode::NumpadSubtract => "numpadsubtract",
        | keyboard::KeyCode::Escape => "escape",
        | keyboard::KeyCode::Fn => "fn",
        | keyboard::KeyCode::FnLock => "fnlock",
        | keyboard::KeyCode::PrintScreen => "printscreen",
        | keyboard::KeyCode::ScrollLock => "scrolllock",
        | keyboard::KeyCode::Pause => "pause",
        | keyboard::KeyCode::BrowserBack => "browserback",
        | keyboard::KeyCode::BrowserFavorites => "browserfavorites",
        | keyboard::KeyCode::BrowserForward => "browserforward",
        | keyboard::KeyCode::BrowserHome => "browserhome",
        | keyboard::KeyCode::BrowserRefresh => "browserrefresh",
        | keyboard::KeyCode::BrowserSearch => "browsersearch",
        | keyboard::KeyCode::BrowserStop => "browserstop",
        | keyboard::KeyCode::Eject => "eject",
        | keyboard::KeyCode::LaunchApp1 => "launchapp1",
        | keyboard::KeyCode::LaunchApp2 => "launchapp2",
        | keyboard::KeyCode::LaunchMail => "launchmail",
        | keyboard::KeyCode::MediaPlayPause => "mediaplaypause",
        | keyboard::KeyCode::MediaSelect => "mediaselect",
        | keyboard::KeyCode::MediaStop => "mediastop",
        | keyboard::KeyCode::MediaTrackNext => "mediatracknext",
        | keyboard::KeyCode::MediaTrackPrevious => "mediatrackprevious",
        | keyboard::KeyCode::Power => "power",
        | keyboard::KeyCode::Sleep => "sleep",
        | keyboard::KeyCode::AudioVolumeDown => "audiovolumedown",
        | keyboard::KeyCode::AudioVolumeMute => "audiovolumemute",
        | keyboard::KeyCode::AudioVolumeUp => "audiovolumeup",
        | keyboard::KeyCode::WakeUp => "wakeup",
        | keyboard::KeyCode::Meta => "meta",
        | keyboard::KeyCode::Hyper => "hyper",
        | keyboard::KeyCode::Turbo => "turbo",
        | keyboard::KeyCode::Abort => "abort",
        | keyboard::KeyCode::Resume => "resume",
        | keyboard::KeyCode::Suspend => "suspend",
        | keyboard::KeyCode::Again => "again",
        | keyboard::KeyCode::Copy => "copy",
        | keyboard::KeyCode::Cut => "cut",
        | keyboard::KeyCode::Find => "find",
        | keyboard::KeyCode::Open => "open",
        | keyboard::KeyCode::Paste => "paste",
        | keyboard::KeyCode::Props => "props",
        | keyboard::KeyCode::Select => "select",
        | keyboard::KeyCode::Undo => "undo",
        | keyboard::KeyCode::Hiragana => "hiragana",
        | keyboard::KeyCode::Katakana => "katakana",
        | keyboard::KeyCode::F1 => "f1",
        | keyboard::KeyCode::F2 => "f2",
        | keyboard::KeyCode::F3 => "f3",
        | keyboard::KeyCode::F4 => "f4",
        | keyboard::KeyCode::F5 => "f5",
        | keyboard::KeyCode::F6 => "f6",
        | keyboard::KeyCode::F7 => "f7",
        | keyboard::KeyCode::F8 => "f8",
        | keyboard::KeyCode::F9 => "f9",
        | keyboard::KeyCode::F10 => "f10",
        | keyboard::KeyCode::F11 => "f11",
        | keyboard::KeyCode::F12 => "f12",
        | keyboard::KeyCode::F13 => "f13",
        | keyboard::KeyCode::F14 => "f14",
        | keyboard::KeyCode::F15 => "f15",
        | keyboard::KeyCode::F16 => "f16",
        | keyboard::KeyCode::F17 => "f17",
        | keyboard::KeyCode::F18 => "f18",
        | keyboard::KeyCode::F19 => "f19",
        | keyboard::KeyCode::F20 => "f20",
        | keyboard::KeyCode::F21 => "f21",
        | keyboard::KeyCode::F22 => "f22",
        | keyboard::KeyCode::F23 => "f23",
        | keyboard::KeyCode::F24 => "f24",
        | keyboard::KeyCode::F25 => "f25",
        | keyboard::KeyCode::F26 => "f26",
        | keyboard::KeyCode::F27 => "f27",
        | keyboard::KeyCode::F28 => "f28",
        | keyboard::KeyCode::F29 => "f29",
        | keyboard::KeyCode::F30 => "f30",
        | keyboard::KeyCode::F31 => "f31",
        | keyboard::KeyCode::F32 => "f32",
        | keyboard::KeyCode::F33 => "f33",
        | keyboard::KeyCode::F34 => "f34",
        | keyboard::KeyCode::F35 => "f35",
        | _ => "",
    }
}
