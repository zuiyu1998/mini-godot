use std::sync::Arc;

use crate::renderer::prelude::{GraphicsContext, ImageLoader};
use mini_core::tracing_subscriber::{self, filter::EnvFilter, fmt, prelude::*};
use mini_resource::prelude::ResourceManager;
use mini_task::TaskPool;
use mini_window::prelude::ErasedWindow;

use crate::scene::Scene;

pub struct Engine {
    resource_manager: ResourceManager,
    pub graphics_context: GraphicsContext,
    pub scene: Scene,
}

impl Engine {
    pub fn initialize(&mut self, window: &ErasedWindow) {
        self.graphics_context
            .initialize(&window, &self.resource_manager);
    }

    pub fn from_params() -> Self {
        tracing_subscriber::fmt()
            .with_env_filter("mini_renderer=info")
            .init();

        let task_pool = Arc::new(TaskPool::new());
        let resource_manager = ResourceManager::new(task_pool);

        let scene = Scene {};

        Engine {
            resource_manager,
            graphics_context: GraphicsContext::Uninitialized,
            scene,
        }
    }

    pub fn update(&mut self) {
        self.graphics_context.render();
    }
}
