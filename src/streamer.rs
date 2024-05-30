use parking_lot::Mutex;
use std::sync::Arc;

use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use futures_util::future;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct Streamer {
    inner: Mutex<StreamerInner>,
}

#[derive(Debug, Clone, Default)]
pub struct StreamerInner {
    clients: Vec<mpsc::Sender<sse::Event>>,
}

impl Streamer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(StreamerInner::default()),
        })
    }
    // fn subscribe(&self) -> Sse {
    //     let (tx, rx) = mpsc::channel(1);
    //     self.inner.lock().unwrap().clients.push(tx);
    //     Sse::new(ReceiverStream::new(rx))
    // }
    // fn publish(&self, event: sse::Event) {
    //     let mut inner = self.inner.lock().unwrap();
    //     inner
    //         .clients
    //         .retain(|tx| tx.try_send(event.clone()).is_ok());
    // }
    //
    pub async fn new_client(&self) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
        let (tx, rx) = mpsc::channel(10);

        tx.send(sse::Data::new("connected").into())
            .await
            .expect("Cannot send connected data");

        self.inner.lock().clients.push(tx);

        Sse::from_infallible_receiver(rx)
    }

    pub async fn test_submit(&self, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let _ = future::join_all(
            clients
                .iter()
                .map(|client| client.send(sse::Data::new(msg).into())),
        )
        .await;
    }
}
