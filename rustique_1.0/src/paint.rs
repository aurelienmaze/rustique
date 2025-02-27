use eframe::egui;
use egui::{Color32, TextureHandle, TextureOptions, Rect, Pos2, Vec2};
use image::{ImageBuffer, Rgba};
use std::collections::VecDeque;
use std::time::Instant;
use rfd::FileDialog;

use crate::canvas::Canvas;

pub const CHECKERBOARD_SIZE: usize = 8;

#[derive(PartialEq, Clone, Copy)]
pub enum Tool {
    Brush,
    Eraser,
    PaintBucket,
    ColorPicker,
}

pub struct PaintApp {
    pub canvas: Canvas,
    pub current_tool: Tool,
    pub primary_color: Color32,
    pub brush_size: i32,
    pub eraser_size: i32,
    pub last_position: Option<(i32, i32)>,
    pub is_drawing: bool,
    pub last_action_time: Instant,
    pub texture: Option<TextureHandle>,
    pub texture_dirty: bool,
    pub zoom: f32,
    pub pan: Vec2,
    pub has_unsaved_changes: bool,
}

impl PaintApp {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            canvas: Canvas::new(width as usize, height as usize),
            current_tool: Tool::Brush,
            primary_color: Color32::BLACK,
            brush_size: 3,
            eraser_size: 3,
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
            has_unsaved_changes: false,
        }
    }
    
    pub fn save_as_png(&mut self, path: &str) -> Result<(), String> {
        let path_with_ext = if !path.to_lowercase().ends_with(".png") {
            format!("{}.png", path)
        } else {
            path.to_string()
        };

        let width = self.canvas.width;
        let height = self.canvas.height;
        let mut img = ImageBuffer::new(width as u32, height as u32);
        
        for y in 0..height {
            for x in 0..width {
                let color = self.canvas.get(x, y).unwrap_or(Color32::TRANSPARENT);
                img.put_pixel(x as u32, y as u32, Rgba([color.r(), color.g(), color.b(), color.a()]));
            }
        }

        match img.save(path_with_ext) {
            Ok(_) => {
                self.has_unsaved_changes = false;
                Ok(())
            },
            Err(e) => Err(format!("Error saving PNG: {}", e)),
        }
    }

    pub fn draw_line(&mut self, start: (i32, i32), end: (i32, i32), color: Color32) {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        let mut points = Vec::new();
        
        loop {
            points.push((x, y));
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
        
        let fill_color = if self.current_tool == Tool::Eraser { None } else { Some(color) };
        for &(px, py) in &points {
            self.draw_point_with_color(px, py, fill_color);
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    pub fn draw_point(&mut self, x: i32, y: i32) {
        let fill_color = if self.current_tool == Tool::Eraser { None } else { Some(self.primary_color) };
        self.draw_point_with_color(x, y, fill_color);
    }
    
    pub fn draw_point_with_color(&mut self, x: i32, y: i32, fill_color: Option<Color32>) {
        let width = self.canvas.width as i32;
        let height = self.canvas.height as i32;
        let size = if self.current_tool == Tool::Eraser { self.eraser_size } else { self.brush_size };
        let size_squared = size * size;
        
        let mut pixels = Vec::new();
        for dy in -size..=size {
            for dx in -size..=size {
                if dx*dx + dy*dy <= size_squared {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        pixels.push((nx as usize, ny as usize));
                    }
                }
            }
        }
        
        for (nx, ny) in pixels {
            self.canvas.set(nx, ny, fill_color);
        }
        
        self.has_unsaved_changes = true;
        self.texture_dirty = true;
    }

    pub fn paint_bucket(&mut self, x: usize, y: usize) {
        if x >= self.canvas.width || y >= self.canvas.height {
            return;
        }
        
        let target_color = self.canvas.get(x, y);
        let fill_color = Some(self.primary_color);
        
        if target_color == fill_color {
            return;
        }
        
        let mut queue = VecDeque::with_capacity(1024);
        let mut visited = vec![false; self.canvas.width * self.canvas.height];
        queue.push_back((x, y));
        
        while let Some((cx, cy)) = queue.pop_front() {
            let idx = cy * self.canvas.width + cx;
            if visited[idx] || self.canvas.get(cx, cy) != target_color {
                continue;
            }
            
            visited[idx] = true;
            self.canvas.set(cx, cy, fill_color);
            
            if cx > 0 { queue.push_back((cx - 1, cy)); }
            if cx + 1 < self.canvas.width { queue.push_back((cx + 1, cy)); }
            if cy > 0 { queue.push_back((cx, cy - 1)); }
            if cy + 1 < self.canvas.height { queue.push_back((cx, cy + 1)); }
        }
        
        self.last_action_time = Instant::now();
        self.has_unsaved_changes = true;
        self.texture_dirty = true;
    }

    pub fn pick_color(&mut self, x: usize, y: usize) {
        if let Some(color) = self.canvas.get(x, y) {
            self.primary_color = color;
        }
    }

    pub fn update_texture(&mut self, ctx: &egui::Context) {
        if self.texture_dirty {
            let width = self.canvas.width;
            let height = self.canvas.height;
            
            let mut image_data = vec![0_u8; width * height * 4];
            
            for y in 0..height {
                for x in 0..width {
                    let color = if let Some(pixel) = self.canvas.get(x, y) {
                        pixel
                    } else {
                        let checker_x = x / CHECKERBOARD_SIZE;
                        let checker_y = y / CHECKERBOARD_SIZE;
                        if (checker_x + checker_y) % 2 == 0 {
                            Color32::from_gray(200)
                        } else {
                            Color32::from_gray(160)
                        }
                    };
                    
                    let idx = (y * width + x) * 4;
                    image_data[idx] = color.r();
                    image_data[idx + 1] = color.g();
                    image_data[idx + 2] = color.b();
                    image_data[idx + 3] = color.a();
                }
            }
            
            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &image_data);
            self.texture = Some(ctx.load_texture("canvas", color_image, TextureOptions::NEAREST));
            
            self.texture_dirty = false;
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_texture(ctx);

        egui::SidePanel::right("tools_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Tools");
                if ui.button("Brush").clicked() {
                    self.current_tool = Tool::Brush;
                }
                if ui.button("Eraser").clicked() {
                    self.current_tool = Tool::Eraser;
                }
                if ui.button("Paint Bucket").clicked() {
                    self.current_tool = Tool::PaintBucket;
                }
                if ui.button("Color Picker").clicked() {
                    self.current_tool = Tool::ColorPicker;
                }
                
                ui.separator();
                ui.label("Save Options");
                if ui.button("Save PNG").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("PNG Image", &["png"])
                        .set_directory("/")
                        .save_file() {
                        match self.save_as_png(path.to_str().unwrap()) {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("Error saving PNG: {}", e);
                            }
                        }
                    }
                }
                
                ui.separator();
                
                ui.add_space(10.0);
                ui.label("Brush Size:");
                ui.add(egui::DragValue::new(&mut self.brush_size).speed(0.1).clamp_range(1..=500));
                
                ui.add_space(10.0);
                ui.label("Eraser Size:");
                ui.add(egui::DragValue::new(&mut self.eraser_size).speed(0.1).clamp_range(1..=500));
                
                ui.add_space(10.0);
                ui.label("Color:");
                ui.color_edit_button_srgba(&mut self.primary_color);
                
                ui.add_space(10.0);
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.1..=10.0).logarithmic(true));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let canvas_width = self.canvas.width as f32;
            let canvas_height = self.canvas.height as f32;
            let scale = (available_size.x / canvas_width).min(available_size.y / canvas_height);
            let scaled_size = Vec2::new(canvas_width * scale * self.zoom, canvas_height * scale * self.zoom);
            let canvas_rect = Rect::from_center_size(
                ui.available_rect_before_wrap().center() + self.pan,
                scaled_size,
            );

            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            if let Some(texture) = &self.texture {
                painter.image(texture.id(), canvas_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            }

            let to_canvas = egui::emath::RectTransform::from_to(
                canvas_rect,
                Rect::from_min_size(Pos2::ZERO, Vec2::new(canvas_width, canvas_height)),
            );

            if response.dragged_by(egui::PointerButton::Middle) {
                self.pan += response.drag_delta();
            }

            if (response.dragged() || response.clicked()) && 
               !(response.dragged_by(egui::PointerButton::Middle) || 
                 response.clicked_by(egui::PointerButton::Middle)) {
                if let Some(pos) = response.interact_pointer_pos() {
                    let canvas_pos = to_canvas.transform_pos(pos);
                    let x = canvas_pos.x as usize;
                    let y = canvas_pos.y as usize;
                    
                    if x < self.canvas.width && y < self.canvas.height {
                        match self.current_tool {
                            Tool::PaintBucket => self.paint_bucket(x, y),
                            Tool::ColorPicker => self.pick_color(x, y),
                            _ => {
                                let (x, y) = (canvas_pos.x as i32, canvas_pos.y as i32);
                                if let Some(last_pos) = self.last_position {
                                    self.draw_line(last_pos, (x, y), self.primary_color);
                                } else {
                                    self.draw_point(x, y);
                                }
                                self.last_position = Some((x, y));
                            }
                        }
                        self.is_drawing = true;
                    }
                }
            } else if self.is_drawing {
                self.is_drawing = false;
                self.last_position = None;
            }

            let delta = ui.input(|i| i.scroll_delta.y);
            if delta != 0.0 {
                let zoom_speed = 0.001;
                let old_zoom = self.zoom;
                self.zoom *= 1.0 + delta * zoom_speed;
                self.zoom = self.zoom.clamp(0.1, 10.0);
                
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let center = ui.available_rect_before_wrap().center();
                    let mouse_offset = mouse_pos - center - self.pan;
                    let zoom_factor = self.zoom / old_zoom;
                    self.pan += mouse_offset * (1.0 - zoom_factor);
                }
            }
        });
    }
}