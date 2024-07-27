use std::{
    fmt::{Display, Formatter},
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use mini_core::{
    parking_lot::Mutex,
    type_uuid::TypeUuidProvider,
    uuid::{uuid, Uuid},
};

use crate::loader::{LoadError, ResourceData, ResourceLoadError, TypedResourceData};

pub struct Resource<T>
where
    T: TypedResourceData,
{
    pub untyped: UntypedResource,
    pub type_marker: PhantomData<T>,
}

#[derive(Debug, Clone, TypeUuidProvider)]
#[type_uuid(id = "21613484-7145-4d1c-87d8-62fa767560ab")]
pub struct UntypedResource(pub Arc<Mutex<ResourceHeader>>);

impl Default for UntypedResource {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(ResourceHeader {
            kind: Default::default(),
            type_uuid: Default::default(),
            state: ResourceState::new_load_error(LoadError::new(
                "Default resource state of unknown type.",
            )),
        })))
    }
}

impl UntypedResource {
    pub fn type_uuid(&self) -> Uuid {
        self.0.lock().type_uuid
    }

    pub fn new_ok<T>(kind: ResourceKind, data: T) -> Self
    where
        T: ResourceData,
    {
        Self(Arc::new(Mutex::new(ResourceHeader {
            kind,
            type_uuid: data.type_uuid(),
            state: ResourceState::new_ok(data),
        })))
    }

    pub fn new_load_error(kind: ResourceKind, error: LoadError, type_uuid: Uuid) -> Self {
        Self(Arc::new(Mutex::new(ResourceHeader {
            kind,
            type_uuid,
            state: ResourceState::new_load_error(error),
        })))
    }

    pub fn new_pending(kind: ResourceKind, type_uuid: Uuid) -> Self {
        Self(Arc::new(Mutex::new(ResourceHeader {
            kind,
            type_uuid,
            state: ResourceState::new_pending(),
        })))
    }

    pub fn commit_ok<T: ResourceData>(&self, data: T) {
        let mut guard = self.0.lock();
        guard.type_uuid = data.type_uuid();
        guard.state.commit_ok(data);
    }

    pub fn commit_error<E: ResourceLoadError>(&self, error: E) {
        self.0.lock().state.commit_error(error);
    }
}

impl Future for UntypedResource {
    type Output = Result<Self, LoadError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.0.clone();
        let mut guard = state.lock();
        match guard.state {
            ResourceState::Pending { ref mut wakers, .. } => {
                // Collect wakers, so we'll be able to wake task when worker thread finish loading.
                let cx_waker = cx.waker();
                if let Some(pos) = wakers.iter().position(|waker| waker.will_wake(cx_waker)) {
                    wakers[pos].clone_from(cx_waker);
                } else {
                    wakers.push(cx_waker.clone())
                }

                Poll::Pending
            }
            ResourceState::LoadError { ref error, .. } => Poll::Ready(Err(error.clone())),
            ResourceState::Ok(_) => Poll::Ready(Ok(self.clone())),
        }
    }
}

#[derive(Debug)]
pub struct ResourceHeader {
    pub state: ResourceState,
    pub type_uuid: Uuid,
    pub kind: ResourceKind,
}

#[derive(Debug, Default, Clone)]
pub enum ResourceKind {
    #[default]
    Embedded,
    External(PathBuf),
}

impl ResourceKind {
    #[inline]
    pub fn is_embedded(&self) -> bool {
        matches!(self, Self::Embedded)
    }

    #[inline]
    pub fn is_external(&self) -> bool {
        !self.is_embedded()
    }
}

impl Display for ResourceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceKind::Embedded => {
                write!(f, "Embedded")
            }
            ResourceKind::External(path) => {
                write!(f, "External ({})", path.display())
            }
        }
    }
}

#[derive(Debug)]
pub enum ResourceState {
    Ok(Box<dyn ResourceData>),
    LoadError {
        /// An error. This wrapped in Option only to be Default_ed.
        error: LoadError,
    },
    Pending {
        wakers: WakersList,
    },
}

#[derive(Debug, Default)]
pub struct WakersList(Vec<Waker>);

impl Deref for WakersList {
    type Target = Vec<Waker>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WakersList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ResourceState {
    pub fn new_load_error(error: LoadError) -> Self {
        Self::LoadError { error }
    }

    pub fn new_ok<T: ResourceData>(data: T) -> Self {
        Self::Ok(Box::new(data))
    }

    pub fn new_pending() -> Self {
        Self::Pending {
            wakers: Default::default(),
        }
    }

    pub fn commit(&mut self, state: ResourceState) {
        assert!(!matches!(state, ResourceState::Pending { .. }));

        *self = state;
    }

    pub fn commit_ok<T: ResourceData>(&mut self, data: T) {
        self.commit(ResourceState::Ok(Box::new(data)))
    }

    pub fn commit_error<E: ResourceLoadError>(&mut self, error: E) {
        self.commit(ResourceState::LoadError {
            error: LoadError::new(error),
        })
    }
}
