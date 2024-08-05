use std::sync::Arc;

use mini_renderer::prelude::{GraphicsContext, ImageLoader};
use mini_resource::prelude::ResourceManager;
use mini_task::TaskPool;
use mini_window::prelude::ErasedWindow;

pub struct Engine {
    resource_manager: ResourceManager,
    pub graphics_context: GraphicsContext,
}

impl Engine {
    pub fn initialize(&mut self, window: &ErasedWindow) {
        self.graphics_context
            .initialize(&window, &self.resource_manager);
    }

    pub fn from_params() -> Self {
        let task_pool = Arc::new(TaskPool::new());
        let resource_manager = ResourceManager::new(task_pool);

        build_manager(&resource_manager);

        Engine {
            resource_manager,
            graphics_context: GraphicsContext::Uninitialized,
        }
    }

    pub fn update(&mut self) {
        self.graphics_context.render();
    }
}

fn build_manager(resource_manager: &ResourceManager) {
    resource_manager.add_loader(ImageLoader::default());
}
