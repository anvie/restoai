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

pub struct StreamWriter(pub Arc<mpsc::Sender<String>>);

impl StreamWriter {
    pub async fn write<T: AsRef<str>>(&mut self, msg: T) -> std::io::Result<usize> {
        trace!(
            "writing Stream `{}` with buf len: {}",
            msg.as_ref(),
            msg.as_ref().len()
        );

        self.0
            .send(msg.as_ref().to_string())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(msg.as_ref().len())
    }
}
