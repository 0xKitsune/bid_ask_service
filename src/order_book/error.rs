use crate::server::orderbook_service::Summary;

#[derive(thiserror::Error, Debug)]
pub enum OrderBookError {
    #[error("Poisoned lock")]
    PoisonedLock,
    #[error("Error when sending summary through channel")]
    SummarySendError(#[from] tokio::sync::broadcast::error::SendError<Summary>),
}
