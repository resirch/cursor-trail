use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);

pub struct TrayController {
    _tray: TrayIcon,
    pub settings_id: MenuItem,
    pub toggle_trail_id: MenuItem,
    pub toggle_avatar_id: MenuItem,
    pub quit_id: MenuItem,
}

impl TrayController {
    pub fn new() -> Result<Self> {
        let icon = crate::icon::load_tray_icon().context("Failed to load tray icon")?;

        let menu = Menu::new();
        let settings_id = MenuItem::new("Settings", true, None);
        let toggle_trail_id = MenuItem::new("Toggle Trail", true, None);
        let toggle_avatar_id = MenuItem::new("Toggle Avatar", true, None);
        let quit_id = MenuItem::new("Quit", true, None);

        menu.append(&settings_id)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&toggle_trail_id)?;
        menu.append(&toggle_avatar_id)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_id)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Cursor Trail")
            .with_icon(icon)
            .build()
            .context("Failed to create system tray icon")?;

        Ok(Self {
            _tray: tray,
            settings_id,
            toggle_trail_id,
            toggle_avatar_id,
            quit_id,
        })
    }

    pub fn try_recv_event(&self) -> Option<TrayAction> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        if event.id == self.settings_id.id() {
            Some(TrayAction::OpenSettings)
        } else if event.id == self.toggle_trail_id.id() {
            Some(TrayAction::ToggleTrail)
        } else if event.id == self.toggle_avatar_id.id() {
            Some(TrayAction::ToggleAvatar)
        } else if event.id == self.quit_id.id() {
            Some(TrayAction::Quit)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    OpenSettings,
    ToggleTrail,
    ToggleAvatar,
    Quit,
}

pub fn try_open_settings(
    config_path: std::path::PathBuf,
    config: crate::config::Config,
    apply_tx: std::sync::mpsc::Sender<crate::config::Config>,
) -> bool {
    if SETTINGS_OPEN
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return false;
    }

    std::thread::spawn(move || {
        let result = crate::settings::run_settings_window(config_path, config, apply_tx);
        SETTINGS_OPEN.store(false, Ordering::SeqCst);
        if let Err(error) = result {
            eprintln!("Settings window error: {error}");
        }
    });

    true
}
