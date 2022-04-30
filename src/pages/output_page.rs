use gtk::prelude::*;

use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use gtk::{subclass::prelude::ObjectSubclassIsExt, CompositeTemplate};

use pulse::context::subscribe::Operation;
use pulse_async::SinkInfo;

use crate::widgets::SinkItem;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "output_page.ui")]
    pub struct OutputPage {
        #[template_child]
        pub flow_box: TemplateChild<gtk::FlowBox>,

        pub items: RefCell<HashMap<u32, SinkItem>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OutputPage {
        const NAME: &'static str = "OutputPage";
        type Type = super::OutputPage;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OutputPage {}
    impl WidgetImpl for OutputPage {}
    impl BuildableImpl for OutputPage {}
    impl BinImpl for OutputPage {}
}

glib::wrapper! {
    pub struct OutputPage(ObjectSubclass<imp::OutputPage>) @extends gtk::Widget;
}

impl OutputPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn playback_items(&self) -> RefMut<HashMap<u32, SinkItem>> {
        self.imp().items.borrow_mut()
    }

    pub async fn event(&self, context: &pulse_async::Context, op: &Operation, id: u32) {
        match op {
            Operation::New => {
                if let Ok(info) = context.introspect().sink(id).await {
                    self.add_item(&info);
                }
            }
            Operation::Changed => {
                if let Ok(info) = context.introspect().sink(id).await {
                    self.add_item(&info);
                }
            }
            Operation::Removed => {
                self.remove_item(id);
            }
        }
    }

    pub fn add_item(&self, info: &SinkInfo) -> SinkItem {
        let id = info.index;

        let item = {
            let mut items = self.imp().items.borrow_mut();

            if let Some(item) = items.get(&id) {
                item.clone()
            } else {
                let item = SinkItem::new();

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
