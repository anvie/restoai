use actix_web::rt::{self, time::interval, Runtime};
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use futures_util::future;
use parking_lot::Mutex;
use std::{io::Write, sync::Arc, time::Duration};
use tokio::io::AsyncWrite;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct StreamChannel {
    pub id: usize,
    pub stream: Sse<InfallibleStream<ReceiverStream<sse::Event>>>,
    stream_inner: Arc<Mutex<StreamerInner>>,
}

pub struct StreamWriter(Arc<Mutex<StreamerInner>>, usize);

unsafe impl Send for StreamWriter {}

impl StreamWriter {
    pub async fn write<T: AsRef<str>>(&mut self, msg: T) -> std::io::Result<usize> {
        // let msg = String::from_utf8_lossy(buf);
        trace!("writing Stream `{}` with buf len: {}", msg.as_ref(), msg.as_ref().len());

        let stream_inner = self.0.clone();
        let clients = stream_inner.lock().clients.clone();
        if let Some(client) = clients.get(self.1) {
            client
                .send(sse::Data::new(msg.as_ref().to_string()).into())
                .await
                .expect("Cannot send data");
        }

        Ok(msg.as_ref().len())
    }
}

impl Drop for StreamWriter {
    fn drop(&mut self) {
        trace!("Dropping StreamWriter");
        let stream_inner = self.0.clone();
        let idx = self.1;

        let inner = stream_inner.lock();
        let clients = inner.clients.clone();
        let tx = clients.get(idx).expect("get tx client out of bound").clone();
        tokio::spawn(async move {
            tx.send(sse::Data::new("done.").into())
                .await
                .expect("cannot send disconnected msg");
        });
    }
}

impl StreamChannel {
    pub fn get_stream_writer(&self) -> StreamWriter {
        let stream_inner = self.stream_inner.clone();
        StreamWriter(stream_inner, self.id)
    }

    // pub fn get_stream_inner(&self, idx: usize) -> Option<&mpsc::Sender<sse::Event>> {
    //     self.stream_inner.lock().clients.get(idx)
    // }
}

pub struct Streamer {
    inner: Arc<Mutex<StreamerInner>>,
}

#[derive(Debug, Clone, Default)]
pub struct StreamerInner {
    clients: Vec<Arc<mpsc::Sender<sse::Event>>>,
}

unsafe impl Send for StreamerInner {}

impl Streamer {
    pub fn new() -> Arc<Self> {
        let this = Arc::new(Self {
            inner: Arc::new(Mutex::new(StreamerInner::default())),
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

    pub async fn new_client(&self) -> StreamChannel {
        let (tx, rx) = mpsc::channel(10);

        tx.send(sse::Data::new("connected").into())
            .await
            .expect("Cannot send connected data");

        let idx = self.inner.lock().clients.len();
        self.inner.lock().clients.push(Arc::new(tx));

        StreamChannel {
            id: idx,
            stream: Sse::from_infallible_receiver(rx),
            stream_inner: self.inner.clone(),
        }
    }

    pub async fn broadcast(&self, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let _ = future::join_all(clients.iter().map(|client| client.send(sse::Data::new(msg).into()))).await;
    }

    #[allow(dead_code)]
    pub async fn send_to(&self, msg: &str, index: usize) {
        let clients = self.inner.lock().clients.clone();
        if let Some(client) = clients.get(index) {
            let _ = client.send(sse::Data::new(msg).into()).await;
        }
    }
}
