use adw::subclass::prelude::*;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::CathodeWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct CathodeApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for CathodeApplication {
        const NAME: &'static str = "CathodeApplication";
        type Type = super::CathodeApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for CathodeApplication {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.about", &["<primary>a"]);
        }
    }

    impl ApplicationImpl for CathodeApplication {
        fn activate(&self, application: &Self::Type) {
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = CathodeWindow::new(application);
                crate::run::run(window.clone());
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for CathodeApplication {}
    impl AdwApplicationImpl for CathodeApplication {}
}

glib::wrapper! {
    pub struct CathodeApplication(ObjectSubclass<imp::CathodeApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CathodeApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
            .expect("Failed to create CathodeApplication")
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name("Cathode")
            .logo_icon_name(&self.application_id().unwrap())
            .version(VERSION)
            .authors(vec!["Bartłomiej Maryńczak".into()])
            .build();

        dialog.present();
    }
}
