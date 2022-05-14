use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ::pulse::context::FlagSet;
use ::pulse::proplist::Proplist;
use adw::prelude::*;
use futures::channel::mpsc::UnboundedSender;
use futures::StreamExt;
use pulse::context::subscribe::Facility;
use pulse::context::subscribe::InterestMaskSet;
use pulse::context::subscribe::Operation;
use pulse_async::SinkInputInfo;

use crate::window::CathodeWindow;

struct VolumeUpdateEvent {
    id: u32,
    volume: f64,
    target: Target,
    done_notify: Box<dyn FnOnce()>,
}

enum Target {
    SinkVolume,
    SinkInputVolume,
}

pub fn run(win: CathodeWindow) {
    glib::MainContext::default().spawn_local(async move {
        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(
                pulse::proplist::properties::APPLICATION_ID,
                "com.github.polymelex.cathode",
            )
            .unwrap();

        let context = pulse_async::context::Context::new_with_proplist("Cathode", &proplist);
        win.init_context(context.clone());

        let win = win.clone();

        context.connect(None, FlagSet::NOFAIL).await.unwrap();

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<VolumeUpdateEvent>();

        let sink_input_list = context
            .introspect()
            .sink_input_list()
            .await
            .unwrap_or_else(|_| Vec::new());

        for input in sink_input_list {
            new_sink_input(&win, &tx, &input);
        }

        let sink_list = context
            .introspect()
            .sink_list()
            .await
            .unwrap_or_else(|_| Vec::new());

        for output in sink_list {
            let id = output.index;
            let item = win.output_page().add_item(&output);

            let tx = tx.clone();
            item.connect_volume_changed(move |scale, done_notify| {
                let volume = scale.value();

                tx.unbounded_send(VolumeUpdateEvent {
                    id,
                    volume,
                    done_notify,
                    target: Target::SinkVolume,
                })
                .ok();
            });

            item.level_box().init_stream(context.clone(), id, None);
        }

        glib::MainContext::default().spawn_local(subscribe(win.clone(), tx.clone()));

        while let Some(event) = rx.next().await {
            let mut introspect = context.introspect();
            let id = event.id;
            let volume = event.volume;

            let _success = match event.target {
                Target::SinkVolume => introspect.set_sink_volume(id, volume).await,
                Target::SinkInputVolume => introspect.set_sink_input_volume(id, volume).await,
            };

            glib::timeout_add_local_once(Duration::from_millis(100), move || {
                (event.done_notify)();
            });
        }
    });
}

async fn subscribe(win: CathodeWindow, tx: UnboundedSender<VolumeUpdateEvent>) {
    let context = win.context();
    let mut sub = context.subscribe(InterestMaskSet::SINK_INPUT | InterestMaskSet::SINK);

    let playback_page = win.playback_page().clone();
    let output_page = win.output_page().clone();

    while let Some(event) = sub.next().await {
        if let Ok((facility, operation, id)) = event {
            match facility {
                Some(Facility::SinkInput) => {
                    if let Some(ref op) = operation {
                        match op {
                            Operation::New => {
                                if let Ok(info) = context.introspect().sink_input(id).await {
                                    playback_page.add_item(&info);
                                    new_sink_input(&win, &tx, &info);
                                }
                            }
                            Operation::Changed => {
                                if let Ok(info) = context.introspect().sink_input(id).await {
                                    playback_page.add_item(&info);
                                }
                            }
                            Operation::Removed => {
                                playback_page.remove_item(id);
                            }
                        }
                    }
                }
                Some(Facility::Sink) => {
                    if let Some(ref op) = operation {
                        output_page.event(&context, op, id).await;
                    }
                }
                _ => {}
            };
        }
    }
}

fn new_sink_input(
    win: &CathodeWindow,
    tx: &UnboundedSender<VolumeUpdateEvent>,
    input: &SinkInputInfo,
) {
    let id = input.index;
    let item = win.playback_page().add_item(&input);

    let tx = tx.clone();
    item.connect_volume_changed(move |scale, done_notify| {
        let volume = scale.value();

        tx.unbounded_send(VolumeUpdateEvent {
            id,
            volume,
            done_notify,
            target: Target::SinkInputVolume,
        })
        .ok();
    });

    item.level_box()
        .init_stream(win.context().clone(), input.sink, Some(id));
}

pub struct StreamGuard {
    stream: Rc<RefCell<pulse::stream::Stream>>,
}

impl Drop for StreamGuard {
    fn drop(&mut self) {
        self.stream.borrow_mut().set_read_callback(None);
    }
}
