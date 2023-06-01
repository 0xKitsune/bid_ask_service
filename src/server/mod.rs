use futures::Stream;
use futures::StreamExt;
use orderbook_service::{Empty, Level, Summary};
use tokio::sync::broadcast::error::RecvError;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use std::{
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
};
use tokio::sync::{
    broadcast::{Receiver, Sender},
    mpsc,
};
use tonic::{Request, Response, Status, Streaming};

pub mod orderbook_service {

    tonic::include_proto!("orderbookservice");
}

pub struct OrderbookAggregatorService {
    clients_connected: AtomicU32,
    summary_rx: Receiver<Summary>,
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
        self.clients_connected.fetch_add(1, Ordering::Relaxed);

        let stream =
            tokio_stream::wrappers::BroadcastStream::new(rx).map(|summary| match summary {
                Ok(summary) => Ok(summary),
                Err(e) => match e {
                    BroadcastStreamRecvError::Closed => {
                        //TODO: log error and cleanup
                        self.clients_connected.fetch_sub(1, Ordering::Relaxed);
                        Err(Status::internal("Client disconnected"))
                    }
                    RecvError::Lagged(_) => Err(Status::internal("Client lagged behind")),
                },
            });

        Ok(Response::new(Box::pin(stream)))
    }
}
