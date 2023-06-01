pub mod error;

use futures::Stream;
use futures::StreamExt;
use orderbook_service::{Empty, Level, Summary};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{
    broadcast::{Receiver, Sender},
    mpsc,
};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tonic::transport::server::Router;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};

use self::error::ServerError;

pub mod orderbook_service {

    tonic::include_proto!("orderbookservice");
}

pub fn spawn_order_book_aggregator_service(
    router: Router,
    socket_address: SocketAddr,
) -> JoinHandle<Result<(), ServerError>> {
    let handle = tokio::spawn(async move {
        router.serve(socket_address).await?;
        Ok::<_, ServerError>(())
    });

    handle
}

#[derive(Debug)]
pub struct OrderbookAggregatorService {
    summary_rx: Receiver<Summary>,
}

impl OrderbookAggregatorService {
    pub fn new(summary_buffer: usize) -> (Self, Sender<Summary>) {
        // Create a broadcast channel with a predefined buffer size (summary_buffer).
        // If a receiver is slow and the buffer gets full, the oldest unprocessed message is discarded.
        // If a slow receiver tries to receive this discarded message, it gets a RecvError::Lagged error instead.
        // This error updates the receiver's position to the oldest message still in the buffer.
        let (summary_tx, summary_rx) = tokio::sync::broadcast::channel(summary_buffer);
        (OrderbookAggregatorService { summary_rx }, summary_tx)
    }
}

#[tonic::async_trait]
impl orderbook_service::orderbook_aggregator_server::OrderbookAggregator
    for OrderbookAggregatorService
{
    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync + 'static>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let rx = self.summary_rx.resubscribe();

        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).map(|summary| match summary {
                Ok(summary) => Ok(summary),
                Err(e) => match e {
                    BroadcastStreamRecvError::Lagged(_) => {
                        Err(Status::internal("Stream lagged too far behind"))
                    }
                },
            });

        Ok(Response::new(Box::pin(stream)))
    }
}
