use eframe::egui;
use egui::{Color32, Vec2, Stroke, RichText};
use crate::localization::{Language, get_text};

// Enum to represent actions from the main menu
pub enum MenuAction {
    NewCanvas(u32, u32),
    OpenFile,
}

// Result from main menu
pub enum MenuResult {
    Action(MenuAction),
    LanguageChanged(Language),
}

pub struct MainMenu {
    width: u32,
    height: u32,
    logo: Option<egui::TextureHandle>,
    logo_size: f32,
    language: Language,
}

impl MainMenu {
    pub fn new(language: Language) -> Self {
        Self {
            width: 800,
            height: 600,
            logo: None,
            logo_size: 150.0, // This is now the target height of the logo
            language,
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context) -> Option<MenuResult> {
        let mut result = None;

        if self.logo.is_none() {
            self.logo = load_image_from_path(ctx, "rustique.png");
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Set up a more attractive theme
            ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
            ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(40, 40, 40);
            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 80);
            ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(100, 100, 100);
            
            // Add a background with a solid color
            let rect = ui.max_rect();
            let bg_color = Color32::from_rgb(35, 35, 60);
            ui.painter().rect_filled(rect, 0.0, bg_color);
            
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);

                if let Some(logo) = &self.logo {
                    let aspect_ratio = logo.size()[0] as f32 / logo.size()[1] as f32;
                    let logo_width = self.logo_size * aspect_ratio;
                    let size = Vec2::new(logo_width, self.logo_size);
                    ui.image(logo, size);
                } else {
                    // If no logo, draw a nice title with an underline
                    let title = RichText::new("RUSTIQUE PAINT")
                        .size(48.0)
                        .color(Color32::WHITE)
                        .strong();
                    
                    ui.add_space(40.0);
                    ui.heading(title);
                    
                    // Draw decorative underline
                    let text_size = egui::TextStyle::Heading.resolve(ui.style()).size;
                    let line_width = 240.0;  // Width of underline
                    let line_start = ui.min_rect().center().x - line_width / 2.0;
                    let line_y = ui.min_rect().bottom() + 8.0;
                    
                    ui.painter().line_segment(
                        [egui::pos2(line_start, line_y), egui::pos2(line_start + line_width, line_y)],
                        Stroke::new(2.0, Color32::from_rgb(200, 200, 255))
                    );
                }

                ui.add_space(40.0);

                // Language selection
                ui.horizontal(|ui| {
                    ui.label(RichText::new(get_text("language", self.language)).size(16.0));
                    if ui.button(RichText::new("Français").size(16.0)).clicked() {
                        self.language = Language::French;
                        result = Some(MenuResult::LanguageChanged(Language::French));
                    }
                    if ui.button(RichText::new("English").size(16.0)).clicked() {
                        self.language = Language::English;
                        result = Some(MenuResult::LanguageChanged(Language::English));
                    }
                });
                
                ui.add_space(20.0);

                // Main panel
                egui::Frame::group(ui.style())
                    .inner_margin(20.0)
                    .rounding(10.0)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(100, 100, 180)))
                    .show(ui, |ui| {
                        ui.set_width(350.0);
                        ui.vertical_centered(|ui| {
                            ui.heading(RichText::new(get_text("canvas_dimensions", self.language)).size(20.0));
                            ui.add_space(15.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(get_text("width", self.language)).size(16.0));
                                ui.add(egui::DragValue::new(&mut self.width).speed(1).clamp_range(100..=4000));
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(get_text("height", self.language)).size(16.0));
                                ui.add(egui::DragValue::new(&mut self.height).speed(1).clamp_range(100..=4000));
                            });

                            ui.add_space(25.0);

                            // Big button with nice styling
                            let button_text = RichText::new(get_text("create_new_canvas", self.language))
                                .size(18.0)
                                .color(Color32::WHITE);
                            
                            if ui.add(egui::Button::new(button_text).min_size(Vec2::new(200.0, 36.0))).clicked() {
                                result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
                            }

                            ui.add_space(15.0);
                            
                            let open_file_text = RichText::new(get_text("open_file", self.language))
                                .size(16.0);
                            
                            if ui.add(egui::Button::new(open_file_text).min_size(Vec2::new(180.0, 30.0))).clicked() {
                                result = Some(MenuResult::Action(MenuAction::OpenFile));
                            }
                        });
                    });
                
                ui.add_space(30.0);
                
                // Footer info
                ui.label(RichText::new("© 2023 Rustique Paint").size(14.0).color(Color32::from_rgb(180, 180, 200)));
            });
        });

        result
    }
}

// Optimized image loading function that provides better error handling
fn load_image_from_path(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    match image::open(path) {
        Ok(image) => {
            let image_buffer = image.to_rgba8();
            let size = [image_buffer.width() as _, image_buffer.height() as _];
            let image_data = egui::ColorImage::from_rgba_unmultiplied(size, image_buffer.as_flat_samples().as_slice());
            Some(ctx.load_texture(
                "logo",
                image_data,
                egui::TextureOptions::LINEAR
            ))
        },
        Err(_) => {
            // Create a default logo
            let width = 200;
            let height = 100;
            let mut pixels = vec![0; width * height * 4];
            
            // Fill with a gradient
            for y in 0..height {
                for x in 0..width {
                    let r = (x * 255 / width) as u8;
                    let g = (y * 255 / height) as u8;
                    let b = 128_u8;
                    let idx = (y * width + x) * 4;
                    pixels[idx] = r;
                    pixels[idx + 1] = g;
                    pixels[idx + 2] = b;
                    pixels[idx + 3] = 255;
                }
            }
            
            let image_data = egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels);
            Some(ctx.load_texture(
                "default_logo",
                image_data,
                egui::TextureOptions::LINEAR
            ))
        }
    }
}