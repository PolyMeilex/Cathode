use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ::pulse::context::FlagSet;
use ::pulse::proplist::Proplist;
use adw::prelude::*;
use futures::StreamExt;
use pulse::context::subscribe::Facility;
use pulse::context::subscribe::InterestMaskSet;
use pulse::context::subscribe::Operation;
use pulse::def::BufferAttr;

use crate::window::CathodeWindow;

fn crate_stream(
    context: &mut pulse::context::Context,
    id: u32,
    stream_id: Option<u32>,
    level_bar: gtk::LevelBar,
) {
    let stream = pulse::stream::Stream::new(
        context,
        "Stream Monitor",
        &pulse::sample::Spec {
            format: pulse::sample::Format::F32le,
            rate: 25,
            channels: 1,
        },
        None,
    )
    .unwrap();

    let stream = Rc::new(RefCell::new(stream));

    stream.borrow_mut().set_read_callback(Some(Box::new({
        let stream = stream.clone();
        let mut last = 0.0;
        move |len| {
            let mut stream = stream.borrow_mut();

            let data = stream.peek().unwrap();

            match data {
                pulse::stream::PeekResult::Empty => todo!(),
                pulse::stream::PeekResult::Hole(_) => todo!(),
                pulse::stream::PeekResult::Data(data) => {
                    let bytes: [u8; 4] = data.try_into().unwrap();
                    let v = f32::from_le_bytes(bytes);

                    let mut v = v as f64;

                    // Big thanks to pavu for this block of code <3
                    const DECAY_STEP: f64 = 0.04;
                    if last >= DECAY_STEP {
                        if v < last - DECAY_STEP {
                            v = last - DECAY_STEP
                        }
                    }

                    last = v;

                    level_bar.set_value(v * 10.0);
                }
            }

            if len != 0 {
                stream.discard().unwrap();
            }
        }
    })));

    stream
        .borrow_mut()
        .set_suspended_callback(Some(Box::new(|| {
            dbg!("suspended_callback");
        })));

    if let Some(stream_id) = stream_id {
        stream.borrow_mut().set_monitor_stream(stream_id).unwrap();
    }

    stream
        .borrow_mut()
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
}

pub fn run(win: &CathodeWindow) {
    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(
            pulse::proplist::properties::APPLICATION_ID,
            "com.github.polymelex.cathode",
        )
        .unwrap();

    let mut context = pulse_async::context::Context::new_with_proplist("Cathode", &proplist);

    let playback_page = win.playback_page().clone();
    let output_page = win.output_page().clone();

    glib::MainContext::default().spawn_local(async move {
        context.connect(None, FlagSet::NOFAIL).await.unwrap();

        enum Event {
            Subscription {
                facility: Option<Facility>,
                operation: Option<Operation>,
                id: u32,
            },
            SetSinkVolume {
                id: u32,
                volume: f64,
                done_notify: Box<dyn FnOnce()>,
            },
            SetSinkInputVolume {
                id: u32,
                volume: f64,
                done_notify: Box<dyn FnOnce()>,
            },
            Error,
        }

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<Event>();

        let sink_input_list = context
            .introspect()
            .sink_input_list()
            .await
            .unwrap_or_else(|_| Vec::new());

        for input in sink_input_list {
            let id = input.index;
            let item = playback_page.add_item(&input);

            let tx = tx.clone();
            // item.connect_volume_changed(move |scale, done_notify| {
            //     let volume = scale.value();

            //     tx.unbounded_send(Event::SetSinkInputVolume {
            //         id,
            //         volume,
            //         done_notify,
            //     })
            //     .ok();
            // });

            // let mut context = context.context.borrow_mut();

            // crate_stream(&mut context, input.sink, Some(id), item.level_bar().clone());
        }

        let sink_list = context
            .introspect()
            .sink_list()
            .await
            .unwrap_or_else(|_| Vec::new());

        for output in sink_list {
            let id = output.index;
            let item = output_page.add_item(&output);

            let tx = tx.clone();
            item.connect_volume_changed(move |scale, done_notify| {
                let volume = scale.value();

                tx.unbounded_send(Event::SetSinkVolume {
                    id,
                    volume,
                    done_notify,
                })
                .ok();
            });

            let mut inner = context.inner.borrow_mut();
            let mut context = &mut inner.context;

            crate_stream(&mut context, id, None, item.level_bar().clone());
        }

        let mut sub = context
            .subscribe(InterestMaskSet::SINK_INPUT | InterestMaskSet::SINK)
            .map(|sub| {
                if let Ok((facility, operation, id)) = sub {
                    Event::Subscription {
                        facility,
                        operation,
                        id,
                    }
                } else {
                    Event::Error
                }
            });

        while let Some(event) = futures::stream::select(&mut sub, &mut rx).next().await {
            match event {
                Event::Subscription {
                    facility,
                    operation,
                    id,
                } => match facility {
                    Some(Facility::SinkInput) => {
                        if let Some(ref op) = operation {
                            playback_page.event(&context, op, id).await;
                        }
                    }
                    Some(Facility::Sink) => {
                        if let Some(ref op) = operation {
                            output_page.event(&context, op, id).await;
                        }
                    }
                    _ => {}
                },
                Event::SetSinkVolume {
                    id,
                    volume,
                    done_notify,
                } => {
                    let _success = context.introspect().set_sink_volume(id, volume).await;

                    glib::timeout_add_local_once(Duration::from_millis(100), move || {
                        done_notify();
                    });
                }
                Event::SetSinkInputVolume {
                    id,
                    volume,
                    done_notify,
                } => {
                    let _success = context.introspect().set_sink_input_volume(id, volume).await;

                    glib::timeout_add_local_once(Duration::from_millis(100), move || {
                        done_notify();
                    });
                }
                Event::Error => todo!(),
            }
        }
    });
}
