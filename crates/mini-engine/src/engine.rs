use std::sync::Arc;

use mini_resource::prelude::ResourceManager;
use mini_task::TaskPool;

use crate::prelude::build_manager;

pub struct Engine {
    resource_manager: ResourceManager,
}

impl Engine {
    pub fn from_params() -> Self {
        let task_pool = Arc::new(TaskPool::new());
        let resource_manager = ResourceManager::new(task_pool);

        build_manager(&resource_manager);

        Engine { resource_manager }
    }
}
