use std::cell::RefCell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use pulse::proplist::properties;
use pulse_async::SinkInputInfo;

mod imp {
    use once_cell::unsync::OnceCell;

    use super::*;

    #[derive(Debug, Default)]
    pub struct SinkInput {
        pub index: OnceCell<u32>,
        pub title: RefCell<String>,
        pub subtitle: RefCell<String>,
        pub icon_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SinkInput {
        const NAME: &'static str = "SinkInput";
        type Type = super::SinkInput;
    }

    impl ObjectImpl for SinkInput {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecUInt::new(
                        "index",
                        "Index",
                        "Index",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
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
                "index" => obj.set_index(value.get().unwrap()),
                "title" => obj.set_title(value.get().unwrap()),
                "subtitle" => obj.set_subtitle(value.get().unwrap()),
                "icon-name" => obj.set_icon_name(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "index" => obj.index().to_value(),
                "title" => obj.title().to_value(),
                "subtitle" => obj.subtitle().to_value(),
                "icon-name" => obj.icon_name().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SinkInput(ObjectSubclass<imp::SinkInput>);
}

impl SinkInput {
    pub fn new(info: &SinkInputInfo) -> Self {
        let app_name = info.proplist.get_str(properties::APPLICATION_NAME);

        let title = app_name
            .as_deref()
            .map(|name| glib::markup_escape_text(name).to_string());

        let subtitle = info
            .name
            .as_deref()
            .map(|name| glib::markup_escape_text(name).to_string());

        let icon_name = if let Some(icon) = info.proplist.get_str(properties::APPLICATION_ICON_NAME)
        {
            Some(icon)
        } else {
            let theme = gtk::IconTheme::default();

            app_name.and_then(|name| {
                let name = name.to_lowercase();

                if theme.has_icon(&name) {
                    Some(name)
                } else {
                    None
                }
            })
        };

        let mut poperties: Vec<(&str, &dyn ToValue)> = Vec::new();

        poperties.push(("index", &info.index));

        if let Some(title) = title.as_ref() {
            poperties.push(("title", title));
        }

        if let Some(subtitle) = subtitle.as_ref() {
            poperties.push(("subtitle", subtitle));
        }

        if let Some(icon_name) = icon_name.as_ref() {
            poperties.push(("icon-name", icon_name));
        }

        glib::Object::new(&poperties).unwrap()
    }

    pub fn index(&self) -> u32 {
        *self.imp().index.get().unwrap()
    }

    fn set_index(&self, index: u32) {
        self.imp().index.set(index).unwrap();
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn set_title(&self, title: &str) {
        *self.imp().title.borrow_mut() = title.to_string();
        self.notify("title");
    }

    pub fn subtitle(&self) -> String {
        self.imp().subtitle.borrow().clone()
    }

    pub fn set_subtitle(&self, title: &str) {
        *self.imp().subtitle.borrow_mut() = title.to_string();
        self.notify("subtitle");
    }

    pub fn icon_name(&self) -> String {
        self.imp().icon_name.borrow().clone()
    }

    pub fn set_icon_name(&self, icon: &str) {
        let icon = if gtk::IconTheme::default().has_icon(icon) {
            icon
        } else {
            "audio-speakers-symbolic"
        };

        *self.imp().icon_name.borrow_mut() = icon.to_string();

        self.notify("icon-name");
    }
}
