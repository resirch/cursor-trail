mod config;
mod math;
mod overlay;
mod pendulum;
mod render;
mod trail;

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
    watch_config(&config_path, reload_tx)?;

    let mut overlay = OverlayWindow::create(config.window.click_through)?;
    let (width, height) = overlay.dimensions();
    let mut frame = FrameBuffer::new(width, height);

    let mut trail = TrailSystem::new();
    let mut avatar = PendulumAvatar::new();
    let mut sprite = load_avatar_sprite(&config)?;

    let target_frame_time = Duration::from_secs_f64(1.0 / config.window.fps.max(1) as f64);
    let mut last_frame = Instant::now();

    loop {
        if !overlay.pump_messages() {
            break;
        }

        if reload_rx.try_recv().is_ok() {
            match Config::load(&config_path) {
                Ok(new_config) => {
                    config = new_config;
                    overlay.set_click_through(config.window.click_through)?;
                    sprite = load_avatar_sprite(&config)?;
                    avatar.reset();
                }
                Err(error) => eprintln!("Config reload failed: {error}"),
            }
        }

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(0.05);
        last_frame = now;

        let (cursor_x, cursor_y) = get_cursor_position()?;
        let cursor = math::Vec2::new(cursor_x as f32, cursor_y as f32);

        trail.update(cursor, &config.trail, dt);
        avatar.update(cursor, &config.avatar, dt);

        frame.clear();
        trail.draw(&mut frame, &config.trail);
        avatar.draw(
            &mut frame,
            cursor,
            &config.avatar,
            sprite.as_ref(),
        );
        overlay.present(&frame)?;

        let elapsed = now.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }

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

fn watch_config(path: &Path, reload_tx: mpsc::Sender<()>) -> Result<()> {
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

    Ok(())
}
