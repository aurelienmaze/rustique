mod main_menu;

use eframe::egui;
use egui::{Color32, TextureHandle, TextureOptions, Rect, Pos2, Vec2};
use image::{ImageBuffer, Rgba};
use std::collections::VecDeque;
use rfd::FileDialog;
use std::time::{Duration, Instant};

use main_menu::MainMenu;

// Constants
const MAX_UNDO_STEPS: usize = 20;
const SAVE_STATE_DELAY: Duration = Duration::from_millis(300);
const CHECKERBOARD_SIZE: usize = 8;
const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 800.0;

// Enum to represent different tools
#[derive(PartialEq, Clone, Copy)]
enum Tool {
    Brush,
    Eraser,
    PaintBucket,
    ColorPicker,
}

// Enum to represent the current state of the application
enum AppState {
    MainMenu(MainMenu),
    Canvas(PaintApp),
}

// Struct to represent the state of the canvas
#[derive(Clone)]
struct CanvasState {
    canvas: Vec<Vec<Option<Color32>>>,
}

// Main struct for the paint application
struct PaintApp {
    current_state: CanvasState,
    undo_stack: Vec<CanvasState>,
    redo_stack: Vec<CanvasState>,
    current_tool: Tool,
    current_color: Color32,
    brush_size: i32,
    eraser_size: i32,
    last_position: Option<(i32, i32)>,
    is_drawing: bool,
    last_action_time: Instant,
    texture: Option<TextureHandle>,
    texture_dirty: bool,
    zoom: f32,
    pan: Vec2,
}

impl PaintApp {
    // Initialize a new PaintApp
    fn new(width: u32, height: u32) -> Self {
        let initial_state = CanvasState {
            canvas: vec![vec![None; width as usize]; height as usize],
        };
        Self {
            current_state: initial_state.clone(),
            undo_stack: vec![initial_state],
            redo_stack: Vec::new(),
            current_tool: Tool::Brush,
            current_color: Color32::BLACK,
            brush_size: 3,
            eraser_size: 3,
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }

    // Save the current state for undo functionality
    fn save_state(&mut self) {
        if self.is_drawing && self.last_action_time.elapsed() >= SAVE_STATE_DELAY {
            self.undo_stack.push(self.current_state.clone());
            if self.undo_stack.len() > MAX_UNDO_STEPS {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.is_drawing = false;
        }
    }

    // Undo the last action
    fn undo(&mut self) {
        if self.undo_stack.len() > 1 {
            let current = self.undo_stack.pop().unwrap();
            self.redo_stack.push(current);
            self.current_state = self.undo_stack.last().unwrap().clone();
            self.texture_dirty = true;
        }
    }

    // Redo the last undone action
    fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.current_state.clone());
            self.current_state = next_state;
            self.texture_dirty = true;
        }
    }

    // Draw a line between two points
    fn draw_line(&mut self, start: (i32, i32), end: (i32, i32)) {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_point(x, y);
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
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    // Draw a single point
    fn draw_point(&mut self, x: i32, y: i32) {
        let height = self.current_state.canvas.len() as i32;
        let width = self.current_state.canvas[0].len() as i32;
        let size = if self.current_tool == Tool::Eraser { self.eraser_size } else { self.brush_size };
        for dy in -size..=size {
            for dx in -size..=size {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < width && ny >= 0 && ny < height {
                    self.current_state.canvas[ny as usize][nx as usize] = if self.current_tool == Tool::Eraser {
                        None
                    } else {
                        Some(self.current_color)
                    };
                }
            }
        }
    }

    // Fill an area with a color (paint bucket tool)
    fn paint_bucket(&mut self, x: usize, y: usize) {
        let target_color = self.current_state.canvas[y][x];
        let fill_color = if self.current_tool == Tool::Eraser {
            None
        } else {
            Some(self.current_color)
        };

        if target_color == fill_color {
            return;
        }

        let mut queue = VecDeque::new();
        queue.push_back((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            if self.current_state.canvas[cy][cx] != target_color {
                continue;
            }

            self.current_state.canvas[cy][cx] = fill_color;

            if cx > 0 { queue.push_back((cx - 1, cy)); }
            if cx < self.current_state.canvas[0].len() - 1 { queue.push_back((cx + 1, cy)); }
            if cy > 0 { queue.push_back((cx, cy - 1)); }
            if cy < self.current_state.canvas.len() - 1 { queue.push_back((cx, cy + 1)); }
        }

        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    // Pick a color from the canvas
    fn pick_color(&mut self, x: usize, y: usize) {
        if let Some(color) = self.current_state.canvas[y][x] {
            self.current_color = color;
        }
    }

    // Save the canvas as a PNG image
    fn save_as_png(&self, path: &str) {
        let width = self.current_state.canvas[0].len();
        let height = self.current_state.canvas.len();
        let mut img = ImageBuffer::new(width as u32, height as u32);

        for (y, row) in self.current_state.canvas.iter().enumerate() {
            for (x, &color_opt) in row.iter().enumerate() {
                let color = color_opt.unwrap_or(Color32::TRANSPARENT);
                img.put_pixel(x as u32, y as u32, Rgba([color.r(), color.g(), color.b(), color.a()]));
            }
        }

        img.save(path).expect("Failed to save image");
    }

    // Update the texture used for rendering
    fn update_texture(&mut self, ctx: &egui::Context) {
        if self.texture_dirty {
            let width = self.current_state.canvas[0].len();
            let height = self.current_state.canvas.len();
            let mut image_data = Vec::with_capacity(width * height * 4);

            for (y, row) in self.current_state.canvas.iter().enumerate() {
                for (x, &pixel_opt) in row.iter().enumerate() {
                    let color = if let Some(pixel) = pixel_opt {
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
                    image_data.extend_from_slice(&[color.r(), color.g(), color.b(), color.a()]);
                }
            }

            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &image_data);
            self.texture = Some(ctx.load_texture("canvas", color_image, TextureOptions::NEAREST));
            self.texture_dirty = false;
        }
    }
}

// Main application struct
struct MyApp {
    state: AppState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu(MainMenu::default()),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::MainMenu(menu) => {
                if let Some((width, height)) = menu.show(ctx) {
                    self.state = AppState::Canvas(PaintApp::new(width, height));
                }
            }
            AppState::Canvas(paint_app) => {
                paint_app.update_texture(ctx);

                egui::SidePanel::right("tools_panel").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Tools");
                        if ui.button("Brush").clicked() {
                            paint_app.current_tool = Tool::Brush;
                        }
                        if ui.button("Eraser").clicked() {
                            paint_app.current_tool = Tool::Eraser;
                        }
                        if ui.button("Paint Bucket").clicked() {
                            paint_app.current_tool = Tool::PaintBucket;
                        }
                        if ui.button("Color Picker").clicked() {
                            paint_app.current_tool = Tool::ColorPicker;
                        }
                        if ui.button("Save").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("PNG Image", &["png"])
                                .set_directory("/")
                                .save_file() {
                                paint_app.save_as_png(path.to_str().unwrap());
                            }
                        }
                        
                        ui.add_space(10.0);
                        ui.label("Brush Size:");
                        ui.add(egui::DragValue::new(&mut paint_app.brush_size).speed(0.1).clamp_range(1..=100));
                        
                        ui.add_space(10.0);
                        ui.label("Eraser Size:");
                        ui.add(egui::DragValue::new(&mut paint_app.eraser_size).speed(0.1).clamp_range(1..=100));
                        
                        ui.add_space(10.0);
                        ui.label("Zoom:");
                        ui.add(egui::Slider::new(&mut paint_app.zoom, 0.1..=10.0).logarithmic(true));
                    });
                });

                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Undo").clicked() {
                            paint_app.undo();
                        }
                        if ui.button("Redo").clicked() {
                            paint_app.redo();
                        }
                        ui.color_edit_button_srgba(&mut paint_app.current_color);
                    });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    let available_size = ui.available_size();
                    let canvas_width = paint_app.current_state.canvas[0].len() as f32;
                    let canvas_height = paint_app.current_state.canvas.len() as f32;
                    let scale = (available_size.x / canvas_width).min(available_size.y / canvas_height);
                    let scaled_size = Vec2::new(canvas_width * scale * paint_app.zoom, canvas_height * scale * paint_app.zoom);
                    let canvas_rect = Rect::from_center_size(
                        ui.available_rect_before_wrap().center() + paint_app.pan,
                        scaled_size,
                    );

                    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

                    if let Some(texture) = &paint_app.texture {
                        painter.image(texture.id(), canvas_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    }

                    let to_canvas = egui::emath::RectTransform::from_to(
                        canvas_rect,
                        Rect::from_min_size(Pos2::ZERO, Vec2::new(canvas_width, canvas_height)),
                    );

                    if response.dragged_by(egui::PointerButton::Middle) {
                        paint_app.pan += response.drag_delta();
                    }

                    if response.clicked() {
                        paint_app.is_drawing = true;
                        paint_app.save_state();
                    }

                    if response.dragged() || response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let canvas_pos = to_canvas.transform_pos(pos);
                            let (x, y) = (canvas_pos.x as usize, canvas_pos.y as usize);
                            
                            if x < paint_app.current_state.canvas[0].len() && y < paint_app.current_state.canvas.len() {
                                match paint_app.current_tool {
                                    Tool::PaintBucket => paint_app.paint_bucket(x, y),
                                    Tool::ColorPicker => paint_app.pick_color(x, y),
                                    _ => {
                                        let (x, y) = (canvas_pos.x as i32, canvas_pos.y as i32);
                                        if let Some(last_pos) = paint_app.last_position {
                                            paint_app.draw_line(last_pos, (x, y));
                                        } else {
                                            paint_app.draw_point(x, y);
                                        }
                                        paint_app.last_position = Some((x, y));
                                    }
                                }
                                paint_app.is_drawing = true;
                            }
                        }
                    } else {
                        paint_app.save_state();
                        paint_app.last_position = None;
                    }

                    // Handle zooming with mouse wheel
                    if let delta = ui.input(|i| i.scroll_delta.y) {
                        paint_app.zoom *= 1.0 + delta * 0.001;
                        paint_app.zoom = paint_app.zoom.clamp(0.1, 10.0);
                    }                                        
                });
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        ..Default::default()
    };
    eframe::run_native(
        "Rustique Paint",
        native_options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

