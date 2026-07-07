use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;

use crate::capture::{CaptureMode, CaptureTarget, capture};

pub struct ScreenshotWindow {
    window: adw::ApplicationWindow,
}

impl ScreenshotWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Screenshot")
            .default_width(420)
            .default_height(280)
            .modal(true)
            .build();

        let toast_overlay = adw::ToastOverlay::new();
        window.set_content(Some(&toast_overlay));

        let header = adw::HeaderBar::new();
        let title = Label::new(Some("XOS Screenshot"));
        header.set_title_widget(Some(&title));

        let content = GtkBox::new(Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);

        let description = Label::new(Some(
            "Capture the screen or a selected region.\nUses grim and slurp on Wayland.",
        ));
        description.add_css_class("dim-label");
        description.set_wrap(true);

        content.append(&description);
        content.append(&build_button_row(
            "Region to clipboard",
            move |toast| run_capture(CaptureMode::Region, CaptureTarget::Clipboard, toast),
            &toast_overlay,
        ));
        content.append(&build_button_row(
            "Region to file",
            move |toast| run_capture(CaptureMode::Region, CaptureTarget::Save, toast),
            &toast_overlay,
        ));
        content.append(&build_button_row(
            "Full screen to clipboard",
            move |toast| run_capture(CaptureMode::FullScreen, CaptureTarget::Clipboard, toast),
            &toast_overlay,
        ));
        content.append(&build_button_row(
            "Full screen to file",
            move |toast| run_capture(CaptureMode::FullScreen, CaptureTarget::Save, toast),
            &toast_overlay,
        ));

        let outer = GtkBox::new(Orientation::Vertical, 0);
        outer.append(&header);
        outer.append(&content);
        toast_overlay.set_child(Some(&outer));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn build_button_row(
    label: &str,
    action: impl Fn(&adw::ToastOverlay) + 'static,
    toast_overlay: &adw::ToastOverlay,
) -> Button {
    let button = Button::with_label(label);
    button.add_css_class("pill");
    let toast = toast_overlay.clone();
    button.connect_clicked(move |_| action(&toast));
    button
}

fn run_capture(mode: CaptureMode, target: CaptureTarget, toast_overlay: &adw::ToastOverlay) {
    match capture(mode, target) {
        Ok(()) => {
            let message = match target {
                CaptureTarget::Clipboard => "Screenshot copied to clipboard",
                CaptureTarget::Save => "Screenshot saved to Pictures",
            };
            toast_overlay.add_toast(adw::Toast::new(message));
        }
        Err(err) => toast_overlay.add_toast(adw::Toast::new(&err)),
    }
}
