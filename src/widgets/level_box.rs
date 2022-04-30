use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use glib::ObjectExt;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "level_box.ui")]
    pub struct LevelBox {
        pub icon_name: RefCell<String>,

        #[template_child]
        pub level_bar: TemplateChild<gtk::LevelBar>,
        #[template_child]
        pub image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LevelBox {
        const NAME: &'static str = "LevelBox";
        type Type = super::LevelBox;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("levelbox");

            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LevelBox {
        fn constructed(&self, _obj: &Self::Type) {}

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "icon-name",
                    "App Icon",
                    "The app icon",
                    Some("multimedia-player-symbolic"),
                    glib::ParamFlags::READWRITE
                        | glib::ParamFlags::CONSTRUCT
                        | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "icon-name" => obj.set_icon(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon-name" => self.icon_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for LevelBox {}
    impl BuildableImpl for LevelBox {}
}

glib::wrapper! {
    pub struct LevelBox(ObjectSubclass<imp::LevelBox>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LevelBox {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn set_icon(&self, icon_name: &str) {
        *self.imp().icon_name.borrow_mut() = icon_name.to_string();
        self.notify("icon_name");
    }

    pub fn level_bar(&self) -> &gtk::LevelBar {
        &self.imp().level_bar
    }
}
