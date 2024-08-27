use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("there isn't a defined strategy for beta reduction with this name")]
    UnknownBetaOrder,
}
