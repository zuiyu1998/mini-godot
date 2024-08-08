use std::sync::Arc;

use super::MeshVertexAttributeId;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MeshVertexBufferLayout {
    attribute_ids: Vec<MeshVertexAttributeId>,
    layout: VertexBufferLayout,
}

impl MeshVertexBufferLayout {
    pub fn new(attribute_ids: Vec<MeshVertexAttributeId>, layout: VertexBufferLayout) -> Self {
        Self {
            attribute_ids,
            layout,
        }
    }

    #[inline]
    pub fn contains(&self, attribute_id: impl Into<MeshVertexAttributeId>) -> bool {
        self.attribute_ids.contains(&attribute_id.into())
    }

    #[inline]
    pub fn attribute_ids(&self) -> &[MeshVertexAttributeId] {
        &self.attribute_ids
    }

    #[inline]
    pub fn layout(&self) -> &VertexBufferLayout {
        &self.layout
    }

    pub fn get_layout(
        &self,
        attribute_descriptors: &[VertexAttributeDescriptor],
    ) -> Result<VertexBufferLayout, MissingVertexAttributeError> {
        let mut attributes = Vec::with_capacity(attribute_descriptors.len());
        for attribute_descriptor in attribute_descriptors {
            if let Some(index) = self
                .attribute_ids
                .iter()
                .position(|id| *id == attribute_descriptor.id)
            {
                let layout_attribute = &self.layout.attributes[index];
                attributes.push(VertexAttribute {
                    format: layout_attribute.format,
                    offset: layout_attribute.offset,
                    shader_location: attribute_descriptor.shader_location,
                });
            } else {
                return Err(MissingVertexAttributeError {
                    id: attribute_descriptor.id,
                    name: attribute_descriptor.name,
                    pipeline_type: None,
                });
            }
        }

        Ok(VertexBufferLayout {
            array_stride: self.layout.array_stride,
            step_mode: self.layout.step_mode,
            attributes,
        })
    }
}

/// Describes the layout of the mesh vertices in GPU memory.
///
/// At most one copy of a mesh vertex buffer layout ever exists in GPU memory at
/// once. Therefore, comparing these for equality requires only a single pointer
/// comparison, and this type's [`PartialEq`] and [`Hash`] implementations take
/// advantage of this. To that end, this type doesn't implement
/// [`bevy_derive::Deref`] or [`bevy_derive::DerefMut`] in order to reduce the
/// possibility of accidental deep comparisons, which would be needlessly
/// expensive.
#[derive(Clone, Debug)]
pub struct MeshVertexBufferLayoutRef(pub Arc<MeshVertexBufferLayout>);

/// Stores the single copy of each mesh vertex buffer layout.
#[derive(Clone, Default, Resource)]
pub struct MeshVertexBufferLayouts(HashSet<Arc<MeshVertexBufferLayout>>);

impl MeshVertexBufferLayouts {
    /// Inserts a new mesh vertex buffer layout in the store and returns a
    /// reference to it, reusing the existing reference if this mesh vertex
    /// buffer layout was already in the store.
    pub fn insert(&mut self, layout: MeshVertexBufferLayout) -> MeshVertexBufferLayoutRef {
        // Because the special `PartialEq` and `Hash` implementations that
        // compare by pointer are on `MeshVertexBufferLayoutRef`, not on
        // `Arc<MeshVertexBufferLayout>`, this compares the mesh vertex buffer
        // structurally, not by pointer.
        MeshVertexBufferLayoutRef(
            self.0
                .get_or_insert_with(&layout, |layout| Arc::new(layout.clone()))
                .clone(),
        )
    }
}

impl PartialEq for MeshVertexBufferLayoutRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for MeshVertexBufferLayoutRef {}

impl Hash for MeshVertexBufferLayoutRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the address of the underlying data, so two layouts that share the same
        // `MeshVertexBufferLayout` will have the same hash.
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}
