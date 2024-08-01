use std::collections::HashMap;

use mini_window::{
    window::{PrimaryWindow, Window},
    window_wrapper::WindowWrapper,
};
use winit::window::WindowId;

#[derive(Debug)]
pub struct WinitWindow {
    pub handle: WindowWrapper<Window>,
    pub window: Window,
}

#[derive(Debug, Default)]

pub struct Winitwindows {
    pub primary: PrimaryWindow,
    pub windows: HashMap<WindowId, WinitWindow>,
}

impl Winitwindows {
    pub fn create_window(&mut self) {}
}

pub fn create_window() {}
