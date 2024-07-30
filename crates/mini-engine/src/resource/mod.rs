pub mod image;

use image::PngLoader;

use mini_resource::prelude::ResourceManager;

pub fn build_manager(manager: &ResourceManager) {
    manager.state().add_loader(PngLoader);
}
