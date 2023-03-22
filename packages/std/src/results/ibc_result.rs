/// This is like Result but we add an Abort case
pub enum IbcResult<S, E> {
    Ok(S),
    Err(E),
    Abort,
}

impl<S, E> From<Result<S, E>> for IbcResult<S, E> {
    fn from(original: Result<S, E>) -> IbcResult<S, E> {
        match original {
            Ok(value) => IbcResult::Ok(value),
            Err(err) => IbcResult::Err(err),
        }
    }
}
