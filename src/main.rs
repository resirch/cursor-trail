mod config;
mod math;
mod overlay;
mod pendulum;
mod render;
mod settings;
mod trail;
mod tray;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use overlay::{get_cursor_position, OverlayWindow};
use pendulum::PendulumAvatar;
use render::{FrameBuffer, Sprite};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use trail::TrailSystem;
use tray::{try_open_settings, TrayAction, TrayController};

#[derive(Parser, Debug)]
#[command(name = "cursor-trail", about = "Customizable cursor trail and hanging avatar for Windows 11")]
struct Cli {
    /// Path to the TOML config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Write a default config file and exit
    #[arg(long)]
    init: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(Config::default_path);

    if cli.init {
        Config::write_default_if_missing(&config_path)?;
        println!("Default config written to {}", config_path.display());
        return Ok(());
    }

    Config::write_default_if_missing(&config_path)?;
    let mut config = Config::load(&config_path)?;

    let (reload_tx, reload_rx) = mpsc::channel();
    let _watcher = watch_config(&config_path, reload_tx)?;

    let (apply_tx, apply_rx) = mpsc::channel();
    let tray = TrayController::new().context("Failed to create system tray icon")?;

    let mut overlay =
        OverlayWindow::create().context("Failed to create overlay window")?;
    let (width, height) = overlay.dimensions();
    let mut frame = FrameBuffer::new(width, height);

    let mut trail = TrailSystem::new();
    let mut avatar = PendulumAvatar::new();
    let mut sprite = load_avatar_sprite(&config)?;

    let mut target_frame_time =
        Duration::from_secs_f64(1.0 / config.window.fps.max(1) as f64);
    let mut last_frame = Instant::now();
    let mut running = true;

    while running {
        if !overlay.pump_messages() {
            break;
        }

        while let Some(action) = tray.try_recv_event() {
            match action {
                TrayAction::OpenSettings => {
                    let _ = try_open_settings(config_path.clone(), config.clone(), apply_tx.clone());
                }
                TrayAction::ToggleTrail => {
                    config.trail.enabled = !config.trail.enabled;
                    let _ = config.save(&config_path);
                }
                TrayAction::ToggleAvatar => {
                    config.avatar.enabled = !config.avatar.enabled;
                    let _ = config.save(&config_path);
                    sprite = load_avatar_sprite(&config)?;
                    avatar.reset();
                }
                TrayAction::Quit => running = false,
            }
        }

        if let Ok(new_config) = apply_rx.try_recv() {
            apply_config(
                &mut config,
                new_config,
                &mut overlay,
                &mut sprite,
                &mut avatar,
                &mut target_frame_time,
            )?;
        }

        if reload_rx.try_recv().is_ok() {
            match Config::load(&config_path) {
                Ok(new_config) => {
                    apply_config(
                        &mut config,
                        new_config,
                        &mut overlay,
                        &mut sprite,
                        &mut avatar,
                        &mut target_frame_time,
                    )?;
                }
                Err(error) => eprintln!("Config reload failed: {error}"),
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.05);
        last_frame = now;

        let (cursor_x, cursor_y) = get_cursor_position()?;
        let (origin_x, origin_y) = overlay.origin();
        let cursor = math::Vec2::new(
            (cursor_x - origin_x) as f32,
            (cursor_y - origin_y) as f32,
        );

        trail.update(cursor, &config.trail, dt);
        avatar.update(cursor, &config.avatar, dt);

        frame.clear();
        trail.draw(&mut frame, &config.trail, cursor);
        avatar.draw(&mut frame, cursor, &config.avatar, sprite.as_ref());
        overlay.present(&frame)?;

        let elapsed = now.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }

    Ok(())
}

fn apply_config(
    config: &mut Config,
    new_config: Config,
    _overlay: &mut OverlayWindow,
    sprite: &mut Option<Sprite>,
    avatar: &mut PendulumAvatar,
    target_frame_time: &mut Duration,
) -> Result<()> {
    *config = new_config;
    *sprite = load_avatar_sprite(config)?;
    avatar.reset();
    *target_frame_time = Duration::from_secs_f64(1.0 / config.window.fps.max(1) as f64);
    Ok(())
}

fn load_avatar_sprite(config: &Config) -> Result<Option<Sprite>> {
    if !config.avatar.enabled {
        return Ok(None);
    }

    let Some(path) = &config.avatar.image_path else {
        return Ok(None);
    };

    let image = image::open(path)
        .with_context(|| format!("Failed to load avatar image at {}", path.display()))?;
    Ok(Some(Sprite::from_image(&image.to_rgba8())))
}

fn watch_config(path: &Path, reload_tx: mpsc::Sender<()>) -> Result<RecommendedWatcher> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let _ = tx.send(());
                }
            }
        },
        NotifyConfig::default(),
    )?;

    if path.exists() {
        watcher.watch(path, RecursiveMode::NonRecursive)?;
    } else if let Some(parent) = path.parent() {
        watcher.watch(parent, RecursiveMode::NonRecursive)?;
    }

    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            std::thread::sleep(Duration::from_millis(150));
            let _ = reload_tx.send(());
        }
    });

    Ok(watcher)
}
