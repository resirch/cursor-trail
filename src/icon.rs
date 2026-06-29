use anyhow::{Context, Result};
use tray_icon::Icon;

const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

pub fn load_tray_icon() -> Result<Icon> {
    let (rgba, width, height) = load_rgba(32)?;
    Icon::from_rgba(rgba, width, height).context("Failed to create tray icon")
}

pub fn load_window_icon() -> Result<egui::IconData> {
    let (rgba, width, height) = load_rgba(32)?;
    Ok(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn load_rgba(size: u32) -> Result<(Vec<u8>, u32, u32)> {
    let image = image::load_from_memory(ICON_PNG)
        .context("Failed to decode application icon")?
        .to_rgba8();
    let resized = image::imageops::resize(
        &image,
        size,
        size,
        image::imageops::FilterType::Lanczos3,
    );
    Ok((resized.into_raw(), size, size))
}
