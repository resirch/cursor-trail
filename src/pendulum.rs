use crate::config::AvatarConfig;
use crate::math::Vec2;
use crate::render::{FrameBuffer, Sprite};

const ROPE_SEGMENTS: usize = 10;
const CONSTRAINT_ITERATIONS: usize = 12;

pub struct PendulumAvatar {
    positions: Vec<Vec2>,
    previous_positions: Vec<Vec2>,
    last_anchor: Vec2,
    initialized: bool,
}

impl PendulumAvatar {
    pub fn new() -> Self {
        Self {
            positions: vec![Vec2::ZERO; ROPE_SEGMENTS],
            previous_positions: vec![Vec2::ZERO; ROPE_SEGMENTS],
            last_anchor: Vec2::ZERO,
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

        let dt = dt.clamp(1.0 / 360.0, 1.0 / 30.0);
        let drag = config.damping.clamp(0.0, 1.0);
        let retention = if drag <= f32::EPSILON {
            1.0
        } else {
            (1.0 - drag * dt * 18.0).clamp(0.0, 1.0)
        };

        if !self.initialized {
            let segment_length = config.string_length / (ROPE_SEGMENTS - 1) as f32;
            for i in 0..ROPE_SEGMENTS {
                self.positions[i] = anchor + Vec2::new(0.0, i as f32 * segment_length);
                self.previous_positions[i] = self.positions[i];
            }
            self.last_anchor = anchor;
            self.initialized = true;
            return;
        }

        let anchor_delta = anchor - self.last_anchor;
        self.last_anchor = anchor;
        self.positions[0] = anchor;

        for i in 1..ROPE_SEGMENTS {
            let mut velocity =
                (self.positions[i] - self.previous_positions[i]) * retention;

            if i <= 2 {
                velocity += anchor_delta * (0.85 / i as f32);
            }

            self.previous_positions[i] = self.positions[i];

            let weight = (i as f32 / (ROPE_SEGMENTS - 1) as f32).powi(2);
            self.positions[i] += velocity;

            if config.gravity.abs() > f32::EPSILON {
                self.positions[i].y += config.gravity * weight * dt * dt;
            }
        }

        let rest_segment = config.string_length / (ROPE_SEGMENTS - 1) as f32;
        let slack = config.string_slack.clamp(0.05, 1.0);
        let max_segment = rest_segment * (1.0 + slack);

        for _ in 0..CONSTRAINT_ITERATIONS {
            for i in 0..ROPE_SEGMENTS - 1 {
                let (left, right) = self.positions.split_at_mut(i + 1);
                satisfy_segment(
                    &mut left[i],
                    &mut right[0],
                    rest_segment,
                    max_segment,
                    i == 0,
                );
            }
            self.positions[0] = anchor;
        }
    }

    pub fn draw(
        &self,
        frame: &mut FrameBuffer,
        _anchor: Vec2,
        config: &AvatarConfig,
        sprite: Option<&Sprite>,
    ) {
        if !config.enabled || !self.initialized {
            return;
        }

        for window in self.positions.windows(2) {
            frame.draw_line(
                window[0],
                window[1],
                config.string_width,
                config.string_color,
            );
        }

        let avatar_position = *self.positions.last().unwrap_or(&self.positions[0]);
        let half = config.size * 0.5;

        if let Some(sprite) = sprite {
            frame.blit_sprite(
                avatar_position.x - half,
                avatar_position.y - half,
                config.size,
                config.size,
                sprite,
            );
        } else {
            frame.fill_circle(avatar_position, half, [255, 200, 120, 255]);
            frame.stroke_circle(avatar_position, half, 2.0, [80, 50, 20, 255]);
        }
    }
}

fn satisfy_segment(
    start: &mut Vec2,
    end: &mut Vec2,
    rest_length: f32,
    max_length: f32,
    pin_start: bool,
) {
    let delta = *end - *start;
    let distance = delta.length();
    if distance <= f32::EPSILON {
        return;
    }

    let target = if distance > max_length {
        max_length
    } else {
        rest_length
    };

    let correction = delta * ((distance - target) / distance);

    if pin_start {
        *end -= correction;
    } else {
        *start += correction * 0.5;
        *end -= correction * 0.5;
    }
}
