use crate::payload::{Payload, PayloadContainer};

#[derive(Debug)]
struct PoolRecord<T, P = Option<T>>
where
    T: Sized,
    P: PayloadContainer<Element = T>,
{
    //只有handle中generation一致，才可以访问payload
    generation: u32,
    payload: Payload<P>,
}
