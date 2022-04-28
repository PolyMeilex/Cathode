use std::{cell::RefMut, collections::HashMap};

use adw::gtk;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use pulse::{context::subscribe::Operation, proplist::properties};

use pulse_async::SinkInputInfo;

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
    #[template(file = "playback_page.ui")]
    pub struct PlaybackPage {
        // #[template_child]
        // pub vbox: TemplateChild<gtk::Box>,
        #[template_child]
        pub flow_box: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub input_flow_box: TemplateChild<gtk::FlowBox>,

        pub items: RefCell<HashMap<u32, PlaybackItem>>,
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
    pub struct PlaybackPage(ObjectSubclass<imp::PlaybackPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PlaybackPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn playback_items(&self) -> RefMut<HashMap<u32, PlaybackItem>> {
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

    pub fn add_item(&self, info: &SinkInputInfo) -> PlaybackItem {
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

        let app_name = info.proplist.get_str(properties::APPLICATION_NAME);

        let title = glib::markup_escape_text(app_name.as_deref().unwrap_or(""));
        item.set_title(title.as_str());
        let subtitle = glib::markup_escape_text(info.name.as_deref().unwrap_or("Unknown"));
        item.set_subtitle(subtitle.as_str());

        if let Some(icon) = info.proplist.get_str(properties::APPLICATION_ICON_NAME) {
            item.set_icon(&icon);
        } else {
            let theme = gtk::IconTheme::default();

            if let Some(name) = app_name {
                let name = name.to_lowercase();

                if theme.has_icon(&name) {
                    item.set_icon(&name);
                }
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
