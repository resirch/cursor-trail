use crate::config::TrailConfig;
use crate::math::Vec2;
use crate::render::FrameBuffer;

const SAMPLE_SPACING: f32 = 1.5;

pub struct TrailSystem {
    points: Vec<Vec2>,
    last_cursor: Option<Vec2>,
}

impl TrailSystem {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            last_cursor: None,
        }
    }

    pub fn update(&mut self, cursor: Vec2, config: &TrailConfig, dt: f32) {
        if !config.enabled {
            self.points.clear();
            self.last_cursor = None;
            return;
        }

        let moved = self
            .last_cursor
            .map(|last| (cursor - last).length())
            .unwrap_or(f32::MAX);

        if moved >= SAMPLE_SPACING {
            self.points.push(cursor);
            trim_to_length(&mut self.points, config.max_length);
        } else if config.kill_time > f32::EPSILON && dt > f32::EPSILON {
            let current_length = path_length_to_cursor(&self.points, cursor);
            if current_length > f32::EPSILON {
                let consume = (current_length / config.kill_time) * dt;
                consume_from_tail(&mut self.points, consume);
            }
        }

        self.last_cursor = Some(cursor);
    }

    pub fn draw(&self, frame: &mut FrameBuffer, config: &TrailConfig, cursor: Vec2) {
        if !config.enabled || config.max_length <= 0.0 || config.width <= 0.0 {
            return;
        }

        let mut polyline = self.points.clone();
        if polyline.last().copied() != Some(cursor) {
            polyline.push(cursor);
        }

        if polyline.len() < 2 {
            return;
        }

        let total_length = path_length(&polyline);
        if total_length <= f32::EPSILON {
            return;
        }

        let base_color = config.color;
        let taper = config.taper.clamp(0.0, 1.0);
        let samples = resample_polyline(&polyline, 0.75);

        for sample in samples {
            let along = (sample.distance / total_length).clamp(0.0, 1.0);
            let width = tapered_width(config.width, along, taper);
            if width <= 0.05 {
                continue;
            }

            let alpha = (base_color[3] as f32 * along.sqrt()) as u8;
            if alpha == 0 {
                continue;
            }

            let color = [base_color[0], base_color[1], base_color[2], alpha];
            frame.draw_soft_dot(sample.position, width * 0.5, color);
        }
    }
}

struct PolylineSample {
    position: Vec2,
    distance: f32,
}

fn resample_polyline(polyline: &[Vec2], step: f32) -> Vec<PolylineSample> {
    let total = path_length(polyline);
    if total <= f32::EPSILON {
        return Vec::new();
    }

    let steps = (total / step).ceil() as i32;
    (0..=steps)
        .filter_map(|i| {
            let distance = (i as f32 / steps as f32) * total;
            point_at_distance(polyline, distance).map(|position| PolylineSample { position, distance })
        })
        .collect()
}

fn point_at_distance(polyline: &[Vec2], mut distance: f32) -> Option<Vec2> {
    for window in polyline.windows(2) {
        let start = window[0];
        let end = window[1];
        let segment_length = (end - start).length();
        if segment_length <= f32::EPSILON {
            continue;
        }

        if distance <= segment_length {
            let t = distance / segment_length;
            return Some(start + (end - start) * t);
        }

        distance -= segment_length;
    }

    polyline.last().copied()
}

fn tapered_width(base_width: f32, along: f32, taper: f32) -> f32 {
    if taper <= f32::EPSILON {
        return base_width;
    }

    let head_fraction = (1.0 - taper).powf(1.5) * 0.35 + 0.015;
    if along >= 1.0 - head_fraction {
        return base_width;
    }

    let taper_span = (1.0 - head_fraction).max(0.001);
    let t = (along / taper_span).clamp(0.0, 1.0);
    let exponent = 2.0 + taper * 8.0;
    base_width * t.powf(exponent)
}

fn trim_to_length(points: &mut Vec<Vec2>, max_length: f32) {
    while points.len() >= 2 && path_length(points) > max_length {
        points.remove(0);
    }
}

fn consume_from_tail(points: &mut Vec<Vec2>, mut amount: f32) {
    while amount > f32::EPSILON && points.len() >= 2 {
        let segment_length = (points[1] - points[0]).length();
        if segment_length <= f32::EPSILON {
            points.remove(0);
            continue;
        }

        if amount >= segment_length {
            amount -= segment_length;
            points.remove(0);
        } else {
            let t = amount / segment_length;
            points[0] = points[0] + (points[1] - points[0]) * t;
            amount = 0.0;
        }
    }

    if points.len() == 1 && amount > f32::EPSILON {
        points.clear();
    }
}

fn path_length(points: &[Vec2]) -> f32 {
    points
        .windows(2)
        .map(|window| (window[1] - window[0]).length())
        .sum()
}

fn path_length_to_cursor(points: &[Vec2], cursor: Vec2) -> f32 {
    if points.is_empty() {
        return 0.0;
    }

    let mut length = path_length(points);
    if points.last().copied() != Some(cursor) {
        length += (cursor - *points.last().unwrap()).length();
    }
    length
}
