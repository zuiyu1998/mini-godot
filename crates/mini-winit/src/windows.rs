use std::{collections::HashMap, sync::Arc};

use mini_core::parking_lot::Mutex;
use mini_window::{
    window::{ErasedWindow, Window},
    window_wrapper::{RawHandleWrapper, RawHandleWrapperHolder, WindowWrapper},
};
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window as RawWinitWindow, WindowId},
};

#[derive(Debug)]
pub struct WinitWindow {
    pub window_wrapper: WindowWrapper<RawWinitWindow>,
    pub erased_window: ErasedWindow,
}

#[derive(Debug, Default)]

pub struct WinitWindows {
    pub primary: Option<WindowId>,
    pub windows: HashMap<WindowId, WinitWindow>,
}

impl WinitWindows {
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop, window: Window) {
        let winit_window_attributes = RawWinitWindow::default_attributes();
        let winit_window = event_loop.create_window(winit_window_attributes).unwrap();
        let window_id = winit_window.id();

        let window_wrapper = WindowWrapper::new(winit_window);

        let raw_handle_wrapper = RawHandleWrapper::new(&window_wrapper).unwrap();

        let raw_handle_wrapper_holder =
            RawHandleWrapperHolder(Arc::new(Mutex::new(Some(raw_handle_wrapper.clone()))));

        let window = WinitWindow {
            window_wrapper,
            erased_window: ErasedWindow {
                raw_handle_wrapper,
                raw_handle_wrapper_holder,
                window,
            },
        };

        if self.primary.is_none() {
            self.primary = Some(window_id.clone());
        }

        self.windows.insert(window_id, window);
    }
}
