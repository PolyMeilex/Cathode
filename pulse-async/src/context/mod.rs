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

mod introspector;
use introspector::Introspector;
pub use introspector::*;

pub type ContextRc = Rc<RefCell<pulse::context::Context>>;

pub type SubscribeEvent = Result<(Option<Facility>, Option<Operation>, u32), ()>;

pub struct Context {
    pub context: ContextRc,
    _mainloop: Mainloop,
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

        let context = Rc::new(RefCell::new(
            context::Context::new_with_proplist(&mainloop, name, proplist)
                .expect("Failed to create new context"),
        ));

        Context {
            context,
            _mainloop: mainloop,
        }
    }

    pub async fn connect(&mut self, server: Option<&str>, flags: FlagSet) -> Result<(), PAErr> {
        let (mut tx, mut rx) = futures::channel::mpsc::unbounded::<()>();

        self.context
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                tx.start_send(()).ok();
            })));

        self.context.borrow_mut().connect(server, flags, None)?;

        while rx.next().await.is_some() {
            match self.context.borrow_mut().get_state() {
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
        self.context.borrow_mut().disconnect();
    }

    pub fn introspect(&self) -> Introspector<'_> {
        Introspector::from(self.context.borrow().introspect())
    }

    pub fn subscribe(
        &mut self,
        mask: InterestMaskSet,
    ) -> impl futures::Stream<Item = SubscribeEvent> {
        let (mut tx, rx) = futures::channel::mpsc::unbounded::<SubscribeEvent>();

        let callback = Box::new({
            let mut tx = tx.clone();
            move |facility, operation, index| {
                tx.start_send(Ok((facility, operation, index))).ok();
            }
        });

        self.context
            .borrow_mut()
            .set_subscribe_callback(Some(callback));

        self.context.borrow_mut().subscribe(mask, move |success| {
            if !success {
                tx.start_send(Err(())).ok();
            }
        });

        rx
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        let props = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist("Test", &props);

        glib::MainContext::default().block_on(async move {
            context.connect(None, FlagSet::NOFLAGS).await.unwrap();
        });
    }

    #[test]
    #[ignore]
    fn subscribe() {
        let props = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist("Test", &props);

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
