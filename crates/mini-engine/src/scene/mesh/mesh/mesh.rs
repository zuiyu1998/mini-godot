use std::collections::BTreeMap;

use mini_core::prelude::EnumVariantMeta;
use mini_renderer::wgpu::{PrimitiveTopology, VertexFormat};

///网格
pub struct Mesh {
    primitive_topology: PrimitiveTopology,

    attributes: BTreeMap<MeshVertexAttributeId, MeshAttributeData>,
}

impl Mesh {
    /// Where the vertex is located in space. Use in conjunction with [`Mesh::insert_attribute`]
    /// or [`Mesh::with_inserted_attribute`].
    ///
    /// The format of this attribute is [`VertexFormat::Float32x3`].
    /// 顶点位置
    pub const ATTRIBUTE_POSITION: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3);

    /// The direction the vertex normal is facing in.
    /// Use in conjunction with [`Mesh::insert_attribute`] or [`Mesh::with_inserted_attribute`].
    ///
    /// The format of this attribute is [`VertexFormat::Float32x3`].
    /// 顶点法线
    pub const ATTRIBUTE_NORMAL: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x3);

    /// Texture coordinates for the vertex. Use in conjunction with [`Mesh::insert_attribute`]
    /// or [`Mesh::with_inserted_attribute`].
    ///
    /// Generally `[0.,0.]` is mapped to the top left of the texture, and `[1.,1.]` to the bottom-right.
    ///
    /// By default values outside will be clamped per pixel not for the vertex,
    /// "stretching" the borders of the texture.
    /// This behavior can be useful in some cases, usually when the borders have only
    /// one color, for example a logo, and you want to "extend" those borders.
    ///
    /// For different mapping outside of `0..=1` range,
    /// see [`ImageAddressMode`](crate::texture::ImageAddressMode).
    ///
    /// The format of this attribute is [`VertexFormat::Float32x2`].
    pub const ATTRIBUTE_UV_0: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Uv", 2, VertexFormat::Float32x2);

    /// Alternate texture coordinates for the vertex. Use in conjunction with
    /// [`Mesh::insert_attribute`] or [`Mesh::with_inserted_attribute`].
    ///
    /// Typically, these are used for lightmaps, textures that provide
    /// precomputed illumination.
    ///
    /// The format of this attribute is [`VertexFormat::Float32x2`].
    pub const ATTRIBUTE_UV_1: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Uv_1", 3, VertexFormat::Float32x2);

    /// The direction of the vertex tangent. Used for normal mapping.
    /// Usually generated with [`generate_tangents`](Mesh::generate_tangents) or
    /// [`with_generated_tangents`](Mesh::with_generated_tangents).
    ///
    /// The format of this attribute is [`VertexFormat::Float32x4`].
    pub const ATTRIBUTE_TANGENT: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Tangent", 4, VertexFormat::Float32x4);

    /// Per vertex coloring. Use in conjunction with [`Mesh::insert_attribute`]
    /// or [`Mesh::with_inserted_attribute`].
    ///
    /// The format of this attribute is [`VertexFormat::Float32x4`].
    pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Color", 5, VertexFormat::Float32x4);

    /// Per vertex joint transform matrix weight. Use in conjunction with [`Mesh::insert_attribute`]
    /// or [`Mesh::with_inserted_attribute`].
    ///
    /// The format of this attribute is [`VertexFormat::Float32x4`].
    pub const ATTRIBUTE_JOINT_WEIGHT: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_JointWeight", 6, VertexFormat::Float32x4);

    /// Per vertex joint transform matrix index. Use in conjunction with [`Mesh::insert_attribute`]
    /// or [`Mesh::with_inserted_attribute`].
    ///
    /// The format of this attribute is [`VertexFormat::Uint16x4`].
    pub const ATTRIBUTE_JOINT_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_JointIndex", 7, VertexFormat::Uint16x4);

    /// Construct a new mesh. You need to provide a [`PrimitiveTopology`] so that the
    /// renderer knows how to treat the vertex data. Most of the time this will be
    /// [`PrimitiveTopology::TriangleList`].
    pub fn new(primitive_topology: PrimitiveTopology) -> Self {
        Mesh {
            primitive_topology,
            attributes: Default::default(),
        }
    }
}

///顶点数据的索引，可能会冲突
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MeshVertexAttributeId(usize);

impl From<MeshVertexAttribute> for MeshVertexAttributeId {
    fn from(attribute: MeshVertexAttribute) -> Self {
        attribute.id
    }
}

#[derive(Debug, Clone)]
struct MeshAttributeData {
    attribute: MeshVertexAttribute,
    values: VertexAttributeValues,
}

#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
    /// The friendly name of the vertex attribute
    pub name: &'static str,

    /// The _unique_ id of the vertex attribute. This will also determine sort ordering
    /// when generating vertex buffers. Built-in / standard attributes will use "close to zero"
    /// indices. When in doubt, use a random / very large usize to avoid conflicts.
    pub id: MeshVertexAttributeId,

    /// The format of the vertex attribute.
    pub format: VertexFormat,
}

impl MeshVertexAttribute {
    pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
        Self {
            name,
            id: MeshVertexAttributeId(id),
            format,
        }
    }

    // pub const fn at_shader_location(&self, shader_location: u32) -> VertexAttributeDescriptor {
    //     VertexAttributeDescriptor::new(shader_location, self.id, self.name)
    // }
}

/// Contains an array where each entry describes a property of a single vertex.
/// Matches the [`VertexFormats`](VertexFormat).
#[derive(Clone, Debug, EnumVariantMeta)]
pub enum VertexAttributeValues {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Sint32x2(Vec<[i32; 2]>),
    Uint32x2(Vec<[u32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Sint32x3(Vec<[i32; 3]>),
    Uint32x3(Vec<[u32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<[i16; 2]>),
    Snorm16x2(Vec<[i16; 2]>),
    Uint16x2(Vec<[u16; 2]>),
    Unorm16x2(Vec<[u16; 2]>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<[i8; 2]>),
    Snorm8x2(Vec<[i8; 2]>),
    Uint8x2(Vec<[u8; 2]>),
    Unorm8x2(Vec<[u8; 2]>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}
