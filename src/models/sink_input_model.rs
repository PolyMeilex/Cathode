use std::cell::RefCell;

use crate::models::sink_input::SinkInput;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::IndexMap;
use log::warn;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SinkInputModel {
        pub map: RefCell<IndexMap<u32, SinkInput>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SinkInputModel {
        const NAME: &'static str = "SwStationModel";
        type Type = super::SinkInputModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SinkInputModel {}

    impl ListModelImpl for SinkInputModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            SinkInput::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.map.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.map
                .borrow()
                .get_index(position as usize)
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SinkInputModel(ObjectSubclass<imp::SinkInputModel>) @implements gio::ListModel;
}

impl SinkInputModel {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_sink_input(&self, station: &SinkInput) {
        let pos = {
            let mut map = self.imp().map.borrow_mut();
            if map.contains_key(&station.index()) {
                return;
            }

            map.insert(station.index(), station.clone());
            (map.len() - 1) as u32
        };

        dbg!(pos);
        self.items_changed(pos, 0, 1);
    }

    pub fn remove_sink_input(&self, sink_input: &SinkInput) {
        let mut map = self.imp().map.borrow_mut();

        match map.get_index_of(&sink_input.index()) {
            Some(pos) => {
                map.remove(&sink_input.index());
                self.items_changed(pos.try_into().unwrap(), 1, 0);
            }
            None => warn!("SinkInput {:?} not found in model", sink_input.title()),
        }
    }

    pub fn clear(&self) {
        let len = self.n_items();
        self.imp().map.borrow_mut().clear();
        self.items_changed(0, len, 0);
    }
}

impl Default for SinkInputModel {
    fn default() -> Self {
        Self::new()
    }
}
