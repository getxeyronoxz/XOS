use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::sysdata::SystemStats;

pub struct OverviewPage {
    pub container: ScrolledWindow,
    cpu_label: Label,
    cpu_bar: ProgressBar,
    cores_box: GtkBox,
    core_bars: Vec<ProgressBar>,
    mem_label: Label,
    mem_bar: ProgressBar,
    disk_box: GtkBox,
    net_rx_label: Label,
    net_tx_label: Label,
}

impl OverviewPage {
    pub fn new() -> Self {
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        // 1. CPU Section
        let cpu_group = adw::PreferencesGroup::builder()
            .title("CPU Utilization")
            .build();

        let cpu_vbox = GtkBox::new(Orientation::Vertical, 8);
        cpu_vbox.set_margin_top(8);
        cpu_vbox.set_margin_bottom(8);
        cpu_vbox.set_margin_start(12);
        cpu_vbox.set_margin_end(12);

        let cpu_label = Label::builder()
            .label("CPU Usage: 0.0%")
            .xalign(0.0)
            .build();
        cpu_label.add_css_class("heading");

        let cpu_bar = ProgressBar::new();
        cpu_bar.set_show_text(false);

        let cores_box = GtkBox::new(Orientation::Vertical, 6);
        cores_box.set_margin_top(8);

        cpu_vbox.append(&cpu_label);
        cpu_vbox.append(&cpu_bar);
        cpu_vbox.append(&cores_box);
        cpu_group.add(&cpu_vbox);
        main_box.append(&cpu_group);

        // 2. Memory Section
        let mem_group = adw::PreferencesGroup::builder()
            .title("System Memory")
            .build();

        let mem_vbox = GtkBox::new(Orientation::Vertical, 8);
        mem_vbox.set_margin_top(8);
        mem_vbox.set_margin_bottom(8);
        mem_vbox.set_margin_start(12);
        mem_vbox.set_margin_end(12);

        let mem_label = Label::builder()
            .label("Memory: 0.0 GB / 0.0 GB (0.0%)")
            .xalign(0.0)
            .build();
        mem_label.add_css_class("heading");

        let mem_bar = ProgressBar::new();
        mem_bar.set_show_text(false);

        mem_vbox.append(&mem_label);
        mem_vbox.append(&mem_bar);
        mem_group.add(&mem_vbox);
        main_box.append(&mem_group);

        // 3. Disk Section
        let disk_group = adw::PreferencesGroup::builder()
            .title("Disk Space")
            .build();

        let disk_box = GtkBox::new(Orientation::Vertical, 12);
        disk_box.set_margin_top(8);
        disk_box.set_margin_bottom(8);
        disk_box.set_margin_start(12);
        disk_box.set_margin_end(12);
        disk_group.add(&disk_box);
        main_box.append(&disk_group);

        // 4. Network Section
        let net_group = adw::PreferencesGroup::builder()
            .title("Network Activity")
            .build();

        let net_hbox = GtkBox::new(Orientation::Horizontal, 24);
        net_hbox.set_margin_top(8);
        net_hbox.set_margin_bottom(8);
        net_hbox.set_margin_start(12);
        net_hbox.set_margin_end(12);

        let net_rx_label = Label::builder()
            .label("Download: 0.0 KB/s")
            .xalign(0.0)
            .hexpand(true)
            .build();
        net_rx_label.add_css_class("heading");

        let net_tx_label = Label::builder()
            .label("Upload: 0.0 KB/s")
            .xalign(0.0)
            .hexpand(true)
            .build();
        net_tx_label.add_css_class("heading");

        net_hbox.append(&net_rx_label);
        net_hbox.append(&net_tx_label);
        net_group.add(&net_hbox);
        main_box.append(&net_group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&main_box)
            .build();

        Self {
            container: scrolled,
            cpu_label,
            cpu_bar,
            cores_box,
            core_bars: Vec::new(),
            mem_label,
            mem_bar,
            disk_box,
            net_rx_label,
            net_tx_label,
        }
    }

    pub fn update(&mut self, stats: &SystemStats) {
        // CPU
        self.cpu_label.set_label(&format!("CPU Usage: {:.1}%", stats.cpu_global));
        self.cpu_bar.set_fraction((stats.cpu_global / 100.0) as f64);

        // CPU Cores
        if self.core_bars.len() != stats.cpu_cores.len() {
            // Re-populate core bars
            while let Some(child) = self.cores_box.first_child() {
                self.cores_box.remove(&child);
            }
            self.core_bars.clear();

            for i in 0..stats.cpu_cores.len() {
                let core_hbox = GtkBox::new(Orientation::Horizontal, 8);
                let core_lbl = Label::builder()
                    .label(&format!("Core {i}"))
                    .xalign(0.0)
                    .width_chars(8)
                    .build();
                let core_bar = ProgressBar::new();
                core_bar.set_hexpand(true);
                core_hbox.append(&core_lbl);
                core_hbox.append(&core_bar);
                self.cores_box.append(&core_hbox);
                self.core_bars.push(core_bar);
            }
        }

        for (bar, usage) in self.core_bars.iter().zip(stats.cpu_cores.iter()) {
            bar.set_fraction((*usage / 100.0) as f64);
        }

        // Memory (bytes to GB)
        let total_gb = stats.mem_total as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_gb = stats.mem_used as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_pct = if stats.mem_total > 0 {
            (stats.mem_used as f64 / stats.mem_total as f64) * 100.0
        } else {
            0.0
        };
        self.mem_label.set_label(&format!("Memory: {:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, mem_pct));
        self.mem_bar.set_fraction(mem_pct / 100.0);

        // Disks
        while let Some(child) = self.disk_box.first_child() {
            self.disk_box.remove(&child);
        }
        for (mount, total, used) in &stats.disks {
            let total_gb = *total as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_gb = *used as f64 / 1024.0 / 1024.0 / 1024.0;
            let pct = if *total > 0 {
                (*used as f64 / *total as f64) * 100.0
            } else {
                0.0
            };

            let disk_vbox = GtkBox::new(Orientation::Vertical, 4);
            let lbl = Label::builder()
                .label(&format!("{} — {:.1} GB / {:.1} GB ({:.1}%)", mount, used_gb, total_gb, pct))
                .xalign(0.0)
                .build();
            let bar = ProgressBar::new();
            bar.set_fraction(pct / 100.0);

            disk_vbox.append(&lbl);
            disk_vbox.append(&bar);
            self.disk_box.append(&disk_vbox);
        }

        // Network rates (diff in bytes per 2s -> bytes/sec)
        let rx_bps = stats.net_rx as f64 / 2.0;
        let tx_bps = stats.net_tx as f64 / 2.0;
        
        self.net_rx_label.set_label(&format!("Download: {}", format_rate(rx_bps)));
        self.net_tx_label.set_label(&format!("Upload: {}", format_rate(tx_bps)));
    }
}

fn format_rate(bps: f64) -> String {
    if bps >= 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bps / 1024.0 / 1024.0)
    } else if bps >= 1024.0 {
        format!("{:.1} KB/s", bps / 1024.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}
