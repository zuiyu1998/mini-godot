use std::{
    clone,
    collections::{HashMap, HashSet},
};

use mini_resource::prelude::Resource;

use super::{Shader, ShaderDefVal, ShaderImport};
use crate::wrapper::render_resource_wrapper;

render_resource_wrapper!(ErasedShaderModule, wgpu::ShaderModule);

#[derive(Default)]
struct ShaderData {
    // pipelines: HashSet<CachedPipelineId>,
    processed_shaders: HashMap<Box<[ShaderDefVal]>, ErasedShaderModule>,
    //已加载的import
    resolved_imports: HashSet<ShaderImport>,
    //依赖它的import
    dependents: HashSet<ShaderImport>,

    //所有需要加载的import
    all_resolved_imports: HashSet<ShaderImport>,
    finished: bool,
}

impl ShaderData {
    fn from_imports(imports: &[ShaderImport]) -> Self {
        Self {
            all_resolved_imports: imports
                .iter()
                .map(|import_path| import_path.clone())
                .collect::<HashSet<ShaderImport>>(),
            ..Default::default()
        }
    }
}

pub struct ShaderCache {
    data: HashMap<ShaderImport, ShaderData>,

    composer: naga_oil::compose::Composer,

    shaders: HashMap<ShaderImport, Resource<Shader>>,

    import_path_shaders: HashSet<ShaderImport>,

    //被依赖的import的观察者
    waiting_on_import: HashMap<ShaderImport, Vec<ShaderImport>>,
}

impl ShaderCache {
    fn set_shader(&mut self, shader: Resource<Shader>) {
        let import_path = shader.data_ref().import_path().clone();
        let import_paths = shader.data_ref().imports.clone();

        self.import_path_shaders.insert(import_path.clone());

        //唤醒所有的注册者
        if let Some(waiting_imports) = self.waiting_on_import.get_mut(&import_path) {
            for waiting_import in waiting_imports.drain(..) {
                // resolve waiting shader import
                let data = self
                    .data
                    .entry(waiting_import.clone())
                    .or_insert(ShaderData::from_imports(&import_paths));
                data.resolved_imports.insert(waiting_import.clone());
                data.dependents.insert(import_path.clone());

                data.all_resolved_imports.remove(&waiting_import);
                if data.all_resolved_imports.is_empty() {
                    data.finished = true;
                }
            }
        }
        //将所有未加载的path注册到waiting_on_import 观察器中
        for dependent_import_path in import_paths.clone() {
            if let Some(import_path) = self.import_path_shaders.get(&dependent_import_path) {
                // resolve import because it is currently available
                let data = self
                    .data
                    .entry(import_path.clone())
                    .or_insert(ShaderData::from_imports(&import_paths));
                data.resolved_imports.insert(import_path.clone());
                data.dependents.insert(import_path.clone());

                data.all_resolved_imports.remove(&import_path);
                if data.all_resolved_imports.is_empty() {
                    data.finished = true;
                }
            } else {
                let waiting = self
                    .waiting_on_import
                    .entry(dependent_import_path.clone())
                    .or_default();
                waiting.push(import_path.clone());
            }
        }
        self.shaders.insert(import_path.to_owned(), shader);
    }
}
