use crate::error::Error;

#[derive(PartialEq, Debug)]
pub enum State {
    Idle,
    Alive,
    Error(Error),
}
