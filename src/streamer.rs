use actix_web::rt::time::interval;
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use futures_util::future;
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
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
        let this = Arc::new(Self {
            inner: Mutex::new(StreamerInner::default()),
        });

        Streamer::start_stale_monitor(this.clone());

        this
    }

    fn start_stale_monitor(this: Arc<Self>) {
        trace!("Starting stale monitor");

        let this = this.clone();

        // Removes all clients that are disconnected or not responsive.

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                this.scan_stale_clients().await;
            }
        });
    }

    async fn scan_stale_clients(&self) {
        let connected_clients = self.inner.lock().clients.clone();
        let mut live_clients = Vec::new();
        let mut stale_clients = 0;
        for client in connected_clients {
            if client.send(sse::Data::new("ping").into()).await.is_ok() {
                live_clients.push(client.clone());
            } else {
                stale_clients += 1;
            }
        }
        if stale_clients > 0 {
            info!("Removed {} stale clients", stale_clients);
        }

        self.inner.lock().clients = live_clients;
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

    pub async fn broadcast(&self, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let _ = future::join_all(
            clients
                .iter()
                .map(|client| client.send(sse::Data::new(msg).into())),
        )
        .await;
    }
}
