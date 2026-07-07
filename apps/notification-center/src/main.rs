mod app;
mod notification;
mod notification_row;
mod window;

use app::NotificationCenterApp;

fn main() {
    if let Err(err) = NotificationCenterApp::run() {
        eprintln!("Notification Center application error: {err}");
        std::process::exit(1);
    }
}
