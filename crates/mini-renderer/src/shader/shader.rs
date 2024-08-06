use std::borrow::Cow;

use mini_core::{
    prelude::TypeUuidProvider,
    tracing,
    uuid::{uuid, Uuid},
};
use mini_resource::prelude::ResourceData;

#[derive(Debug, Clone, ResourceData, TypeUuidProvider)]
#[type_uuid(id = "5fb10a22-4ea9-4a13-a58c-82f2734aefd8")]
pub struct Shader {
    pub path: String,
    pub source: Source,
    pub import_path: ShaderImport,
    pub imports: Vec<ShaderImport>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ShaderDefVal {
    Bool(String, bool),
    Int(String, i32),
    UInt(String, u32),
}

impl From<&str> for ShaderDefVal {
    fn from(key: &str) -> Self {
        ShaderDefVal::Bool(key.to_string(), true)
    }
}

impl From<String> for ShaderDefVal {
    fn from(key: String) -> Self {
        ShaderDefVal::Bool(key, true)
    }
}

impl ShaderDefVal {
    pub fn value_as_string(&self) -> String {
        match self {
            ShaderDefVal::Bool(_, def) => def.to_string(),
            ShaderDefVal::Int(_, def) => def.to_string(),
            ShaderDefVal::UInt(_, def) => def.to_string(),
        }
    }
}

impl Shader {
    fn preprocess(source: &str, path: &str) -> (ShaderImport, Vec<ShaderImport>) {
        let (import_path, imports, _) = naga_oil::compose::get_preprocessor_data(source);

        tracing::info!("{:?}, {:?}", import_path, imports);

        let import_path = import_path
            .map(ShaderImport::Custom)
            .unwrap_or_else(|| ShaderImport::AssetPath(path.to_owned()));

        let imports = imports
            .into_iter()
            .map(|import| {
                if import.import.starts_with('\"') {
                    let import = import
                        .import
                        .chars()
                        .skip(1)
                        .take_while(|c| *c != '\"')
                        .collect();
                    ShaderImport::AssetPath(import)
                } else {
                    ShaderImport::Custom(import.import)
                }
            })
            .collect();

        (import_path, imports)
    }

    pub fn from_wgsl(source: impl Into<Cow<'static, str>>, path: impl Into<String>) -> Shader {
        let source = source.into();
        let path = path.into();
        let (import_path, imports) = Shader::preprocess(&source, &path);
        Shader {
            path,
            imports,
            import_path,
            source: Source::Wgsl(source),
        }
    }

    #[inline]
    pub fn import_path(&self) -> &ShaderImport {
        &self.import_path
    }
}

#[derive(Debug, Clone)]
pub enum Source {
    Wgsl(Cow<'static, str>),
    Glsl(Cow<'static, str>, naga::ShaderStage),
    SpirV(Cow<'static, [u8]>),
    // TODO: consider the following
    // PrecompiledSpirVMacros(HashMap<HashSet<String>, Vec<u32>>)
    // NagaModule(Module) ... Module impls Serialize/Deserialize
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ShaderImport {
    AssetPath(String),
    Custom(String),
}

impl ShaderImport {
    pub fn module_name(&self) -> Cow<'_, String> {
        match self {
            ShaderImport::AssetPath(s) => Cow::Owned(format!("\"{s}\"")),
            ShaderImport::Custom(s) => Cow::Borrowed(s),
        }
    }
}
