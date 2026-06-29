use crate::config::AvatarConfig;
use crate::math::Vec2;
use crate::render::{FrameBuffer, Sprite};

pub struct PendulumAvatar {
    position: Vec2,
    previous_position: Vec2,
    initialized: bool,
}

impl PendulumAvatar {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            previous_position: Vec2::ZERO,
            initialized: false,
        }
    }

    pub fn reset(&mut self) {
        self.initialized = false;
    }

    pub fn update(&mut self, anchor: Vec2, config: &AvatarConfig, dt: f32) {
        if !config.enabled {
            return;
        }

        if !self.initialized {
            self.position = anchor + Vec2::new(0.0, config.string_length);
            self.previous_position = self.position;
            self.initialized = true;
            return;
        }

        let anchor_delta = anchor - self.previous_anchor(anchor, dt);
        let swing_impulse = anchor_delta * config.swing_boost;

        let velocity = (self.position - self.previous_position) * config.damping + swing_impulse;

        self.previous_position = self.position;
        self.position += velocity;

        if config.gravity.abs() > f32::EPSILON {
            self.position.y += config.gravity * dt * dt;
        }

        self.enforce_string_constraint(anchor, config.string_length);
    }

    fn previous_anchor(&self, current_anchor: Vec2, dt: f32) -> Vec2 {
        // Approximate last anchor from velocity direction for swing response.
        let velocity = self.position - self.previous_position;
        if velocity.length() <= f32::EPSILON || dt <= f32::EPSILON {
            return current_anchor;
        }
        current_anchor - velocity * (dt * 0.5)
    }

    fn enforce_string_constraint(&mut self, anchor: Vec2, string_length: f32) {
        let delta = self.position - anchor;
        let distance = delta.length();

        if distance <= f32::EPSILON {
            self.position = anchor + Vec2::new(0.0, string_length);
            return;
        }

        if (distance - string_length).abs() > f32::EPSILON {
            self.position = anchor + delta.normalized() * string_length;
        }
    }

    pub fn draw(
        &self,
        frame: &mut FrameBuffer,
        anchor: Vec2,
        config: &AvatarConfig,
        sprite: Option<&Sprite>,
    ) {
        if !config.enabled || !self.initialized {
            return;
        }

        frame.draw_line(
            anchor,
            self.position,
            config.string_width,
            config.string_color,
        );

        let half = config.size * 0.5;

        if let Some(sprite) = sprite {
            frame.blit_sprite(
                self.position.x - half,
                self.position.y - half,
                config.size,
                config.size,
                sprite,
            );
        } else {
            frame.fill_circle(
                self.position,
                half,
                [255, 200, 120, 255],
            );
            frame.stroke_circle(
                self.position,
                half,
                2.0,
                [80, 50, 20, 255],
            );
        }
    }
}
