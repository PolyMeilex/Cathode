use std::{cell::RefCell, rc::Rc};

use futures::StreamExt;
use libpulse_glib_binding::Mainloop;
use pulse::{
    context::{
        self,
        subscribe::{Facility, InterestMaskSet, Operation},
        FlagSet, State,
    },
    error::PAErr,
    proplist::Proplist,
};

pub mod introspector;
pub mod stream;

use introspector::Introspector;
pub use introspector::*;

pub type ContextRc = Rc<RefCell<pulse::context::Context>>;

pub type SubscribeEvent = Result<(Option<Facility>, Option<Operation>, u32), ()>;

pub struct Inner {
    pub context: pulse::context::Context,
    _mainloop: Mainloop,
}

#[derive(Clone)]
pub struct Context {
    pub inner: Rc<RefCell<Inner>>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("context", &"...")
            .field("mainloop", &"...")
            .finish()
    }
}

impl Context {
    pub fn new_with_proplist(name: &str, proplist: &Proplist) -> Context {
        let mainloop = Mainloop::new(None).expect("Failed to create mainloop");

        let context = context::Context::new_with_proplist(&mainloop, name, proplist)
            .expect("Failed to create new context");

        Self {
            inner: Rc::new(RefCell::new(Inner {
                context,
                _mainloop: mainloop,
            })),
        }
    }

    pub async fn connect(&self, server: Option<&str>, flags: FlagSet) -> Result<(), PAErr> {
        let (mut tx, mut rx) = futures::channel::mpsc::unbounded::<()>();

        self.inner
            .borrow_mut()
            .context
            .set_state_callback(Some(Box::new(move || {
                tx.start_send(()).ok();
            })));

        self.inner
            .borrow_mut()
            .context
            .connect(server, flags, None)?;

        while rx.next().await.is_some() {
            match self.inner.borrow_mut().context.get_state() {
                State::Ready => {
                    return Ok(());
                }
                State::Failed => {
                    return Err(pulse::error::Code::Unknown.into());
                }
                State::Terminated => {
                    return Err(pulse::error::Code::ConnectionTerminated.into());
                }
                _ => {}
            }
        }

        Err(pulse::error::Code::Unknown.into())
    }

    pub fn disconnect(&mut self) {
        self.inner.borrow_mut().context.disconnect();
    }

    pub fn introspect(&self) -> Introspector<'_> {
        Introspector::from(self.inner.borrow().context.introspect())
    }

    pub fn subscribe(&self, mask: InterestMaskSet) -> impl futures::Stream<Item = SubscribeEvent> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<SubscribeEvent>();

        let callback = Box::new({
            let tx = tx.clone();
            move |facility, operation, index| {
                tx.unbounded_send(Ok((facility, operation, index))).ok();
            }
        });

        self.inner
            .borrow_mut()
            .context
            .set_subscribe_callback(Some(callback));

        self.inner
            .borrow_mut()
            .context
            .subscribe(mask, move |success| {
                if !success {
                    tx.unbounded_send(Err(())).ok();
                }
            });

        rx
    }

    pub fn crate_stream(&self, id: u32, stream_id: Option<u32>) -> stream::Stream {
        super::stream::crate_stream(self, id, stream_id)
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.context.set_subscribe_callback(None);
        self.context.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        let props = Proplist::new().unwrap();
        let context = Context::new_with_proplist("Test", &props);

        glib::MainContext::default().block_on(async move {
            context.connect(None, FlagSet::NOFLAGS).await.unwrap();
        });
    }

    #[test]
    #[ignore]
    fn subscribe() {
        let props = Proplist::new().unwrap();
        let context = Context::new_with_proplist("Test", &props);

        glib::MainContext::default().block_on(async move {
            context.connect(None, FlagSet::NOFLAGS).await.unwrap();

            let mut stream = context.subscribe(InterestMaskSet::SINK_INPUT);

            while let Some(event) = stream.next().await {
                let (facility, operation, index) = event.unwrap();
                dbg!(facility, operation, index);
            }
        });
    }
}
