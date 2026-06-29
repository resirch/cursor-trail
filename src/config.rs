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
    /// Maximum trail length in pixels.
    #[serde(alias = "length", default = "default_trail_max_length")]
    pub max_length: f32,
    /// Line thickness in pixels.
    #[serde(default = "default_trail_width")]
    pub width: f32,
    /// Pinch line width toward the tail. 0 = uniform, 1 = tapers to a point.
    #[serde(default = "default_trail_taper")]
    pub taper: f32,
    /// Seconds for the trail to catch up to the cursor when stopped or moving slowly. 0 = disabled.
    #[serde(default = "default_trail_kill_time")]
    pub kill_time: f32,
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
    /// How quickly swinging settles. 0 = endless swing, 1 = instant stop.
    #[serde(default = "default_damping")]
    pub damping: f32,
    #[serde(default = "default_string_color")]
    pub string_color: [u8; 4],
    #[serde(default = "default_string_width")]
    pub string_width: f32,
    /// How much each rope segment can compress or stretch. Higher = more give.
    #[serde(default = "default_string_slack")]
    pub string_slack: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_fps")]
    pub fps: u32,
    /// Hide overlay after this many seconds without cursor movement (e.g. YouTube idle hide). 0 = off.
    #[serde(default = "default_hide_after_idle_secs")]
    pub hide_after_idle_secs: f32,
}

fn default_true() -> bool {
    true
}

fn default_trail_color() -> [u8; 4] {
    [255, 255, 255, 255]
}

fn default_trail_max_length() -> f32 {
    300.0
}

fn default_trail_width() -> f32 {
    5.0
}

fn default_trail_taper() -> f32 {
    1.0
}

fn default_trail_kill_time() -> f32 {
    0.10
}

fn default_string_length() -> f32 {
    90.0
}

fn default_avatar_size() -> f32 {
    25.0
}

fn default_gravity() -> f32 {
    900.0
}

fn default_damping() -> f32 {
    0.15
}

fn default_string_color() -> [u8; 4] {
    [220, 220, 220, 220]
}

fn default_string_width() -> f32 {
    1.0
}

fn default_string_slack() -> f32 {
    0.40
}

fn default_fps() -> u32 {
    60
}

fn default_hide_after_idle_secs() -> f32 {
    3.0
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
            max_length: default_trail_max_length(),
            width: default_trail_width(),
            taper: default_trail_taper(),
            kill_time: default_trail_kill_time(),
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
            string_slack: default_string_slack(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            fps: default_fps(),
            hide_after_idle_secs: default_hide_after_idle_secs(),
        }
    }
}

impl TrailConfig {
    pub fn reset_defaults_preserving_state(&mut self) {
        let enabled = self.enabled;
        *self = Self::default();
        self.enabled = enabled;
    }
}

impl AvatarConfig {
    pub fn reset_defaults_preserving_state(&mut self) {
        let enabled = self.enabled;
        let image_path = self.image_path.clone();
        *self = Self::default();
        self.enabled = enabled;
        self.image_path = image_path;
    }
}

impl WindowConfig {
    pub fn reset_defaults(&mut self) {
        *self = Self::default();
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
