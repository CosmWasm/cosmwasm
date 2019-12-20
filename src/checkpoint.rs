use crate::traits::Storage;
use crate::errors::Result;

// TODO implement Storage with queue
pub struct Checkpoint<'a, T: Storage> {
    storage: &'a mut T,
}

impl<'a, S: Storage> Checkpoint<'a, S> {
    pub fn commit(&mut self) {}
}


pub fn with_checkpoint<S: Storage, T>(storage: &mut S, tx: &dyn Fn(&mut Checkpoint<S>) -> Result<T>) -> Result<T>  {
    let mut c = Checkpoint{storage};
    let res = tx(&mut c)?;
    c.commit();
    Ok(res)
}