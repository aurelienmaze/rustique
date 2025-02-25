mod main_menu;

use eframe::egui;
use egui::{Color32, TextureHandle, TextureOptions, Rect, Pos2, Vec2};
use image::{ImageBuffer, Rgba};
use std::collections::VecDeque;
use rfd::FileDialog;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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

// Optimized canvas state structure with flat array
#[derive(Clone)]
struct CanvasState {
    width: usize,
    height: usize,
    data: Vec<Option<Color32>>,
}

impl CanvasState {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![None; width * height],
        }
    }
    
    #[inline]
    fn get(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            None
        }
    }
    
    #[inline]
    fn set(&mut self, x: usize, y: usize, color: Option<Color32>) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = color;
        }
    }
}

// Store changes for efficient undo/redo
#[derive(Clone)]
struct CanvasChange {
    x: usize,
    y: usize,
    old_color: Option<Color32>,
    new_color: Option<Color32>,
}

// Main struct for the paint application
struct PaintApp {
    current_state: CanvasState,
    undo_stack: Vec<Vec<CanvasChange>>,
    redo_stack: Vec<Vec<CanvasChange>>,
    current_changes: Vec<CanvasChange>,
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
        let initial_state = CanvasState::new(width as usize, height as usize);
        Self {
            current_state: initial_state,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_changes: Vec::new(),
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

    // Record a pixel change for undo/redo
    fn record_change(&mut self, x: usize, y: usize, new_color: Option<Color32>) {
        if x < self.current_state.width && y < self.current_state.height {
            let old_color = self.current_state.get(x, y);
            if old_color != new_color {
                self.current_changes.push(CanvasChange {
                    x, y, old_color, new_color
                });
                self.current_state.set(x, y, new_color);
            }
        }
    }

    // Save the current state for undo functionality
    fn save_state(&mut self) {
        if !self.current_changes.is_empty() && self.last_action_time.elapsed() >= SAVE_STATE_DELAY {
            self.undo_stack.push(std::mem::take(&mut self.current_changes));
            self.current_changes = Vec::new();
            if self.undo_stack.len() > MAX_UNDO_STEPS {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.is_drawing = false;
        }
    }

    // Undo the last action
    fn undo(&mut self) {
        if let Some(changes) = self.undo_stack.pop() {
            let mut redo_changes = Vec::with_capacity(changes.len());
            
            // Apply changes in reverse
            for change in changes.iter().rev() {
                let current_color = self.current_state.get(change.x, change.y);
                redo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    old_color: current_color,
                    new_color: change.old_color,
                });
                self.current_state.set(change.x, change.y, change.old_color);
            }
            
            self.redo_stack.push(redo_changes);
            self.texture_dirty = true;
        }
    }

    // Redo the last undone action
    fn redo(&mut self) {
        if let Some(changes) = self.redo_stack.pop() {
            let mut undo_changes = Vec::with_capacity(changes.len());
            
            // Apply changes in reverse
            for change in changes.iter().rev() {
                let current_color = self.current_state.get(change.x, change.y);
                undo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    old_color: current_color,
                    new_color: change.old_color,
                });
                self.current_state.set(change.x, change.y, change.old_color);
            }
            
            self.undo_stack.push(undo_changes);
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

        // For large brush sizes, collect points first and then draw them
        let size = if self.current_tool == Tool::Eraser { self.eraser_size } else { self.brush_size };
        if size > 10 {
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
            
            // Draw each point individually - no parallel processing here to avoid borrow issues
            for &(px, py) in &points {
                self.draw_point(px, py);
            }
        } else {
            // For small brush sizes, original algorithm works fine
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
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    // Draw a single point with optimized circular brush
    fn draw_point(&mut self, x: i32, y: i32) {
        let width = self.current_state.width as i32;
        let height = self.current_state.height as i32;
        let size = if self.current_tool == Tool::Eraser { self.eraser_size } else { self.brush_size };
        let size_squared = size * size;
        let fill_color = if self.current_tool == Tool::Eraser { None } else { Some(self.current_color) };
        
        // Collect all points that need to be modified
        let mut pixels = Vec::new();
        for dy in -size..=size {
            for dx in -size..=size {
                // Use circle equation dx²+dy² ≤ r² for circular brush
                if dx*dx + dy*dy <= size_squared {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        pixels.push((nx as usize, ny as usize));
                    }
                }
            }
        }
        
        // Process all pixels sequentially
        for (nx, ny) in pixels {
            self.record_change(nx, ny, fill_color);
        }
        
        self.texture_dirty = true;
    }

    // Optimized paint bucket fill
    fn paint_bucket(&mut self, x: usize, y: usize) {
        if x >= self.current_state.width || y >= self.current_state.height {
            return;
        }
        
        let target_color = self.current_state.get(x, y);
        let fill_color = if self.current_tool == Tool::Eraser {
            None
        } else {
            Some(self.current_color)
        };
        
        if target_color == fill_color {
            return;
        }
        
        // Pre-allocate for better performance
        let mut queue = VecDeque::with_capacity(1024);
        let mut visited = vec![false; self.current_state.width * self.current_state.height];
        queue.push_back((x, y));
        
        while let Some((cx, cy)) = queue.pop_front() {
            let idx = cy * self.current_state.width + cx;
            if visited[idx] || self.current_state.get(cx, cy) != target_color {
                continue;
            }
            
            visited[idx] = true;
            self.record_change(cx, cy, fill_color);
            
            // Add adjacent pixels to queue
            if cx > 0 { queue.push_back((cx - 1, cy)); }
            if cx + 1 < self.current_state.width { queue.push_back((cx + 1, cy)); }
            if cy > 0 { queue.push_back((cx, cy - 1)); }
            if cy + 1 < self.current_state.height { queue.push_back((cx, cy + 1)); }
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    // Pick a color from the canvas
    fn pick_color(&mut self, x: usize, y: usize) {
        if let Some(color) = self.current_state.get(x, y) {
            self.current_color = color;
        }
    }

    // Optimized PNG saving
    fn save_as_png(&self, path: &str) {
        let width = self.current_state.width;
        let height = self.current_state.height;
        let mut img = ImageBuffer::new(width as u32, height as u32);

        // Process rows one by one (no parallelism to avoid memory conflicts)
        for y in 0..height {
            for x in 0..width {
                let color = self.current_state.get(x, y).unwrap_or(Color32::TRANSPARENT);
                img.put_pixel(x as u32, y as u32, Rgba([color.r(), color.g(), color.b(), color.a()]));
            }
        }

        img.save(path).expect("Failed to save image");
    }

    // Optimized texture update
    fn update_texture(&mut self, ctx: &egui::Context) {
        if self.texture_dirty {
            let width = self.current_state.width;
            let height = self.current_state.height;
            
            // Create the image data
            let mut image_data = vec![0_u8; width * height * 4];
            
            // Process all pixels
            for y in 0..height {
                for x in 0..width {
                    let color = if let Some(pixel) = self.current_state.get(x, y) {
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
                        ui.add(egui::DragValue::new(&mut paint_app.brush_size).speed(0.1).clamp_range(1..=500));
                        
                        ui.add_space(10.0);
                        ui.label("Eraser Size:");
                        ui.add(egui::DragValue::new(&mut paint_app.eraser_size).speed(0.1).clamp_range(1..=500));
                        
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
                    let canvas_width = paint_app.current_state.width as f32;
                    let canvas_height = paint_app.current_state.height as f32;
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

                    // Improved panning
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
                            let x = canvas_pos.x as usize;
                            let y = canvas_pos.y as usize;
                            
                            if x < paint_app.current_state.width && y < paint_app.current_state.height {
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

                    // Improved zooming with mouse wheel - fixed the irrefutable pattern
                    let delta = ui.input(|i| i.scroll_delta.y);
                    if delta != 0.0 {
                        let zoom_speed = 0.001;
                        let old_zoom = paint_app.zoom;
                        paint_app.zoom *= 1.0 + delta * zoom_speed;
                        paint_app.zoom = paint_app.zoom.clamp(0.1, 10.0);
                        
                        // Zoom toward mouse cursor position
                        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                            let center = ui.available_rect_before_wrap().center();
                            let mouse_offset = mouse_pos - center - paint_app.pan;
                            let zoom_factor = paint_app.zoom / old_zoom;
                            paint_app.pan += mouse_offset * (1.0 - zoom_factor);
                        }
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
