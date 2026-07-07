mod appearance;
mod battery;
mod display;
mod heat;
mod network;
mod permissions;
mod resources;
mod sound;
mod updates;

pub use appearance::appearance_page;
pub use battery::BatteryPage;
pub use display::display_page;
pub use heat::HeatPage;
pub use network::network_page;
pub use permissions::PermissionsPage;
pub use resources::ResourcesPage;
pub use sound::sound_page;
pub use updates::UpdatesPage;
use libadwaita as adw;
use adw::prelude::*;

pub fn preferences_page(title: &str, description: &str, rows: &[(&str, &str)]) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::new();
    page.set_title(title);

    let group = adw::PreferencesGroup::new();
    group.set_title(title);
    group.set_description(Some(description));

    for (row_title, row_subtitle) in rows {
        let row = adw::ActionRow::new();
        row.set_title(row_title);
        row.set_subtitle(row_subtitle);
        group.add(&row);
    }

    page.add(&group);
    page
}
