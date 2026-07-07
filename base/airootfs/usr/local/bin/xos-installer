#!/usr/bin/env python3
import os
import sys
import time
import threading
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, GLib, Adw

class InstallerWindow(Adw.ApplicationWindow):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.set_title("XOS System Installer")
        self.set_default_size(680, 500)

        # Root box
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.set_content(box)

        # Header
        header = Adw.HeaderBar()
        header.set_title_widget(Gtk.Label(label="XOS Installation Wizard"))
        box.append(header)

        # View Stack for slides
        self.stack = Adw.ViewStack()
        box.append(self.stack)

        # Build Slides
        self.build_welcome_slide()
        self.build_partitioning_slide()
        self.build_user_slide()
        self.build_progress_slide()

        # Bottom navigation controls
        nav_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        nav_box.set_margin_start(16)
        nav_box.set_margin_end(16)
        nav_box.set_margin_top(12)
        nav_box.set_margin_bottom(16)
        box.append(nav_box)

        self.back_btn = Gtk.Button(label="Back")
        self.back_btn.connect("clicked", self.on_back)
        self.back_btn.set_sensitive(False)
        nav_box.append(self.back_btn)

        # Spacer
        spacer = Gtk.Box()
        spacer.set_hexpand(True)
        nav_box.append(spacer)

        self.next_btn = Gtk.Button(label="Next", css_classes=["suggested-action"])
        self.next_btn.connect("clicked", self.on_next)
        nav_box.append(self.next_btn)

        self.current_step = 0
        self.steps = ["welcome", "partition", "user", "progress"]

    def build_welcome_slide(self):
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        vbox.set_margin_start(24)
        vbox.set_margin_end(24)
        vbox.set_margin_top(24)
        vbox.set_margin_bottom(24)

        title = Gtk.Label(label="Welcome to XOS", css_classes=["title-1"])
        title.set_halign(Gtk.Align.CENTER)
        
        desc = Gtk.Label(
            label="This installer will guide you through setting up XOS on your computer. "
                  "XOS ships with modern default settings including Btrfs root structures, "
                  "automatic bootloader restore snapshots, and cgroup-enforced limits.",
            wrap=True,
            max_width_chars=50
        )
        desc.set_halign(Gtk.Align.CENTER)

        logo = Gtk.Image.new_from_icon_name("system-software-update-symbolic")
        logo.set_pixel_size(96)

        vbox.append(logo)
        vbox.append(title)
        vbox.append(desc)

        self.stack.add_titled_with_icon(vbox, "welcome", "Welcome", "go-home-symbolic")

    def build_partitioning_slide(self):
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        vbox.set_margin_start(24)
        vbox.set_margin_end(24)
        vbox.set_margin_top(24)
        vbox.set_margin_bottom(24)

        title = Gtk.Label(label="Disk Partitioning & Encryption", css_classes=["title-2"])
        title.set_halign(Gtk.Align.START)
        vbox.append(title)

        # Drive selector
        drive_lbl = Gtk.Label(label="Select Target Hard Drive:", xalign=0.0)
        vbox.append(drive_lbl)

        self.drive_combo = Gtk.ComboBoxText()
        self.drive_combo.append("sda", "/dev/sda (SATA SSD, 256 GB)")
        self.drive_combo.append("nvme0n1", "/dev/nvme0n1 (NVMe SSD, 512 GB)")
        self.drive_combo.set_active_id("nvme0n1")
        vbox.append(self.drive_combo)

        # Encryption toggle
        self.enc_check = Gtk.CheckButton(label="Enable LUKS2 Disk Encryption")
        self.enc_check.set_active(True)
        vbox.append(self.enc_check)

        self.pass_lbl = Gtk.Label(label="Passphrase for Encryption:", xalign=0.0)
        self.pass_entry = Gtk.Entry()
        self.pass_entry.set_visibility(False)
        self.pass_entry.set_text("supersecret")

        vbox.append(self.pass_lbl)
        vbox.append(self.pass_entry)

        # Bind visibility of password inputs
        self.enc_check.connect("toggled", self.on_enc_toggled)

        self.stack.add_titled_with_icon(vbox, "partition", "Partitioning", "drive-harddisk-symbolic")

    def on_enc_toggled(self, widget):
        active = widget.get_active()
        self.pass_lbl.set_visible(active)
        self.pass_entry.set_visible(active)

    def build_user_slide(self):
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        vbox.set_margin_start(24)
        vbox.set_margin_end(24)
        vbox.set_margin_top(24)
        vbox.set_margin_bottom(24)

        title = Gtk.Label(label="User Account Setup", css_classes=["title-2"])
        title.set_halign(Gtk.Align.START)
        vbox.append(title)

        username_lbl = Gtk.Label(label="Username:", xalign=0.0)
        self.username_entry = Gtk.Entry()
        self.username_entry.set_text("xuser")

        hostname_lbl = Gtk.Label(label="Computer Name (Hostname):", xalign=0.0)
        self.hostname_entry = Gtk.Entry()
        self.hostname_entry.set_text("xos-desktop")

        password_lbl = Gtk.Label(label="User Password:", xalign=0.0)
        self.password_entry = Gtk.Entry()
        self.password_entry.set_visibility(False)
        self.password_entry.set_text("xos2026")

        vbox.append(username_lbl)
        vbox.append(self.username_entry)
        vbox.append(hostname_lbl)
        vbox.append(self.hostname_entry)
        vbox.append(password_lbl)
        vbox.append(self.password_entry)

        self.stack.add_titled_with_icon(vbox, "user", "User Setup", "avatar-default-symbolic")

    def build_progress_slide(self):
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        vbox.set_margin_start(24)
        vbox.set_margin_end(24)
        vbox.set_margin_top(24)
        vbox.set_margin_bottom(24)

        self.progress_title = Gtk.Label(label="Installing XOS...", css_classes=["title-2"])
        self.progress_title.set_halign(Gtk.Align.START)
        vbox.append(self.progress_title)

        self.progress_bar = Gtk.ProgressBar()
        vbox.append(self.progress_bar)

        self.progress_log = Gtk.Label(label="Preparing installation layout...", xalign=0.0)
        self.progress_log.add_css_class("caption")
        vbox.append(self.progress_log)

        self.stack.add_titled_with_icon(vbox, "progress", "Installation", "emblem-system-symbolic")

    def on_back(self, widget):
        if self.current_step > 0:
            self.current_step -= 1
            step_name = self.steps[self.current_step]
            self.stack.set_visible_child_name(step_name)
            
            self.next_btn.set_label("Next")
            self.back_btn.set_sensitive(self.current_step > 0)

    def on_next(self, widget):
        if self.current_step < len(self.steps) - 1:
            self.current_step += 1
            step_name = self.steps[self.current_step]
            self.stack.set_visible_child_name(step_name)
            
            self.back_btn.set_sensitive(True)
            if self.current_step == len(self.steps) - 1:
                self.next_btn.set_label("Install")
        else:
            # Install trigger
            self.next_btn.set_sensitive(False)
            self.back_btn.set_sensitive(False)
            threading.Thread(target=self.run_installation).start()

    def run_installation(self):
        steps = [
            ("Creating EFI system partition...", 0.1),
            ("Formatting root partition with Btrfs...", 0.25),
            ("Setting up Btrfs subvolumes (@ and @snapshots)...", 0.4),
            ("Mounting target file systems...", 0.5),
            ("Installing system packages (pacstrap)...", 0.7),
            ("Configuring bootloader (systemd-boot)...", 0.85),
            ("Setting up default user credentials...", 0.95),
            ("Installation finished successfully! Reboot to load XOS.", 1.0)
        ]

        for log, val in steps:
            time.sleep(1.2)
            GLib.idle_add(self.update_progress, log, val)

    def update_progress(self, log, val):
        self.progress_log.set_label(log)
        self.progress_bar.set_fraction(val)
        if val == 1.0:
            self.progress_title.set_label("Installation Complete")
            self.next_btn.set_label("Reboot Now")
            self.next_btn.set_sensitive(True)
            self.next_btn.disconnect_by_func(self.on_next)
            self.next_btn.connect("clicked", lambda w: sys.exit(0))
        return False

class InstallerApp(Adw.Application):
    def __init__(self, **kwargs):
        super().__init__(application_id="org.xos.Installer", **kwargs)
        self.connect("activate", self.on_activate)

    def on_activate(self, app):
        self.win = InstallerWindow(application=app)
        self.win.present()

if __name__ == "__main__":
    app = InstallerApp()
    sys.exit(app.run(sys.argv))
