use std::{cell::RefMut, collections::HashMap};

use adw::gtk;
use glib::{subclass::types::ObjectSubclassExt, Cast};
use gtk::{subclass::prelude::ObjectSubclassIsExt, traits::WidgetExt};
use pulse::{context::subscribe::Operation, proplist::properties};

use pulse_async::SinkInputInfo;

use super::PlaybackItem;
use crate::models::{sink_input::SinkInput, sink_input_model::SinkInputModel};

mod imp {
    use super::*;
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

        pub model: SinkInputModel,
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

    impl ObjectImpl for PlaybackPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.init();
        }
    }
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
        let this: Self = glib::Object::new(&[]).expect("Failed to create ChannelScale");
        this.init();
        this
    }

    fn init(&self) {
        let this = self.imp();
        this.flow_box.bind_model(Some(&this.model), |obj| {
            let data = obj.downcast_ref::<SinkInput>().unwrap();
            let item = PlaybackItem::new();
            item.set_title(&data.title());
            item.set_subtitle(&data.subtitle());
            item.set_icon(&data.icon_name());
            item.upcast::<gtk::Widget>()
        })
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

    pub fn add_item(&self, info: &SinkInputInfo) {
        let sink_input = SinkInput::new(info);
        self.imp().model.add_sink_input(&sink_input);

        // let id = info.index;

        // let item = {
        //     let mut items = self.imp().items.borrow_mut();

        //     if let Some(item) = items.get(&id) {
        //         item.clone()
        //     } else {
        //         let item = PlaybackItem::new();

        //         self.imp().flow_box.get().append(&item);
        //         items.insert(id, item.clone());

        //         item
        //     }
        // };

        // let app_name = info.proplist.get_str(properties::APPLICATION_NAME);

        // let title = glib::markup_escape_text(app_name.as_deref().unwrap_or(""));
        // item.set_title(title.as_str());
        // let subtitle = glib::markup_escape_text(info.name.as_deref().unwrap_or("Unknown"));
        // item.set_subtitle(subtitle.as_str());

        // if let Some(icon) = info.proplist.get_str(properties::APPLICATION_ICON_NAME) {
        //     item.set_icon(&icon);
        // } else {
        //     let theme = gtk::IconTheme::default();

        //     if let Some(name) = app_name {
        //         let name = name.to_lowercase();

        //         if theme.has_icon(&name) {
        //             item.set_icon(&name);
        //         }
        //     }
        // }

        // item
    }

    pub fn remove_item(&self, id: u32) {
        // if let Some(item) = self.imp().items.borrow_mut().remove(&id) {
        //     self.imp().flow_box.remove(&item);
        // }
    }
}
