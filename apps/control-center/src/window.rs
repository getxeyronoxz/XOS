use std::cell::RefCell;
use std::rc::Rc;
use gtk4::prelude::*;
use gtk4::{Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SelectionMode, Stack};
use libadwaita as adw;
use adw::prelude::*;

use crate::pages::{appearance_page, display_page, network_page, sound_page, BatteryPage, HeatPage, ResourcesPage, PermissionsPage, UpdatesPage};

struct Section {
    id: &'static str,
    title: &'static str,
    icon: &'static str,
}

const SECTIONS: [Section; 9] = [
    Section {
        id: "appearance",
        title: "Appearance",
        icon: "preferences-system-appearance-symbolic",
    },
    Section {
        id: "display",
        title: "Displays",
        icon: "preferences-desktop-display-symbolic",
    },
    Section {
        id: "sound",
        title: "Sound",
        icon: "audio-volume-high-symbolic",
    },
    Section {
        id: "network",
        title: "Network",
        icon: "preferences-system-network-symbolic",
    },
    Section {
        id: "battery",
        title: "Battery",
        icon: "battery-level-100-charged-symbolic",
    },
    Section {
        id: "heat",
        title: "Heat & Fans",
        icon: "weather-clear-symbolic",
    },
    Section {
        id: "resources",
        title: "Resources",
        icon: "utilities-system-monitor-symbolic",
    },
    Section {
        id: "permissions",
        title: "Permissions",
        icon: "security-high-symbolic",
    },
    Section {
        id: "updates",
        title: "Updates",
        icon: "system-software-update-symbolic",
    },
];

pub struct ControlCenterWindow {
    window: adw::ApplicationWindow,
}

impl ControlCenterWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Settings")
            .default_width(920)
            .default_height(640)
            .build();

        let split_view = adw::NavigationSplitView::new();
        split_view.set_collapsed(false);
        split_view.set_show_content(true);

        let sidebar = build_sidebar();
        let sidebar_page = adw::NavigationPage::builder()
            .title("Settings")
            .child(&sidebar)
            .build();

        let battery_page = Rc::new(RefCell::new(BatteryPage::new()));
        let heat_page = Rc::new(RefCell::new(HeatPage::new()));
        let resources_page = ResourcesPage::new();
        let permissions_page = PermissionsPage::new();
        let updates_page = Rc::new(RefCell::new(UpdatesPage::new()));

        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(150);
        stack.add_named(&wrap_page(appearance_page()), Some("appearance"));
        stack.add_named(&wrap_page(display_page()), Some("display"));
        stack.add_named(&wrap_page(sound_page()), Some("sound"));
        stack.add_named(&wrap_page(network_page()), Some("network"));
        stack.add_named(&wrap_page(battery_page.borrow().container.clone()), Some("battery"));
        stack.add_named(&wrap_page(heat_page.borrow().container.clone()), Some("heat"));
        stack.add_named(&wrap_page(resources_page.container), Some("resources"));
        stack.add_named(&wrap_page(permissions_page.container), Some("permissions"));
        stack.add_named(&wrap_page(updates_page.borrow().container.clone()), Some("updates"));
        stack.set_visible_child_name("appearance");

        let content_page = adw::NavigationPage::builder()
            .title("Details")
            .child(&stack)
            .build();

        split_view.set_sidebar(Some(&sidebar_page));
        split_view.set_content(Some(&content_page));

        let toolbar_view = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&Label::new(Some("XOS Settings"))));
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&split_view));

        window.set_content(Some(&toolbar_view));

        // Periodic status polling
        glib::timeout_add_local(std::time::Duration::from_secs(3), {
            let battery = Rc::clone(&battery_page);
            let heat = Rc::clone(&heat_page);
            let updates = Rc::clone(&updates_page);
            move || {
                battery.borrow_mut().update();
                heat.borrow_mut().update();
                updates.borrow_mut().update();
                glib::ControlFlow::Continue
            }
        });

        sidebar.connect_row_selected(move |_, row| {
            let Some(row) = row else { return };
            stack.set_visible_child_name(&row.widget_name());
        });

        if let Some(first_row) = sidebar.row_at_index(0) {
            sidebar.select_row(Some(&first_row));
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn wrap_page(page: adw::PreferencesPage) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&page)
        .build();
    scrolled
}

fn build_sidebar() -> ListBox {
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Single);
    list_box.add_css_class("navigation-sidebar");

    for section in SECTIONS {
        let row = ListBoxRow::new();
        row.set_widget_name(section.id);

        let content = gtk4::Box::new(Orientation::Horizontal, 12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(10);
        content.set_margin_bottom(10);

        let icon = gtk4::Image::from_icon_name(section.icon);
        let label = Label::new(Some(section.title));
        label.set_xalign(0.0);

        content.append(&icon);
        content.append(&label);
        row.set_child(Some(&content));
        list_box.append(&row);
    }

    list_box
}
