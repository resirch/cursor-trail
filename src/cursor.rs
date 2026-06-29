use crate::config::WindowConfig;
use std::time::Instant;

const OS_HIDDEN_FRAMES: u32 = 2;

pub struct CursorTracker {
    last_screen_pos: (i32, i32),
    last_move_at: Instant,
    os_hidden_streak: u32,
}

impl CursorTracker {
    pub fn new() -> Self {
        Self {
            last_screen_pos: (i32::MIN, i32::MIN),
            last_move_at: Instant::now(),
            os_hidden_streak: 0,
        }
    }

    pub fn should_hide_overlay(
        &mut self,
        window: &WindowConfig,
        screen_pos: (i32, i32),
        os_visible: bool,
    ) -> bool {
        if screen_pos != self.last_screen_pos {
            self.last_screen_pos = screen_pos;
            self.last_move_at = Instant::now();
        }

        if os_visible {
            self.os_hidden_streak = 0;
        } else {
            self.os_hidden_streak += 1;
        }

        let os_hidden = self.os_hidden_streak >= OS_HIDDEN_FRAMES;
        let idle_hidden = window.hide_after_idle_secs > 0.0
            && self.last_move_at.elapsed().as_secs_f32() >= window.hide_after_idle_secs;

        os_hidden || idle_hidden
    }
}

pub fn query_cursor_state() -> anyhow::Result<(i32, i32, bool)> {
    let screen_pos = crate::overlay::get_cursor_position()?;
    let os_visible = crate::overlay::is_cursor_visible()?;
    Ok((screen_pos.0, screen_pos.1, os_visible))
}

pub fn screen_to_overlay(screen_pos: (i32, i32), origin: (i32, i32)) -> crate::math::Vec2 {
    crate::math::Vec2::new(
        (screen_pos.0 - origin.0) as f32,
        (screen_pos.1 - origin.1) as f32,
    )
}