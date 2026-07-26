
//! NexTerm — entry point

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, KeyEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use nexterm_core::{App, Config};
use nexterm_input::keyboard::KeyHandler;
use nexterm_input::mouse::MouseHandler;
use nexterm_renderer::{Renderer, TabBar, TabInfo};
use nexterm_terminal::parser::TerminalParser;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    info!("NexTerm v{} starting", env!("CARGO_PKG_VERSION"));

    let config = Config::load_or_default();

    let event_loop = EventLoop::new()
        .expect("Failed to create event loop");

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = NexTermApp::new(config);

    event_loop.run_app(&mut app)
        .expect("Event loop failed");
}

struct NexTermApp {
    config:    Config,
    window:    Option<Arc<Window>>,
    renderer:  Option<Renderer>,
    parser:    Option<TerminalParser>,
    keys:      Option<KeyHandler>,
    mouse:     Option<MouseHandler>,
    app:       Option<App>,
    modifiers: ModifiersState,
    tabbar:    TabBar,
}

impl NexTermApp {
    fn new(config: Config) -> Self {
        Self {
            config,
            window:    None,
            renderer:  None,
            parser:    None,
            keys:      None,
            mouse:     None,
            app:       None,
            modifiers: ModifiersState::default(),
            tabbar:    TabBar::new(),
        }
    }
}

impl ApplicationHandler for NexTermApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("NexTerm")
            .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
            .with_min_inner_size(winit::dpi::LogicalSize::new(400, 300));

        let window = match event_loop.create_window(attrs) {
            Ok(w)  => Arc::new(w),
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        self.window = Some(Arc::clone(&window));

        let config   = &self.config;
        let renderer = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Renderer::new(
                Arc::clone(&window),
                &config.renderer.font_family,
                config.renderer.font_size,
                config.renderer.line_height,
            ));

        match renderer {
            Ok(r) => {
                self.renderer = Some(r);
            }
            Err(e) => {
                error!("Renderer failed: {}", e);
                event_loop.exit();
                return;
            }
        }

        let (rows, cols) = self.renderer
            .as_ref()
            .unwrap()
            .grid_size();

        self.parser = Some(TerminalParser::new(rows, cols));

        self.keys = Some(KeyHandler::new(
            self.config.keybinds.clone()
        ));

        self.mouse = Some(MouseHandler::new(
            self.renderer.as_ref().unwrap().font.cell_size().width  as f64,
            self.renderer.as_ref().unwrap().font.cell_size().height as f64,
        ));

        let mut core_app = App::new(self.config.clone());
        if let Err(e) = core_app.start() {
            error!("App start failed: {}", e);
            event_loop.exit();
            return;
        }

        self.app = Some(core_app);

        info!("NexTerm ready — {}x{} grid", cols, rows);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event:      WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    let (rows, cols) = renderer.grid_size();
                    if let Some(parser) = &mut self.parser {
                        parser.resize(rows, cols);
                    }
                    if let Some(app) = &mut self.app {
                        app.on_resize(size.width, size.height);
                    }
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                if let Some(keys) = &mut self.keys {
                    keys.update_modifiers(mods.state());
                }
            }

            WindowEvent::KeyboardInput { event: key_event, .. } => {
                self.handle_key(key_event, event_loop);
            }

            WindowEvent::CursorMoved { position, .. } => {
                if let Some(mouse) = &mut self.mouse {
                    mouse.on_move(position.x, position.y);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(mouse) = &mut self.mouse {
                    let action = mouse.on_scroll(delta);
                    self.handle_mouse_scroll(action);
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mouse) = &mut self.mouse {
                    mouse.on_button(button, state);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.poll_pty();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl NexTermApp {
    fn handle_key(
        &mut self,
        event:      KeyEvent,
        event_loop: &ActiveEventLoop,
    ) {
        use nexterm_input::keyboard::KeyAction;
        use nexterm_core::event::AppEvent;

        let action = match &self.keys {
            Some(k) => k.handle(&event),
            None    => return,
        };

        match action {
            KeyAction::SendBytes(bytes) => {
                if let Some(app) = &self.app {
                    if let Some(tab) = app.active_tab() {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        if let Err(e) = rt.block_on(tab.session.write(&bytes)) {
                            error!("PTY write failed: {}", e);
                        }
                    }
                }
            }
            KeyAction::AppEvent(ev) => {
                match ev {
                    AppEvent::TabCreated { .. } => {
                        if let Some(app) = &mut self.app {
                            let _ = app.new_tab();
                        }
                    }
                    AppEvent::TabClosed { .. } => {
                        if let Some(app) = &mut self.app {
                            let active = app.active_tab;
                            let _ = app.close_tab(active);
                        }
                    }
                    AppEvent::Quit => {
                        event_loop.exit();
                    }
                    _ => {}
                }
            }
            KeyAction::Ignore => {}
        }
    }

    fn handle_mouse_scroll(
        &mut self,
        action: nexterm_input::mouse::MouseAction,
    ) {
        use nexterm_input::mouse::MouseAction;
        match action {
            MouseAction::ScrollUp   { lines } => { info!("Scroll up: {}",   lines); }
            MouseAction::ScrollDown { lines } => { info!("Scroll down: {}", lines); }
            _ => {}
        }
    }

    fn poll_pty(&mut self) {
        use nexterm_pty::session::SessionEvent;

        let app = match &mut self.app {
            Some(a) => a,
            None    => return,
        };

        let parser = match &mut self.parser {
            Some(p) => p,
            None    => return,
        };

        let active_id = app.active_tab;
        if let Some(tab) = app.tabs.iter_mut().find(|t| t.id == active_id) {
            while let Ok(event) = tab.session.rx.try_recv() {
                match event {
                    SessionEvent::Data(bytes) => { parser.process(&bytes); }
                    SessionEvent::Exited      => { info!("Shell exited");  }
                    SessionEvent::Error(e)    => { error!("PTY error: {}", e); }
                }
            }
        }
    }

    fn render(&mut self) {
        let renderer = match &mut self.renderer {
            Some(r) => r,
            None    => return,
        };
        let parser = match &self.parser {
            Some(p) => p,
            None    => return,
        };
        if let Err(e) = renderer.render(parser.screen()) {
            error!("Render error: {}", e);
        }
    }
}