use crate::math::Vec2;
use image::RgbaImage;

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn as_bgra_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in &self.pixels {
            let a = ((pixel >> 24) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            bytes.extend_from_slice(&[b, g, r, a]);
        }
        bytes
    }

    fn plot(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = (y as u32 * self.width + x as u32) as usize;
        self.pixels[idx] = blend_pixel(self.pixels[idx], color);
    }

    pub fn fill_circle(&mut self, center: Vec2, radius: f32, color: [u8; 4]) {
        if radius <= 0.0 {
            return;
        }

        let r = radius.ceil() as i32;
        let cx = center.x.round() as i32;
        let cy = center.y.round() as i32;
        let radius_sq = radius * radius;

        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                let dx = x as f32 - center.x;
                let dy = y as f32 - center.y;
                if dx * dx + dy * dy <= radius_sq {
                    self.plot(x, y, color);
                }
            }
        }
    }

    pub fn stroke_circle(&mut self, center: Vec2, radius: f32, width: f32, color: [u8; 4]) {
        let inner = (radius - width * 0.5).max(0.0);
        let outer = radius + width * 0.5;
        let inner_sq = inner * inner;
        let outer_sq = outer * outer;

        let r = outer.ceil() as i32;
        let cx = center.x.round() as i32;
        let cy = center.y.round() as i32;

        for y in (cy - r)..=(cy + r) {
            for x in (cx - r)..=(cx + r) {
                let dx = x as f32 - center.x;
                let dy = y as f32 - center.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= outer_sq && dist_sq >= inner_sq {
                    self.plot(x, y, color);
                }
            }
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [u8; 4]) {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = (x + width).ceil() as i32;
        let y1 = (y + height).ceil() as i32;

        for py in y0..y1 {
            for px in x0..x1 {
                self.plot(px, py, color);
            }
        }
    }

    pub fn fill_star(&mut self, center: Vec2, size: f32, color: [u8; 4]) {
        let points = 5;
        let outer = size;
        let inner = size * 0.45;
        let mut vertices = Vec::with_capacity(points * 2);

        for i in 0..points * 2 {
            let radius = if i % 2 == 0 { outer } else { inner };
            let angle = std::f32::consts::PI * i as f32 / points as f32
                - std::f32::consts::FRAC_PI_2;
            vertices.push(Vec2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }

        fill_polygon(self, &vertices, color);
    }

    pub fn draw_line(&mut self, start: Vec2, end: Vec2, width: f32, color: [u8; 4]) {
        let delta = end - start;
        let length = delta.length();
        if length <= f32::EPSILON {
            return;
        }

        let step = 0.5f32;
        let steps = (length / step).ceil() as i32;
        let radius = width * 0.5;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let point = start + delta * t;
            self.draw_soft_dot(point, radius, color);
        }
    }

    pub fn draw_soft_dot(&mut self, center: Vec2, radius: f32, color: [u8; 4]) {
        if radius <= 0.0 || color[3] == 0 {
            return;
        }

        let extent = radius + 1.0;
        let min_x = (center.x - extent).floor() as i32;
        let max_x = (center.x + extent).ceil() as i32;
        let min_y = (center.y - extent).floor() as i32;
        let max_y = (center.y + extent).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - center.x;
                let dy = y as f32 + 0.5 - center.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let edge = radius + 0.5;
                let coverage = (edge - dist).clamp(0.0, 1.0);
                if coverage <= 0.0 {
                    continue;
                }

                let alpha = (color[3] as f32 * coverage) as u8;
                if alpha == 0 {
                    continue;
                }

                self.plot(
                    x,
                    y,
                    [color[0], color[1], color[2], alpha],
                );
            }
        }
    }

    pub fn blit_sprite(&mut self, x: f32, y: f32, width: f32, height: f32, sprite: &Sprite) {
        let dest_w = width.max(1.0) as u32;
        let dest_h = height.max(1.0) as u32;
        let start_x = x.floor() as i32;
        let start_y = y.floor() as i32;

        for dy in 0..dest_h {
            for dx in 0..dest_w {
                let src_x = dx * sprite.width / dest_w;
                let src_y = dy * sprite.height / dest_h;
                let src_idx = (src_y * sprite.width + src_x) as usize;
                let src_pixel = sprite.pixels[src_idx];
                let alpha = ((src_pixel >> 24) & 0xFF) as u8;
                if alpha == 0 {
                    continue;
                }

                let color = [
                    ((src_pixel >> 16) & 0xFF) as u8,
                    ((src_pixel >> 8) & 0xFF) as u8,
                    (src_pixel & 0xFF) as u8,
                    alpha,
                ];

                self.plot(start_x + dx as i32, start_y + dy as i32, color);
            }
        }
    }
}

impl Sprite {
    pub fn from_image(image: &RgbaImage) -> Self {
        let (width, height) = image.dimensions();
        let pixels = image
            .pixels()
            .map(|pixel| {
                let [r, g, b, a] = pixel.0;
                ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
            })
            .collect();

        Self {
            width,
            height,
            pixels,
        }
    }
}

fn fill_polygon(frame: &mut FrameBuffer, vertices: &[Vec2], color: [u8; 4]) {
    if vertices.len() < 3 {
        return;
    }

    let min_y = vertices
        .iter()
        .map(|v| v.y.floor() as i32)
        .min()
        .unwrap_or(0);
    let max_y = vertices
        .iter()
        .map(|v| v.y.ceil() as i32)
        .max()
        .unwrap_or(0);

    for y in min_y..=max_y {
        let mut intersections = Vec::new();
        for i in 0..vertices.len() {
            let a = vertices[i];
            let b = vertices[(i + 1) % vertices.len()];

            if (a.y <= y as f32 && b.y > y as f32) || (b.y <= y as f32 && a.y > y as f32) {
                let t = (y as f32 - a.y) / (b.y - a.y);
                intersections.push(a.x + t * (b.x - a.x));
            }
        }

        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut i = 0;
        while i + 1 < intersections.len() {
            let x0 = intersections[i].floor() as i32;
            let x1 = intersections[i + 1].ceil() as i32;
            for x in x0..=x1 {
                frame.plot(x, y, color);
            }
            i += 2;
        }
    }
}

fn blend_pixel(dst: u32, src: [u8; 4]) -> u32 {
    let dst_a = ((dst >> 24) & 0xFF) as f32 / 255.0;
    let dst_r = ((dst >> 16) & 0xFF) as f32 / 255.0;
    let dst_g = ((dst >> 8) & 0xFF) as f32 / 255.0;
    let dst_b = (dst & 0xFF) as f32 / 255.0;

    let src_a = src[3] as f32 / 255.0;
    if src_a <= 0.0 {
        return dst;
    }

    let src_r = src[0] as f32 / 255.0;
    let src_g = src[1] as f32 / 255.0;
    let src_b = src[2] as f32 / 255.0;

    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        return 0;
    }

    let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
    let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
    let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;

    ((out_a * 255.0) as u32) << 24
        | ((out_r * 255.0) as u32) << 16
        | ((out_g * 255.0) as u32) << 8
        | (out_b * 255.0) as u32
}
