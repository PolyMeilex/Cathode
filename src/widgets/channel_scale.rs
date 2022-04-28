use adw::gtk;
use gtk::{glib, subclass::prelude::ObjectSubclassIsExt};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::{glib, CompositeTemplate};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "channel_scale.ui")]
    pub struct ChannelScale {
        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelScale {
        const NAME: &'static str = "ChannelScale";
        type Type = super::ChannelScale;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Buildable,);

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChannelScale {
        fn constructed(&self, obj: &Self::Type) {
            self.scale
                .add_mark(0.0, gtk::PositionType::Bottom, Some("0%"));
            self.scale
                .add_mark(100.0, gtk::PositionType::Bottom, Some("100%"));

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ChannelScale {}
    impl BuildableImpl for ChannelScale {}
    impl BinImpl for ChannelScale {}
}

glib::wrapper! {
    pub struct ChannelScale(ObjectSubclass<imp::ChannelScale>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChannelScale {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ChannelScale")
    }

    pub fn scale(&self) -> &gtk::Scale {
        &self.imp().scale
    }
}
