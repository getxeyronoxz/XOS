use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::notification::NotificationStore;
use crate::notification_row::build_notification_row;

pub struct NotificationCenterWindow {
    window: adw::ApplicationWindow,
}

impl NotificationCenterWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Notifications")
            .default_width(420)
            .default_height(700)
            .build();

        let header = adw::HeaderBar::new();
        
        let clear_all_btn = Button::builder()
            .label("Clear All")
            .build();
        clear_all_btn.add_css_class("destructive-action");
        header.pack_end(&clear_all_btn);

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);

        // Do Not Disturb Switch Row
        let dnd_group = adw::PreferencesGroup::new();
        dnd_group.set_margin_start(16);
        dnd_group.set_margin_end(16);
        dnd_group.set_margin_top(12);
        dnd_group.set_margin_bottom(12);

        let dnd_switch = gtk4::Switch::builder()
            .valign(gtk4::Align::Center)
            .state(false)
            .build();

        let dnd_row = adw::ActionRow::builder()
            .title("Do Not Disturb")
            .subtitle("Mute all incoming notification popups")
            .build();
        dnd_row.add_suffix(&dnd_switch);
        dnd_row.set_activatable_widget(Some(&dnd_switch));
        dnd_group.add(&dnd_row);
        main_box.append(&dnd_group);

        // List Area
        let list_container = GtkBox::new(Orientation::Vertical, 12);
        list_container.set_margin_start(16);
        list_container.set_margin_end(16);
        list_container.set_margin_bottom(16);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_container)
            .vexpand(true)
            .build();

        main_box.append(&scrolled);
        window.set_content(Some(&main_box));

        // Shared store
        let store = Rc::new(RefCell::new(NotificationStore::new()));

        // We wrap the window setup and dynamic functions in a helper struct or closures
        let refresh_list = {
            let list_container = list_container.clone();
            let store = Rc::clone(&store);

            // We use Rc::new_cyclic or a shared closure to allow recursive calls
            let refresh_ptr: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
            let refresh_ptr_clone = Rc::clone(&refresh_ptr);
            
            let refresh = move || {
                // Clear all children
                while let Some(child) = list_container.first_child() {
                    list_container.remove(&child);
                }

                let store_borrow = store.borrow();
                let grouped = store_borrow.get_grouped();

                if grouped.is_empty() {
                    let empty_page = adw::StatusPage::builder()
                        .title("No Notifications")
                        .description("You're all caught up!")
                        .icon_name("preferences-system-notifications-symbolic")
                        .vexpand(true)
                        .build();
                    list_container.append(&empty_page);
                } else {
                    // For each app group, create a PreferencesGroup
                    // Sort app names to keep UI stable
                    let mut apps: Vec<String> = grouped.keys().cloned().collect();
                    apps.sort();

                    for app in apps {
                        let group = adw::PreferencesGroup::builder()
                            .title(&app)
                            .build();

                        let group_box = GtkBox::new(Orientation::Vertical, 4);

                        if let Some(notifications) = grouped.get(&app) {
                            for n in notifications {
                                let id = n.id;
                                let store_inner = Rc::clone(&store);
                                let refresh_inner = Rc::clone(&refresh_ptr_clone);

                                let row = build_notification_row(n, move || {
                                    store_inner.borrow_mut().remove(id);
                                    if let Some(ref r) = *refresh_inner.borrow() {
                                        let r: &dyn Fn() = r.as_ref();
                                        r();
                                    }
                                });
                                group_box.append(&row);
                            }
                        }

                        group.add(&group_box);
                        list_container.append(&group);
                    }
                }
            };

            *refresh_ptr.borrow_mut() = Some(Rc::new(refresh.clone()) as Rc<dyn Fn()>);
            refresh
        };

        // Connect Clear All action
        clear_all_btn.connect_clicked({
            let store = Rc::clone(&store);
            let refresh_list = refresh_list.clone();
            move |_| {
                store.borrow_mut().clear_all();
                refresh_list();
            }
        });

        // Initial populate
        refresh_list();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
