use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub trail: TrailConfig,
    #[serde(default)]
    pub avatar: AvatarConfig,
    #[serde(default)]
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_trail_color")]
    pub color: [u8; 4],
    #[serde(default = "default_max_points")]
    pub max_points: usize,
    #[serde(default = "default_point_size")]
    pub point_size: f32,
    #[serde(default = "default_fade_speed")]
    pub fade_speed: f32,
    #[serde(default = "default_spacing")]
    pub spacing: f32,
    #[serde(default = "default_trail_shape")]
    pub shape: TrailShape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrailShape {
    Circle,
    Square,
    Star,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub image_path: Option<PathBuf>,
    #[serde(default = "default_string_length")]
    pub string_length: f32,
    #[serde(default = "default_avatar_size")]
    pub size: f32,
    /// Gravity in pixels per second squared. Set to 0 for zero-gravity floating.
    #[serde(default = "default_gravity")]
    pub gravity: f32,
    #[serde(default = "default_damping")]
    pub damping: f32,
    #[serde(default = "default_string_color")]
    pub string_color: [u8; 4],
    #[serde(default = "default_string_width")]
    pub string_width: f32,
    #[serde(default = "default_swing_boost")]
    pub swing_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default = "default_true")]
    pub click_through: bool,
}

fn default_true() -> bool {
    true
}

fn default_trail_color() -> [u8; 4] {
    [120, 180, 255, 200]
}

fn default_max_points() -> usize {
    28
}

fn default_point_size() -> f32 {
    10.0
}

fn default_fade_speed() -> f32 {
    1.2
}

fn default_spacing() -> f32 {
    6.0
}

fn default_trail_shape() -> TrailShape {
    TrailShape::Circle
}

fn default_string_length() -> f32 {
    90.0
}

fn default_avatar_size() -> f32 {
    56.0
}

fn default_gravity() -> f32 {
    980.0
}

fn default_damping() -> f32 {
    0.985
}

fn default_string_color() -> [u8; 4] {
    [220, 220, 220, 220]
}

fn default_string_width() -> f32 {
    2.0
}

fn default_swing_boost() -> f32 {
    1.0
}

fn default_fps() -> u32 {
    60
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trail: TrailConfig::default(),
            avatar: AvatarConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            color: default_trail_color(),
            max_points: default_max_points(),
            point_size: default_point_size(),
            fade_speed: default_fade_speed(),
            spacing: default_spacing(),
            shape: default_trail_shape(),
        }
    }
}

impl Default for AvatarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            image_path: None,
            string_length: default_string_length(),
            size: default_avatar_size(),
            gravity: default_gravity(),
            damping: default_damping(),
            string_color: default_string_color(),
            string_width: default_string_width(),
            swing_boost: default_swing_boost(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            fps: default_fps(),
            click_through: default_true(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cursor-trail")
            .join("config.toml")
    }

    pub fn write_default_if_missing(path: &Path) -> Result<()> {
        if path.exists() {
            return Ok(());
        }
        Config::default().save(path)
    }
}
