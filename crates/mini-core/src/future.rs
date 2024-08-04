mod conditional_send {
    /// Use [`ConditionalSend`] to mark an optional Send trait bound. Useful as on certain platforms (eg. Wasm),
    /// futures aren't Send.
    pub trait ConditionalSend: Send {}
    impl<T: Send> ConditionalSend for T {}
}

pub use conditional_send::*;

/// Use [`ConditionalSendFuture`] for a future with an optional Send trait bound, as on certain platforms (eg. Wasm),
/// futures aren't Send.
pub trait ConditionalSendFuture: std::future::Future + ConditionalSend {}

pub type BoxedFuture<'a, T> = std::pin::Pin<Box<dyn ConditionalSendFuture<Output = T> + 'a>>;

impl<T: std::future::Future + ConditionalSend> ConditionalSendFuture for T {}
