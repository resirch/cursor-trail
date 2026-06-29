use crate::config::{TrailConfig, TrailShape};
use crate::math::Vec2;
use crate::render::FrameBuffer;

pub struct TrailSystem {
    points: Vec<TrailPoint>,
}

struct TrailPoint {
    position: Vec2,
    age: f32,
}

impl TrailSystem {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn update(&mut self, cursor: Vec2, config: &TrailConfig, dt: f32) {
        if !config.enabled {
            self.points.clear();
            return;
        }

        for point in &mut self.points {
            point.age += dt * config.fade_speed;
        }
        self.points.retain(|p| p.age < 1.0);

        let should_add = self
            .points
            .last()
            .map(|last| (last.position - cursor).length() >= config.spacing)
            .unwrap_or(true);

        if should_add {
            self.points.push(TrailPoint {
                position: cursor,
                age: 0.0,
            });
        }

        if self.points.len() > config.max_points {
            let overflow = self.points.len() - config.max_points;
            self.points.drain(0..overflow);
        }
    }

    pub fn draw(&self, frame: &mut FrameBuffer, config: &TrailConfig) {
        if !config.enabled {
            return;
        }

        let base_color = config.color;
        let size = config.point_size;

        for point in &self.points {
            let alpha_factor = 1.0 - point.age;
            if alpha_factor <= 0.0 {
                continue;
            }

            let color = [
                base_color[0],
                base_color[1],
                base_color[2],
                (base_color[3] as f32 * alpha_factor) as u8,
            ];

            match config.shape {
                TrailShape::Circle => frame.fill_circle(point.position, size * alpha_factor, color),
                TrailShape::Square => {
                    let half = size * alpha_factor * 0.5;
                    frame.fill_rect(
                        point.position.x - half,
                        point.position.y - half,
                        half * 2.0,
                        half * 2.0,
                        color,
                    );
                }
                TrailShape::Star => frame.fill_star(point.position, size * alpha_factor, color),
            }
        }
    }
}
