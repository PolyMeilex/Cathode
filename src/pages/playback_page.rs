use gtk::prelude::*;

use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use gtk::{subclass::prelude::ObjectSubclassIsExt, CompositeTemplate};

use pulse::context::subscribe::Operation;
use pulse_async::SinkInputInfo;

use crate::widgets::SinkInputItem;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "playback_page.ui")]
    pub struct PlaybackPage {
        #[template_child]
        pub flow_box: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub input_flow_box: TemplateChild<gtk::FlowBox>,

        pub items: RefCell<HashMap<u32, SinkInputItem>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackPage {
        const NAME: &'static str = "PlaybackPage";
        type Type = super::PlaybackPage;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackPage {}
    impl WidgetImpl for PlaybackPage {}
    impl BuildableImpl for PlaybackPage {}
    impl BinImpl for PlaybackPage {}
}

glib::wrapper! {
    pub struct PlaybackPage(ObjectSubclass<imp::PlaybackPage>) @extends gtk::Widget;
}

impl PlaybackPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn playback_items(&self) -> RefMut<HashMap<u32, SinkInputItem>> {
        self.imp().items.borrow_mut()
    }

    pub async fn event(&self, context: &pulse_async::Context, op: &Operation, id: u32) {
        match op {
            Operation::New => {
                if let Ok(info) = context.introspect().sink_input(id).await {
                    self.add_item(&info);
                }
            }
            Operation::Changed => {
                if let Ok(info) = context.introspect().sink_input(id).await {
                    self.add_item(&info);
                }
            }
            Operation::Removed => {
                self.remove_item(id);
            }
        }
    }

    pub fn add_item(&self, info: &SinkInputInfo) -> SinkInputItem {
        let id = info.index;

        let item = {
            let mut items = self.imp().items.borrow_mut();

            if let Some(item) = items.get(&id) {
                item.clone()
            } else {
                let item = SinkInputItem::new();

                self.imp().flow_box.get().append(&item);
                items.insert(id, item.clone());

                item
            }
        };

        item.update(info);

        item
    }

    pub fn remove_item(&self, id: u32) {
        if let Some(item) = self.imp().items.borrow_mut().remove(&id) {
            self.imp().flow_box.remove(&item);
        }
    }
}
