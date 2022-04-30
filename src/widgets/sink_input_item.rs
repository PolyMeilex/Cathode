use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use pulse::proplist::properties;
use pulse_async::SinkInputInfo;

use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "sink_input_item.ui")]
    pub struct SinkInputItem {
        #[template_child]
        pub channel_scale: TemplateChild<crate::widgets::ChannelScale>,
        #[template_child]
        pub level_box: TemplateChild<crate::widgets::LevelBox>,

        pub title: RefCell<String>,
        pub subtitle: RefCell<String>,
        pub icon_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SinkInputItem {
        const NAME: &'static str = "SinkInputItem";
        type Type = super::SinkInputItem;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SinkInputItem {
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
    impl WidgetImpl for SinkInputItem {}
    impl BuildableImpl for SinkInputItem {}
    impl BinImpl for SinkInputItem {}
}

glib::wrapper! {
    pub struct SinkInputItem(ObjectSubclass<imp::SinkInputItem>) @extends gtk::Widget;
}

impl SinkInputItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn update(&self, info: &SinkInputInfo) {
        let app_name = info.proplist.get_str(properties::APPLICATION_NAME);

        if let Some(title) = app_name.as_deref() {
            let title = glib::markup_escape_text(title);
            self.set_title(title.as_str());
        }

        if let Some(subtitle) = info.name.as_deref() {
            let subtitle = glib::markup_escape_text(subtitle);
            self.set_subtitle(&subtitle);
        }

        let theme = gtk::IconTheme::default();
        let icon_name = info
            .proplist
            .get_str(properties::APPLICATION_ICON_NAME)
            .and_then(|icon| theme.has_icon(&icon).then(|| icon))
            .or_else(|| {
                let icon = info.proplist.get_str(properties::APPLICATION_ID)?;
                theme.has_icon(&icon).then(|| icon)
            })
            .or_else(|| {
                let icon = app_name?.to_lowercase();
                theme.has_icon(&icon).then(|| icon)
            });

        if let Some(icon_name) = icon_name {
            self.set_icon(&icon_name);
        }

        if !info.corked && !info.mute {
            self.channel_scale()
                .scale()
                .style_context()
                .remove_class("inactive");
        } else {
            self.channel_scale()
                .scale()
                .style_context()
                .add_class("inactive");
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
