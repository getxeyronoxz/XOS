use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::overview::OverviewPage;
use crate::processes::ProcessesPage;
use crate::sysdata::SysDataLoader;

pub struct SystemMonitorWindow {
    window: adw::ApplicationWindow,
}

impl SystemMonitorWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("System Monitor")
            .default_width(960)
            .default_height(680)
            .build();

        let header = adw::HeaderBar::new();
        
        let view_stack = adw::ViewStack::new();
        
        let view_switcher = adw::ViewSwitcher::builder()
            .stack(&view_stack)
            .build();
            
        header.set_title_widget(Some(&view_switcher));

        let mut overview_page = OverviewPage::new();
        let mut processes_page = ProcessesPage::new();

        // Add to stack
        let overview_stack_page = view_stack.add_titled(&overview_page.container, Some("overview"), "Overview");
        overview_stack_page.set_icon_name(Some("sensors-applet-symbolic"));

        let processes_stack_page = view_stack.add_titled(&processes_page.container, Some("processes"), "Processes");
        processes_stack_page.set_icon_name(Some("utilities-system-monitor-symbolic"));

        let content_box = GtkBox::new(Orientation::Vertical, 0);
        content_box.append(&header);
        content_box.append(&view_stack);
        window.set_content(Some(&content_box));

        // Shared Data Loader
        let data_loader = Rc::new(RefCell::new(SysDataLoader::new()));

        // Perform initial load
        {
            let mut loader = data_loader.borrow_mut();
            let stats = loader.fetch_stats();
            overview_page.update(&stats);
            processes_page.update(&stats.processes);
        }

        // Setup periodic 2-second timer
        let overview_ref = Rc::new(RefCell::new(overview_page));
        let processes_ref = Rc::new(RefCell::new(processes_page));

        glib::timeout_add_local(std::time::Duration::from_secs(2), {
            let loader = Rc::clone(&data_loader);
            let overview = Rc::clone(&overview_ref);
            let processes = Rc::clone(&processes_ref);
            move || {
                let mut loader_borrow = loader.borrow_mut();
                let stats = loader_borrow.fetch_stats();
                
                overview.borrow_mut().update(&stats);
                processes.borrow_mut().update(&stats.processes);

                glib::ControlFlow::Continue
            }
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
