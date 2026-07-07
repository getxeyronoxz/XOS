use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::Path;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Button, ListBox, Entry, ComboBoxText};
use libadwaita as adw;
use adw::prelude::*;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "/etc/xos/automations.toml";
const FALLBACK_CONFIG: &str = "/tmp/xos_automations.toml";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Rule {
    name: String,
    trigger: String,
    action: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct AutomationsConfig {
    rules: Vec<Rule>,
}

pub struct AutomationBuilderWindow {
    window: adw::ApplicationWindow,
    list_box: ListBox,
    config: Rc<RefCell<AutomationsConfig>>,
}

impl AutomationBuilderWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Automation Visual Builder")
            .default_width(620)
            .default_height(580)
            .build();

        let root_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_content(Some(&root_box));

        // Header
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&Label::new(Some("System Automations"))));
        
        let add_btn = Button::builder()
            .label("Add Rule")
            .css_classes(vec!["suggested-action".to_string()])
            .build();
        header.pack_end(&add_btn);
        
        root_box.append(&header);

        // Scrolled view
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        root_box.append(&scroll);

        let content_box = GtkBox::new(Orientation::Vertical, 12);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(16);
        scroll.set_child(Some(&content_box));

        // Title and description
        let title_lbl = Label::builder()
            .label("Configure Automatic Tasks")
            .xalign(0.0)
            .build();
        title_lbl.add_css_class("title-1");
        content_box.append(&title_lbl);

        let desc_lbl = Label::builder()
            .label("Define trigger-action rules. The automation engine runs these rules in the background.")
            .xalign(0.0)
            .build();
        desc_lbl.add_css_class("body");
        content_box.append(&desc_lbl);

        // List of rules
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["boxed-list".to_string()])
            .build();
        content_box.append(&list_box);

        // Load configuration
        let config_path = if Path::new(CONFIG_FILE).exists() {
            CONFIG_FILE
        } else {
            FALLBACK_CONFIG
        };

        let config = match read_config(config_path) {
            Ok(cfg) => cfg,
            Err(_) => {
                let default_cfg = get_default_config();
                let _ = write_config(config_path, &default_cfg);
                default_cfg
            }
        };

        let config_rc = Rc::new(RefCell::new(config));

        let mut win_self = Self {
            window,
            list_box,
            config: Rc::clone(&config_rc),
        };

        win_self.rebuild_list();

        // Connect Add Rule Dialog
        add_btn.connect_clicked({
            let config_rc = Rc::clone(&config_rc);
            let list_box = win_self.list_box.clone();
            let parent_window = win_self.window.clone();
            move |_| {
                show_add_dialog(&parent_window, &config_rc, {
                    let config_rc = Rc::clone(&config_rc);
                    let list_box = list_box.clone();
                    let parent_window = parent_window.clone();
                    move || {
                        rebuild_list_static(&list_box, &config_rc, &parent_window);
                    }
                });
            }
        });

        win_self
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn rebuild_list(&mut self) {
        rebuild_list_static(&self.list_box, &self.config, &self.window);
    }
}

fn rebuild_list_static(list_box: &ListBox, config_rc: &Rc<RefCell<AutomationsConfig>>, parent_window: &adw::ApplicationWindow) {
    // Clear list
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let rules = config_rc.borrow().rules.clone();
    if rules.is_empty() {
        let row = adw::ActionRow::builder()
            .title("No Automations Configured")
            .subtitle("Click 'Add Rule' to create a new task.")
            .build();
        list_box.append(&row);
        return;
    }

    for (idx, rule) in rules.iter().enumerate() {
        let row = adw::ActionRow::builder()
            .title(&rule.name)
            .subtitle(&format!("Trigger: {}  |  Action: {}", rule.trigger, rule.action))
            .build();

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(vec!["flat".to_string(), "destructive-action".to_string()])
            .valign(gtk4::Align::Center)
            .build();

        row.add_suffix(&delete_btn);

        delete_btn.connect_clicked({
            let config_rc = Rc::clone(config_rc);
            let list_box = list_box.clone();
            let parent_window = parent_window.clone();
            move |_| {
                config_rc.borrow_mut().rules.remove(idx);
                let config_path = if Path::new(CONFIG_FILE).exists() {
                    CONFIG_FILE
                } else {
                    FALLBACK_CONFIG
                };
                let _ = write_config(config_path, &config_rc.borrow());
                rebuild_list_static(&list_box, &config_rc, &parent_window);
            }
        });

        list_box.append(&row);
    }
}

fn show_add_dialog(parent: &adw::ApplicationWindow, config_rc: &Rc<RefCell<AutomationsConfig>>, on_save: impl Fn() + 'static) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Add Automation Rule")
        .default_width(400)
        .build();

    // Workaround deprecated dialog actions by creating custom button boxes
    let content_area = dialog.content_area();
    content_area.set_spacing(12);
    content_area.set_margin_start(16);
    content_area.set_margin_end(16);
    content_area.set_margin_top(16);
    content_area.set_margin_bottom(16);

    let name_lbl = Label::builder().label("Rule Name:").xalign(0.0).build();
    let name_entry = Entry::new();
    
    let trigger_lbl = Label::builder().label("Select Trigger:").xalign(0.0).build();
    let trigger_combo = ComboBoxText::new();
    trigger_combo.append(Some("capacity < 20"), "Battery Low (< 20%)");
    trigger_combo.append(Some("cpu_temp > 85"), "High CPU Temp (> 85°C)");
    trigger_combo.append(Some("status == Charging"), "AC Power Plugged In");
    trigger_combo.append(Some("status == Discharging"), "AC Power Unplugged");
    trigger_combo.set_active_id(Some("capacity < 20"));

    let action_lbl = Label::builder().label("Action Command / Script:").xalign(0.0).build();
    let action_entry = Entry::builder()
        .placeholder_text("e.g. xos-notify 'alert' 'plugged in'")
        .build();

    content_area.append(&name_lbl);
    content_area.append(&name_entry);
    content_area.append(&trigger_lbl);
    content_area.append(&trigger_combo);
    content_area.append(&action_lbl);
    content_area.append(&action_entry);

    // Dialog Buttons
    let actions_box = GtkBox::new(Orientation::Horizontal, 8);
    actions_box.set_halign(gtk4::Align::End);
    
    let cancel_btn = Button::builder().label("Cancel").build();
    let save_btn = Button::builder().label("Save").css_classes(vec!["suggested-action".to_string()]).build();
    
    actions_box.append(&cancel_btn);
    actions_box.append(&save_btn);
    content_area.append(&actions_box);

    cancel_btn.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.destroy();
        }
    });

    save_btn.connect_clicked({
        let dialog = dialog.clone();
        let config_rc = Rc::clone(config_rc);
        move |_| {
            let name = name_entry.text().to_string();
            let trigger = trigger_combo.active_id().unwrap_or_else(|| glib::GString::from("")).to_string();
            let action = action_entry.text().to_string();

            if !name.is_empty() && !trigger.is_empty() && !action.is_empty() {
                config_rc.borrow_mut().rules.push(Rule {
                    name,
                    trigger,
                    action,
                });
                
                let config_path = if Path::new(CONFIG_FILE).exists() {
                    CONFIG_FILE
                } else {
                    FALLBACK_CONFIG
                };
                let _ = write_config(config_path, &config_rc.borrow());
                on_save();
                dialog.destroy();
            }
        }
    });

    dialog.present();
}

fn read_config(path: &str) -> Result<AutomationsConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: AutomationsConfig = toml::from_str(&content)?;
    Ok(config)
}

fn write_config(path: &str, config: &AutomationsConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn get_default_config() -> AutomationsConfig {
    let rules = vec![
        Rule {
            name: "low_battery".to_string(),
            trigger: "capacity < 20".to_string(),
            action: "xos-notify 'Battery Low' 'System is running low on power (< 20%)'".to_string(),
        },
        Rule {
            name: "high_temp".to_string(),
            trigger: "cpu_temp > 85".to_string(),
            action: "xos-notify 'High Temperature' 'CPU temperature is critically high (> 85°C)'".to_string(),
        },
        Rule {
            name: "power_plugged".to_string(),
            trigger: "status == Charging".to_string(),
            action: "xos-notify 'Power Connected' 'Performance mode switched to Balanced'".to_string(),
        },
    ];
    AutomationsConfig { rules }
}
