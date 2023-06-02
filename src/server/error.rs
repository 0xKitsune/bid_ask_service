#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Transport error")]
    TransportError(#[from] tonic::transport::Error),
}
