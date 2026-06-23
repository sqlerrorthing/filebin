use bytes::Bytes;
use derive_new::new;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project::pin_project;
use thiserror::Error;

#[pin_project]
#[derive(Debug)]
pub struct LimitedStream<S> {
    #[pin]
    stream: Option<S>,
    max_size: u64,
    processed: u64,
}

impl<S> LimitedStream<S> {
    pub const fn new(stream: S, max_size: u64) -> Self {
        Self {
            stream: Some(stream),
            max_size,
            processed: 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum LimitStreamError<E> {
    #[error("the stream limits is exceeds")]
    LimitExceeds,
    #[error("stream error: {0}")]
    Stream(#[source] E),
}

impl<S, E> Stream for LimitedStream<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, LimitStreamError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let stream = match this.stream.as_mut().as_pin_mut() {
            Some(s) => s,
            None => return Poll::Ready(None),
        };

        stream.poll_next(cx).map(|opt| {
            opt.map(|result| {
                result
                    .map_err(LimitStreamError::Stream)
                    .and_then(|bytes| {
                        *this.processed += bytes.len() as u64;
                        if *this.processed > *this.max_size {
                            this.stream.set(None);
                            Err(LimitStreamError::LimitExceeds)
                        } else {
                            Ok(bytes)
                        }
                    })
            })
        })
    }
}
