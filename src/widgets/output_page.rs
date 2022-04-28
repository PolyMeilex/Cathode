use std::{borrow::Borrow, cell::RefMut, collections::HashMap};

use adw::gtk;
use gtk::{
    subclass::prelude::ObjectSubclassIsExt,
    traits::{RangeExt, StyleContextExt, WidgetExt},
};
use pulse::{context::subscribe::Operation, volume::Volume};

use pulse_async::SinkInfo;

use super::PlaybackItem;

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use adw::subclass::prelude::*;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::{glib, CompositeTemplate};

    use crate::widgets::PlaybackItem;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "output_page.ui")]
    pub struct OutputPage {
        // #[template_child]
        // pub vbox: TemplateChild<gtk::Box>,
        #[template_child]
        pub flow_box: TemplateChild<gtk::FlowBox>,

        pub items: RefCell<HashMap<u32, PlaybackItem>>,
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
    pub struct OutputPage(ObjectSubclass<imp::OutputPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl OutputPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn playback_items(&self) -> RefMut<HashMap<u32, PlaybackItem>> {
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

    pub fn add_item(&self, info: &SinkInfo) -> PlaybackItem {
        let id = info.index;

        let item = {
            let mut items = self.imp().items.borrow_mut();

            if let Some(item) = items.get(&id) {
                item.clone()
            } else {
                let item = PlaybackItem::new();

                self.imp().flow_box.get().append(&item);
                items.insert(id, item.clone());

                item
            }
        };

        let title = glib::markup_escape_text(
            info.active_port
                .as_ref()
                .unwrap()
                .description
                .as_deref()
                .unwrap_or(""),
        );
        item.set_title(title.as_str());
        let subtitle = glib::markup_escape_text(info.description.as_deref().unwrap_or("Unknown"));
        item.set_subtitle(subtitle.as_str());

        item.set_icon("audio-speakers-symbolic");

        let volume: &[Volume] = info.volume.borrow();
        let volume = (volume[0].0 as f64 / Volume::NORMAL.0 as f64) * 100.0;

        item.channel_scale().scale().set_value(volume);

        match info.state {
            pulse::def::SinkState::Running => {
                item.channel_scale()
                    .scale()
                    .style_context()
                    .remove_class("inactive");
            }
            _ => {
                item.channel_scale()
                    .scale()
                    .style_context()
                    .add_class("inactive");
            }
        }

        item
    }

    pub fn remove_item(&self, id: u32) {
        if let Some(item) = self.imp().items.borrow_mut().remove(&id) {
            self.imp().flow_box.remove(&item);
        }
    }
}
