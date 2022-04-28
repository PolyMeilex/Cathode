use std::cell::Cell;
use std::rc::Rc;

use adw::gtk;
use adw::prelude::*;
use gtk::{glib, subclass::prelude::ObjectSubclassIsExt};

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::{glib, CompositeTemplate};
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "playback_item.ui")]
    pub struct PlaybackItem {
        #[template_child]
        pub channel_scale: TemplateChild<crate::widgets::ChannelScale>,
        #[template_child]
        pub level_box: TemplateChild<crate::widgets::LevelBox>,

        pub title: RefCell<String>,
        pub subtitle: RefCell<String>,
        pub icon_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackItem {
        const NAME: &'static str = "PlaybackItem";
        type Type = super::PlaybackItem;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackItem {
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
                        Some("multimedia-player-symbolic"),
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
    impl WidgetImpl for PlaybackItem {}
    impl BuildableImpl for PlaybackItem {}
    impl BinImpl for PlaybackItem {}
}

glib::wrapper! {
    pub struct PlaybackItem(ObjectSubclass<imp::PlaybackItem>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PlaybackItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
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
        let icon = if gtk::IconTheme::default().has_icon(icon) {
            icon
        } else {
            "audio-speakers-symbolic"
        };

        *self.imp().icon_name.borrow_mut() = icon.to_string();

        self.notify("icon-name");
    }

    pub fn connect_volume_changed<F>(&self, cb: F)
    where
        F: Fn(&gtk::Scale, Box<dyn FnOnce()>) + 'static,
    {
        let acked = Rc::new(Cell::new(true));
        self.imp()
            .channel_scale
            .scale()
            .connect_value_changed(move |scale| {
                if acked.get() {
                    acked.set(false);

                    let acked = acked.clone();
                    let done_notify = move || {
                        acked.set(true);
                    };

                    cb(scale, Box::new(done_notify))
                }
            });
    }
}
