use crate::config::{AvatarConfig, Config, TrailConfig, WindowConfig};
use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn run_settings_window(
    config_path: PathBuf,
    initial_config: Config,
    apply_tx: Sender<Config>,
) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([540.0, 720.0])
            .with_min_inner_size([420.0, 320.0])
            .with_resizable(true)
            .with_title("Cursor Trail Settings"),
        event_loop_builder: Some(Box::new(|builder| {
            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                builder.with_any_thread(true);
            }
        })),
        ..Default::default()
    };

    eframe::run_native(
        "Cursor Trail Settings",
        options,
        Box::new(move |_ctx| {
            Ok(Box::new(SettingsApp {
                config_path,
                config: initial_config.clone(),
                draft: initial_config,
                apply_tx: apply_tx.clone(),
                status: String::new(),
            }))
        }),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))
}

struct SettingsApp {
    config_path: PathBuf,
    config: Config,
    draft: Config,
    apply_tx: Sender<Config>,
    status: String,
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("settings_actions").show(ctx, |ui| {
            ui.add_space(4.0);
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Save && Apply").clicked() {
                    self.save_and_apply();
                }
                if ui.button("Revert").clicked() {
                    self.draft = self.config.clone();
                    self.status = "Reverted unsaved changes.".to_string();
                }
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cursor Trail Settings");
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    section_header(ui, "Trail", || {
                        self.draft.trail.reset_defaults_preserving_state();
                        self.status = "Trail reset to defaults.".to_string();
                    });
                    ui.group(|ui| {
                        ui.set_min_width(ui.available_width() - 8.0);
                        trail_settings(ui, &mut self.draft.trail);
                    });

                    ui.add_space(12.0);

                    section_header(ui, "Hanging Avatar", || {
                        self.draft.avatar.reset_defaults_preserving_state();
                        self.status = "Avatar reset to defaults.".to_string();
                    });
                    ui.group(|ui| {
                        ui.set_min_width(ui.available_width() - 8.0);
                        avatar_settings(ui, &mut self.draft.avatar);
                    });

                    ui.add_space(12.0);

                    section_header(ui, "Window", || {
                        self.draft.window.reset_defaults();
                        self.status = "Window reset to defaults.".to_string();
                    });
                    ui.group(|ui| {
                        ui.set_min_width(ui.available_width() - 8.0);
                        window_settings(ui, &mut self.draft.window);
                    });

                    ui.add_space(8.0);
                });
        });
    }
}

impl SettingsApp {
    fn save_and_apply(&mut self) {
        match self.draft.save(&self.config_path) {
            Ok(()) => {
                self.config = self.draft.clone();
                let _ = self.apply_tx.send(self.config.clone());
                self.status = "Saved and applied.".to_string();
            }
            Err(error) => {
                self.status = format!("Save failed: {error}");
            }
        }
    }
}

fn section_header(ui: &mut egui::Ui, title: &str, on_reset: impl FnOnce()) {
    ui.horizontal(|ui| {
        ui.strong(title);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Reset defaults").clicked() {
                on_reset();
            }
        });
    });
    ui.add_space(4.0);
}

fn trail_settings(ui: &mut egui::Ui, trail: &mut TrailConfig) {
    ui.checkbox(&mut trail.enabled, "Enabled");
    color_picker_rgba(ui, "Color", &mut trail.color);
    ui.add(
        egui::Slider::new(&mut trail.max_length, 20.0..=500.0).text("Max length (px)"),
    );
    ui.add(
        egui::Slider::new(&mut trail.width, 1.0..=20.0).text("Width (px)"),
    );
    ui.add(
        egui::Slider::new(&mut trail.taper, 0.0..=1.0).text("Taper"),
    );
    ui.label(
        "Taper pinches to a sharp point at the tail. Higher values keep full width closer to the cursor only.",
    );
    ui.add(
        egui::Slider::new(&mut trail.kill_time, 0.0..=1.0).text("Kill time (s)"),
    );
    ui.label("Kill time retracts the trail toward the cursor when stopped. 0 = off.");
}

fn avatar_settings(ui: &mut egui::Ui, avatar: &mut AvatarConfig) {
    ui.checkbox(&mut avatar.enabled, "Enabled");

    let mut path_text = avatar
        .image_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label("Image path");
        if ui
            .add(
                egui::TextEdit::singleline(&mut path_text)
                    .desired_width(ui.available_width() - 80.0),
            )
            .changed()
        {
            avatar.image_path = if path_text.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(path_text.trim()))
            };
        }
    });

    ui.add(
        egui::Slider::new(&mut avatar.string_length, 20.0..=300.0).text("String length"),
    );
    ui.add(
        egui::Slider::new(&mut avatar.string_slack, 0.05..=1.0).text("String slack"),
    );
    ui.label("How much the string can compress and stretch while coiling.");
    ui.add(
        egui::Slider::new(&mut avatar.size, 16.0..=160.0).text("Avatar size"),
    );
    ui.add(
        egui::Slider::new(&mut avatar.gravity, 0.0..=12000.0).text("Gravity"),
    );
    ui.label("Pulls the avatar downward. 0 = floating.");
    ui.add(
        egui::Slider::new(&mut avatar.damping, 0.0..=1.0).text("Swing decay"),
    );
    ui.label("How quickly swinging settles. 0 = endless swing, 1 = instant stop.");
    color_picker_rgba(ui, "String color", &mut avatar.string_color);
    ui.add(
        egui::Slider::new(&mut avatar.string_width, 1.0..=8.0).text("String width"),
    );
}

fn window_settings(ui: &mut egui::Ui, window: &mut WindowConfig) {
    ui.add(egui::Slider::new(&mut window.fps, 15..=360).text("FPS"));
    ui.label("Overlay redraw rate. Higher is smoother but uses more CPU.");
}

fn color_picker_rgba(ui: &mut egui::Ui, label: &str, color: &mut [u8; 4]) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_srgba_unmultiplied(color);
    });
}
