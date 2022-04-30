use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use pulse::volume::Volume;
use pulse_async::SinkInfo;

use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::{borrow::Borrow, cell::RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "sink_item.ui")]
    pub struct SinkItem {
        #[template_child]
        pub channel_scale: TemplateChild<crate::widgets::ChannelScale>,
        #[template_child]
        pub level_box: TemplateChild<crate::widgets::LevelBox>,

        pub title: RefCell<String>,
        pub subtitle: RefCell<String>,
        pub icon_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SinkItem {
        const NAME: &'static str = "SinkItem";
        type Type = super::SinkItem;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SinkItem {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "title",
                        "Title",
                        "The pulseaudio sink name",
                        Some("Unknown"),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "subtitle",
                        "Subtitle",
                        "The pulseaudio sink name",
                        Some("Unknown"),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
                        "icon-name",
                        "App Icon",
                        "The app icon",
                        Some("audio-speakers-symbolic"),
                        glib::ParamFlags::READWRITE
                            | glib::ParamFlags::CONSTRUCT
                            | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "title" => obj.set_title(value.get().unwrap()),
                "subtitle" => obj.set_subtitle(value.get().unwrap()),
                "icon-name" => obj.set_icon(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "subtitle" => self.subtitle.borrow().to_value(),
                "icon-name" => self.icon_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SinkItem {}
    impl BuildableImpl for SinkItem {}
    impl BinImpl for SinkItem {}
}

glib::wrapper! {
    pub struct SinkItem(ObjectSubclass<imp::SinkItem>) @extends gtk::Widget;
}

impl SinkItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn update(&self, info: &SinkInfo) {
        let title = glib::markup_escape_text(
            info.active_port
                .as_ref()
                .unwrap()
                .description
                .as_deref()
                .unwrap_or(""),
        );

        self.set_title(title.as_str());
        let subtitle = glib::markup_escape_text(info.description.as_deref().unwrap_or("Unknown"));
        self.set_subtitle(subtitle.as_str());

        self.set_icon("audio-speakers-symbolic");

        let volume: &[Volume] = info.volume.borrow();
        let volume = (volume[0].0 as f64 / Volume::NORMAL.0 as f64) * 100.0;

        self.channel_scale().scale().set_value(volume);

        match info.state {
            pulse::def::SinkState::Running => {
                self.channel_scale()
                    .scale()
                    .style_context()
                    .remove_class("inactive");
            }
            _ => {
                self.channel_scale()
                    .scale()
                    .style_context()
                    .add_class("inactive");
            }
        }
    }

    pub fn channel_scale(&self) -> &crate::widgets::ChannelScale {
        &self.imp().channel_scale
    }

    pub fn level_bar(&self) -> &gtk::LevelBar {
        &self.imp().level_box.level_bar()
    }

    pub fn set_title(&self, title: &str) {
        *self.imp().title.borrow_mut() = title.to_string();
        self.notify("title");
    }

    pub fn set_subtitle(&self, title: &str) {
        *self.imp().subtitle.borrow_mut() = title.to_string();
        self.notify("subtitle");
    }

    pub fn set_icon(&self, icon: &str) {
        *self.imp().icon_name.borrow_mut() = icon.to_string();
        self.notify("icon-name");
    }

    pub fn connect_volume_changed<F>(&self, cb: F)
    where
        F: Fn(&gtk::Scale, Box<dyn FnOnce()>) + 'static,
    {
        self.imp().channel_scale.get().connect_volume_changed(cb);
    }
}
