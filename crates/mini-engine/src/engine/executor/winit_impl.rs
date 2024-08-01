use std::sync::Arc;

use crate::engine::Engine;

use mini_core::parking_lot::Mutex;
use mini_winit::{
    windows::Winitwindows,
    winit::{
        self, application::ApplicationHandler, event::WindowEvent, event_loop::ControlFlow,
        window::Window,
    },
};

use mini_window::window_wrapper::{RawHandleWrapper, RawHandleWrapperHolder, WindowWrapper};

pub struct WinitExecutor {
    pub engine: Engine,
    pub windows: Winitwindows,
}

impl WinitExecutor {
    pub fn new() -> Self {
        WinitExecutor {
            engine: Engine::from_params(),
            windows: Winitwindows::default(),
        }
    }
}

impl WinitExecutor {}

impl ApplicationHandler for WinitExecutor {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);

        let winit_window_attributes = Window::default_attributes();
        let window = event_loop.create_window(winit_window_attributes).unwrap();

        let window_handle = RawHandleWrapperHolder(Arc::new(Mutex::new(Some(
            RawHandleWrapper::new(&WindowWrapper::new(window)).unwrap(),
        ))));

        self.windows.primary.handle = window_handle;

        self.engine
            .graphics_context
            .initialize_graphics_context(&self.windows.primary);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.engine.update(),
            _ => {}
        }
    }
}
