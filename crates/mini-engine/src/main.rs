use mini_winit::winit::event_loop::EventLoop;

use mini_engine::engine::executor::Executor;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut state = Executor::new();

    event_loop.run_app(&mut state).unwrap();
}
