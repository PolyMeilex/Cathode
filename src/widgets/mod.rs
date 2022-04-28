pub mod channel_scale;

pub mod level_box;
pub mod playback_item;

pub mod output_page;
pub mod playback_page;

pub use channel_scale::ChannelScale;

pub use level_box::LevelBox;
pub use playback_item::PlaybackItem;

pub use output_page::OutputPage;
pub use playback_page::PlaybackPage;

use gtk::prelude::StaticType;

pub fn init_types() {
    channel_scale::ChannelScale::static_type();
    level_box::LevelBox::static_type();
    playback_item::PlaybackItem::static_type();

    output_page::OutputPage::static_type();
    playback_page::PlaybackPage::static_type();
}
