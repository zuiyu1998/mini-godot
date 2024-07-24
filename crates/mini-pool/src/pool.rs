use std::marker::PhantomData;

use crate::{
    handle::Handle,
    payload::{Payload, PayloadContainer},
};

#[derive(Debug)]
pub struct Pool<T, P = Option<T>>
where
    T: Sized,
    P: PayloadContainer<Element = T>,
{
    records: Vec<PoolRecord<T, P>>,
    free_stack: Vec<u32>,
}

impl<T, P> Pool<T, P>
where
    T: Sized,
    P: PayloadContainer<Element = T>,
{
    fn records_get_mut(&mut self, index: u32) -> Option<&mut PoolRecord<T, P>> {
        let index = usize::try_from(index).expect("Index overflowed usize");
        self.records.get_mut(index)
    }

    pub fn spawn(&mut self, value: T) -> Handle<T> {
        self.spawn_with(|_| value)
    }

    pub fn spawn_with<F: FnOnce(Handle<T>) -> T>(&mut self, callback: F) -> Handle<T> {
        if let Some(free_index) = self.free_stack.pop() {
            let record = self
                .records_get_mut(free_index)
                .expect("free stack contained invalid index");

            if record.payload.is_some() {
                panic!(
                    "Attempt to spawn an object at pool record with payload! Record index is {}",
                    free_index
                );
            }

            let generation = record.generation + 1;
            let handle = Handle {
                index: free_index,
                generation,
                type_marker: PhantomData,
            };

            let payload = callback(handle);

            record.generation = generation;
            record.payload.replace(payload);
            handle
        } else {
            // No free records, create new one
            let generation = 1;

            let handle = Handle {
                index: self.records.len() as u32,
                generation,
                type_marker: PhantomData,
            };

            let payload = callback(handle);

            let record = PoolRecord {
                generation,
                payload: Payload::new(payload),
            };

            self.records.push(record);

            handle
        }
    }
}

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
