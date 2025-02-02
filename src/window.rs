use std::{num::NonZeroUsize, sync::Arc};

use vello::{
    peniko::color::AlphaColor,
    util::{RenderContext, RenderSurface},
    wgpu::{Maintain, PresentMode},
    AaConfig, RenderParams, Renderer, RendererOptions,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::ModifiersState,
    window::{Window, WindowId},
};

use crate::{drawer::Drawer, editor::Editor, math::Vec2u};

pub struct EditorWindow {
    handler: Option<WindowHandler>,
}

impl EditorWindow {
    pub fn new() -> Self {
        Self { handler: None }
    }

    pub fn run(&mut self) {
        EventLoop::new()
            .expect("able to create event loop, which is core of the app")
            .run_app(self)
            .expect("no issue with loop");
    }
}

impl ApplicationHandler for EditorWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.handler
            .get_or_insert(WindowHandler::init(event_loop))
            .resume();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.handler.as_mut().map(|handler| handler.suspend());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handler
            .as_mut()
            .map(|handler| handler.handle_window_event(event_loop, window_id, event));
    }
}

struct WindowHandler {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    window: Arc<Window>,
    state: WindowState,
    drawer: Drawer,

    modifiers: ModifiersState,

    editor: Editor,
}

impl WindowHandler {
    fn init(event_loop: &ActiveEventLoop) -> Self {
        let window_size = Vec2u { x: 1024, y: 768 };

        let mut editor = Editor::new(window_size);
        // TODO: Any better place to put this?
        editor.open_arg_file();

        Self {
            context: RenderContext::new(),
            renderers: vec![],
            window: create_winit_window(event_loop, window_size),
            state: WindowState::Suspended,
            drawer: Drawer::init(),
            modifiers: ModifiersState::default(),
            editor,
        }
    }

    fn resume(&mut self) {
        if matches!(self.state, WindowState::Suspended) {
            let surface =
                StaticRenderSurface::create_from_arc_window(self.window.clone(), &mut self.context);

            // TODO: Use a cheap hashmap instead?
            self.renderers
                .resize_with(self.context.devices.len(), || None);
            self.renderers[surface.surface().dev_id]
                .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

            self.state = WindowState::Active(ActiveWindowState { surface });
        }
    }

    fn suspend(&mut self) {
        if matches!(self.state, WindowState::Active(..)) {
            self.state = WindowState::Suspended;
        }
    }

    fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if window_id != self.window.id() {
            // not our window's event, ignore
            return;
        }

        let WindowState::Active(active_state) = &mut self.state else {
            // not active, ignore
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.context.resize_surface(
                    active_state.surface.surface_mut(),
                    size.width,
                    size.height,
                );
                self.editor.resize(size.into());
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers = state.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.editor
                    .handle_key_event(&event, &self.modifiers, &event_loop);
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.drawer.reset();

                self.editor.render(&mut self.drawer);

                let surface = active_state.surface.surface();
                let width = surface.config.width;
                let height = surface.config.height;

                let device_handle = &self.context.devices[surface.dev_id];

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("able to get surface texture");

                self.renderers[surface.dev_id]
                    .as_mut()
                    .expect("inited during resume")
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.drawer.scene_ref(),
                        &surface_texture,
                        &RenderParams {
                            base_color: AlphaColor::BLACK,
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("able to render");

                surface_texture.present();

                device_handle.device.poll(Maintain::Poll);
            }
            _ => {}
        }
    }
}

enum WindowState {
    Suspended,
    Active(ActiveWindowState),
}

struct ActiveWindowState {
    surface: StaticRenderSurface,
}

/// A `RenderSurface` that is backed by an `Arc<Window>`, which
/// allows us to safely keep the internal `surface` as a
/// `RenderSurface<'static>`.
struct StaticRenderSurface {
    surface: RenderSurface<'static>,
}

impl StaticRenderSurface {
    fn create_from_arc_window(window: Arc<Window>, context: &mut RenderContext) -> Self {
        let size = window.inner_size();
        let surface = context.create_surface(
            window.clone(),
            size.width,
            size.height,
            PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface).expect("able to create drawing surface");

        Self { surface }
    }

    fn surface(&self) -> &RenderSurface<'static> {
        &self.surface
    }

    fn surface_mut(&mut self) -> &mut RenderSurface<'static> {
        &mut self.surface
    }
}

fn create_winit_window(event_loop: &ActiveEventLoop, window_size: Vec2u) -> Arc<Window> {
    Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize::<u32>::from(window_size))
                    .with_resizable(true)
                    .with_title("knap_editor"),
            )
            .expect("able to create window"),
    )
}

fn create_vello_renderer(context: &RenderContext, surface: &StaticRenderSurface) -> Renderer {
    let surface = surface.surface();
    Renderer::new(
        &context.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("able to create renderer")
}
