use tokio::sync::mpsc::error::SendError;

#[derive(thiserror::Error, Debug)]
pub enum BitstampError {
    // #[error("Order book update send error")]
    // OrderBookUpdateSendError(#[from] SendError<OrderBookUpdate>),
    #[error("Error when sending tungstenite message")]
    MessageSendError(#[from] SendError<tungstenite::Message>),
    #[error("Invalid update id")]
    InvalidUpdateId,
}
