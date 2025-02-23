use eframe::egui;
use egui::{Color32, Vec2};

pub struct MainMenu {
    width: u32,
    height: u32,
    logo: Option<egui::TextureHandle>,
    logo_size: f32,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            logo: None,
            logo_size: 150.0, // This is now the target height of the logo
        }
    }
}

impl MainMenu {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<(u32, u32)> {
        let mut dimensions = None;

        if self.logo.is_none() {
            self.logo = load_image_from_path(ctx, "rustique.png");
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
            ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(40, 40, 40);
            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 80);
            ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(100, 100, 100);

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                if let Some(logo) = &self.logo {
                    let aspect_ratio = logo.size()[0] as f32 / logo.size()[1] as f32;
                    let logo_width = self.logo_size * aspect_ratio;
                    let size = Vec2::new(logo_width, self.logo_size);
                    ui.image(logo, size);
                }

                ui.add_space(20.0);
                ui.heading("Rustique Paint");
                ui.add_space(40.0);

                ui.group(|ui| {
                    ui.set_width(300.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Canvas Dimensions");
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add(egui::DragValue::new(&mut self.width).speed(1).clamp_range(100..=4000));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add(egui::DragValue::new(&mut self.height).speed(1).clamp_range(100..=4000));
                        });

                        ui.add_space(20.0);

                        if ui.button("Create Canvas").clicked() {
                            dimensions = Some((self.width, self.height));
                        }
                    });
                });
            });
        });

        dimensions
    }
}

fn load_image_from_path(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    if let Ok(image) = image::open(path) {
        let image_buffer = image.to_rgba8();
        let size = [image_buffer.width() as _, image_buffer.height() as _];
        let image_data = egui::ColorImage::from_rgba_unmultiplied(size, image_buffer.as_flat_samples().as_slice());
        Some(ctx.load_texture(
            "logo",
            image_data,
            egui::TextureOptions::LINEAR
        ))
    } else {
        None
    }
}
