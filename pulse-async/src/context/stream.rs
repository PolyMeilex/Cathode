use futures::{channel::mpsc::UnboundedReceiver, pin_mut};
use pulse::def::BufferAttr;

use crate::Context;

pub fn crate_stream(context: &Context, id: u32, stream_id: Option<u32>) -> Stream {
    let mut stream = pulse::stream::Stream::new(
        &mut context.inner.borrow_mut().context,
        "Stream Monitor",
        &pulse::sample::Spec {
            format: pulse::sample::Format::F32le,
            rate: 25,
            channels: 1,
        },
        None,
    )
    .unwrap();

    let (tx, rx) = futures::channel::mpsc::unbounded::<usize>();

    stream.set_read_callback(Some(Box::new(move |len| {
        tx.unbounded_send(len).ok();
    })));

    if let Some(stream_id) = stream_id {
        stream.set_monitor_stream(stream_id).ok();
    }

    stream
        .connect_record(
            Some(&format!("{}", id)),
            Some(&BufferAttr {
                fragsize: std::mem::size_of::<f32>() as u32,
                maxlength: u32::MAX,
                ..Default::default()
            }),
            pulse::stream::FlagSet::DONT_MOVE
                | pulse::stream::FlagSet::PEAK_DETECT
                | pulse::stream::FlagSet::ADJUST_LATENCY
                | pulse::stream::FlagSet::DONT_INHIBIT_AUTO_SUSPEND,
        )
        .unwrap();

    Stream { rx, stream }
}

pub struct Stream {
    rx: UnboundedReceiver<usize>,
    stream: pulse::stream::Stream,
}

impl futures::Stream for Stream {
    type Item = f32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let rx = &mut self.rx;
        pin_mut!(rx);
        let pool = futures::Stream::poll_next(rx, cx);

        match pool {
            std::task::Poll::Ready(len) => {
                if let Some(len) = len {
                    let data = self.stream.peek().unwrap();

                    let v = match data {
                        pulse::stream::PeekResult::Empty => None,
                        pulse::stream::PeekResult::Hole(_) => None,
                        pulse::stream::PeekResult::Data(data) => {
                            let bytes: [u8; 4] = data.try_into().unwrap();
                            let v = f32::from_le_bytes(bytes);
                            Some(v)
                        }
                    };

                    if len != 0 {
                        self.stream.discard().unwrap();
                    }

                    if let Some(v) = v {
                        std::task::Poll::Ready(Some(v))
                    } else {
                        std::task::Poll::Pending
                    }
                } else {
                    std::task::Poll::Ready(None)
                }
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stream.set_read_callback(None);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
