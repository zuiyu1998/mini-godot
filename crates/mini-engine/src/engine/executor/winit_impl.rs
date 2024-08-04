use crate::engine::Engine;

use mini_window::window::{AppLifecycle, Window};
use mini_winit::{
    windows::WinitWindows,
    winit::{self, application::ApplicationHandler, event::WindowEvent, event_loop::ControlFlow},
};

pub struct WinitExecutor {
    pub engine: Engine,
    pub windows: WinitWindows,
    lifecycle: AppLifecycle,
    pub is_initialize: bool,
}

impl WinitExecutor {
    pub fn new() -> Self {
        WinitExecutor {
            engine: Engine::from_params(),
            windows: WinitWindows::default(),
            lifecycle: AppLifecycle::Idle,
            is_initialize: false,
        }
    }
}

impl WinitExecutor {}

impl ApplicationHandler for WinitExecutor {
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        if self.lifecycle == AppLifecycle::Idle {
            self.windows.create_window(event_loop, Window::default());
        }
    }

    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.lifecycle = AppLifecycle::WillResume;
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);

        if self.lifecycle == AppLifecycle::WillResume {
            if !self.is_initialize {
                self.is_initialize = true;

                self.engine.initialize(
                    &self
                        .windows
                        .windows
                        .get(&self.windows.primary.unwrap())
                        .unwrap()
                        .erased_window,
                );
            }

            for window in self.windows.windows.values() {
                self.engine
                    .graphics_context
                    .initialize_windows(&window.erased_window);
            }

            self.engine
                .graphics_context
                .add_render_pipeline(&self.windows.primary.unwrap());

            self.lifecycle = AppLifecycle::Running;
        }

        self.engine.update();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.engine.update(),
            _ => {}
        }
    }
}
