use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

mod imp {
    use adw::subclass::prelude::AdwApplicationWindowImpl;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/polymeilex/Cathode/window.ui")]
    pub struct CathodeWindow {
        #[template_child]
        pub playback_page: TemplateChild<crate::widgets::PlaybackPage>,

        #[template_child]
        pub output_page: TemplateChild<crate::widgets::OutputPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CathodeWindow {
        const NAME: &'static str = "CathodeWindow";
        type Type = super::CathodeWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CathodeWindow {}
    impl WidgetImpl for CathodeWindow {}
    impl WindowImpl for CathodeWindow {}
    impl ApplicationWindowImpl for CathodeWindow {}
    impl AdwApplicationWindowImpl for CathodeWindow {}
}

glib::wrapper! {
    pub struct CathodeWindow(ObjectSubclass<imp::CathodeWindow>)
        @extends gtk::Widget, gtk::Window, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CathodeWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)]).expect("Failed to create CathodeWindow")
    }

    pub fn playback_page(&self) -> &crate::widgets::PlaybackPage {
        &self.imp().playback_page
    }

    pub fn output_page(&self) -> &crate::widgets::OutputPage {
        &self.imp().output_page
    }
}
