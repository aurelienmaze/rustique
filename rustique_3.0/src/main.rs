mod main_menu;
mod localization;

use eframe::egui;
use egui::{Color32, TextureHandle, TextureOptions, Rect, Pos2, Vec2, Stroke};
use image::{ImageBuffer, Rgba, ImageFormat};
use std::collections::VecDeque;
use rfd::FileDialog;
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};

use main_menu::MainMenu;
use localization::{Language, get_text};

// Constants
const MAX_UNDO_STEPS: usize = 20;
const SAVE_STATE_DELAY: Duration = Duration::from_millis(300);
const CHECKERBOARD_SIZE: usize = 8;
const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 800.0;
const MAX_SAVED_COLORS: usize = 16;

// Enum to represent different tools
#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
enum Tool {
    AdvancedBrush,
    Eraser,
    PaintBucket,
    ColorPicker,
    Line,
}

// Enum for Brush Types
#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum BrushType {
    Round,
    Flat,
    Bright,
    Filbert,
    Fan,
    Angle,
    Mop,
    Rigger,
}

impl BrushType {
    // Helper to get all brush types for UI iteration
    pub fn all_variants() -> Vec<BrushType> {
        vec![
            BrushType::Round,
            BrushType::Flat,
            BrushType::Bright,
            BrushType::Filbert,
            BrushType::Fan,
            BrushType::Angle,
            BrushType::Mop,
            BrushType::Rigger,
        ]
    }
}

// Struct for Brush Style
#[derive(Clone, Serialize, Deserialize)]
pub struct BrushStyle {
    pub brush_type: BrushType,
    pub size: f32,
    pub angle: f32,
    pub hardness: f32,
    // Optional future fields
    pub bristle_count: Option<u32>,
    pub taper_strength: Option<f32>,
}

impl Default for BrushStyle {
    fn default() -> Self {
        Self {
            brush_type: BrushType::Round,
            size: 10.0,
            angle: 0.0,
            hardness: 1.0,
            bristle_count: Some(10), // Default bristle count for Fan brush
            taper_strength: None,
        }
    }
}

// Enum to represent supported file formats
#[derive(Debug, Clone, Copy, PartialEq)]
enum FileFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    Gif,
    WebP,
    Rustiq,
    Unknown,
}

impl FileFormat {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => FileFormat::Png,
            "jpg" | "jpeg" => FileFormat::Jpeg,
            "bmp" => FileFormat::Bmp,
            "tiff" | "tif" => FileFormat::Tiff,
            "gif" => FileFormat::Gif,
            "webp" => FileFormat::WebP,
            "rustiq" => FileFormat::Rustiq,
            _ => FileFormat::Unknown,
        }
    }
    
    fn get_image_format(&self) -> Option<ImageFormat> {
        match self {
            FileFormat::Png => Some(ImageFormat::Png),
            FileFormat::Jpeg => Some(ImageFormat::Jpeg),
            FileFormat::Bmp => Some(ImageFormat::Bmp),
            FileFormat::Tiff => Some(ImageFormat::Tiff),
            FileFormat::Gif => Some(ImageFormat::Gif),
            FileFormat::WebP => Some(ImageFormat::WebP),
            _ => None,
        }
    }
    
    fn extension(&self) -> &'static str {
        match self {
            FileFormat::Png => "png",
            FileFormat::Jpeg => "jpg",
            FileFormat::Bmp => "bmp",
            FileFormat::Tiff => "tiff",
            FileFormat::Gif => "gif",
            FileFormat::WebP => "webp",
            FileFormat::Rustiq => "rustiq",
            FileFormat::Unknown => "",
        }
    }
}

// Enum to represent the current state of the application
enum AppState {
    MainMenu(MainMenu),
    Canvas(PaintApp),
}

// Layer structure for storing each canvas layer (sans serde)
#[derive(Clone, PartialEq)]
struct Layer {
    name: String,
    data: Vec<Option<Color32>>,
    visible: bool,
}

// Layer structure for serialization
#[derive(Serialize, Deserialize)]
struct LayerData {
    name: String,
    data: Vec<Option<[u8; 4]>>,
    visible: bool,
}

// Structure for saving and loading .rustiq files

// Old format for migration
#[derive(Serialize, Deserialize)]
struct RustiqueFileV1 {
    width: usize,
    height: usize,
    layers: Vec<LayerData>,
    active_layer_index: usize,
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
    saved_colors: Vec<[u8; 4]>,
    brush_size: i32, // Old field
    eraser_size: i32,
}

// Current format
#[derive(Serialize, Deserialize, Clone)] // Added Clone
struct RustiqueFile {
    width: usize,
    height: usize,
    layers: Vec<LayerData>,
    active_layer_index: usize,
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
    saved_colors: Vec<[u8; 4]>,
    current_brush_style: BrushStyle, // New field
    eraser_size: i32,
}

impl RustiqueFile {
    fn deserialize_with_migration(content: &str) -> Result<Self, String> {
        // Try to deserialize as current format (RustiqueFile)
        match serde_json::from_str::<RustiqueFile>(content) {
            Ok(new_format_file) => Ok(new_format_file),
            Err(_e1) => { // Could log _e1 for debugging
                // If that fails, try to deserialize as old format (RustiqueFileV1)
                match serde_json::from_str::<RustiqueFileV1>(content) {
                    Ok(old_format_file) => {
                        // Migrate from V1 to current
                        Ok(RustiqueFile {
                            width: old_format_file.width,
                            height: old_format_file.height,
                            layers: old_format_file.layers,
                            active_layer_index: old_format_file.active_layer_index,
                            primary_color: old_format_file.primary_color,
                            secondary_color: old_format_file.secondary_color,
                            saved_colors: old_format_file.saved_colors,
                            current_brush_style: BrushStyle {
                                size: old_format_file.brush_size as f32,
                                brush_type: BrushType::Round, // Default type for migration
                                angle: 0.0,                  // Default angle
                                hardness: 1.0,               // Default hardness
                                bristle_count: None,
                                taper_strength: None,
                            },
                            eraser_size: old_format_file.eraser_size,
                        })
                    }
                    Err(e2) => Err(format!("Failed to parse Rustiq file as any known format: {}", e2)),
                }
            }
        }
    }
}

// Optimized canvas state structure with layers
#[derive(Clone)]
struct CanvasState {
    width: usize,
    height: usize,
    layers: Vec<Layer>,
    active_layer_index: usize,
}

impl CanvasState {
    fn new(width: usize, height: usize) -> Self {
        let default_layer = Layer {
            name: "Background".to_string(),
            data: vec![None; width * height],
            visible: true,
        };
        
        Self {
            width,
            height,
            layers: vec![default_layer],
            active_layer_index: 0,
        }
    }
    
    #[inline]
    fn get(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height {
            // Iterate through layers from top to bottom
            for layer_index in (0..self.layers.len()).rev() {
                let layer = &self.layers[layer_index];
                if layer.visible {
                    let idx = y * self.width + x;
                    if let Some(color) = layer.data[idx] {
                        return Some(color);
                    }
                }
            }
        }
        None
    }
    
    #[inline]
    fn get_from_active_layer(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height && self.active_layer_index < self.layers.len() {
            let idx = y * self.width + x;
            self.layers[self.active_layer_index].data[idx]
        } else {
            None
        }
    }
    
    #[inline]
    fn set(&mut self, x: usize, y: usize, color: Option<Color32>) {
        if x < self.width && y < self.height && self.active_layer_index < self.layers.len() {
            let idx = y * self.width + x;
            self.layers[self.active_layer_index].data[idx] = color;
        }
    }
    
    #[inline]
    fn is_visible(&self, layer_index: usize) -> bool {
        layer_index < self.layers.len() && self.layers[layer_index].visible
    }
}

// Store changes for efficient undo/redo
#[derive(Clone)]
struct CanvasChange {
    x: usize,
    y: usize,
    layer_index: usize,
    old_color: Option<Color32>,
    new_color: Option<Color32>,
}

// Dialog for asking to save before quitting
enum SaveDialog {
    Hidden,
    AskingSave {
        return_to_menu: bool,
    },
}

// Main struct for the paint application
struct PaintApp {
    current_state: CanvasState,
    undo_stack: Vec<Vec<CanvasChange>>,
    redo_stack: Vec<Vec<CanvasChange>>,
    current_changes: Vec<CanvasChange>,
    current_tool: Tool,
    primary_color: Color32,
    secondary_color: Color32,
    saved_colors: Vec<Color32>,
    current_brush_style: BrushStyle, // Replaced brush_size
    eraser_size: i32,
    last_position: Option<(i32, i32)>,
    is_drawing: bool,
    last_action_time: Instant,
    texture: Option<TextureHandle>,
    texture_dirty: bool,
    zoom: f32,
    pan: Vec2,
    line_start: Option<(i32, i32)>,
    line_end: Option<(i32, i32)>,
    is_drawing_line: bool,
    is_first_click_line: bool,
    has_unsaved_changes: bool,
    last_save_path: Option<String>,
    save_dialog: SaveDialog,
    language: Language,
}

impl PaintApp {
    // Initialize a new PaintApp
    fn new(width: u32, height: u32, language: Language) -> Self {
        let initial_state = CanvasState::new(width as usize, height as usize);
        Self {
            current_state: initial_state,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_changes: Vec::new(),
            current_tool: Tool::AdvancedBrush, // Use the new variant
            primary_color: Color32::BLACK,
            secondary_color: Color32::WHITE,
            saved_colors: Vec::new(),
            current_brush_style: BrushStyle::default(), // Initialize new field
            eraser_size: 3,
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
            line_start: None,
            line_end: None,
            is_drawing_line: false,
            is_first_click_line: true,
            has_unsaved_changes: false,
            last_save_path: None,
            save_dialog: SaveDialog::Hidden,
            language,
        }
    }

    // Create a PaintApp from a .rustiq file
    fn from_rustiq_file(file: RustiqueFile, language: Language) -> Self { // file is already the new format due to migration
        let mut canvas = CanvasState {
            width: file.width,
            height: file.height,
            layers: Vec::with_capacity(file.layers.len()),
            active_layer_index: file.active_layer_index,
        };
        
        // Convert the saved layers to Canvas layers
        for layer_data in file.layers {
            let mut layer = Layer {
                name: layer_data.name,
                data: Vec::with_capacity(layer_data.data.len()),
                visible: layer_data.visible,
            };
            
            for pixel_opt in layer_data.data {
                if let Some(rgba) = pixel_opt {
                    layer.data.push(Some(Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])));
                } else {
                    layer.data.push(None);
                }
            }
            
            canvas.layers.push(layer);
        }
        
        let primary_color = Color32::from_rgba_unmultiplied(
            file.primary_color[0],
            file.primary_color[1],
            file.primary_color[2],
            file.primary_color[3]
        );
        
        let secondary_color = Color32::from_rgba_unmultiplied(
            file.secondary_color[0],
            file.secondary_color[1],
            file.secondary_color[2],
            file.secondary_color[3]
        );
        
        let mut saved_colors = Vec::with_capacity(file.saved_colors.len());
        for color_data in file.saved_colors {
            saved_colors.push(Color32::from_rgba_unmultiplied(
                color_data[0],
                color_data[1],
                color_data[2],
                color_data[3]
            ));
        }
        
        Self {
            current_state: canvas,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_changes: Vec::new(),
            current_tool: Tool::Brush,
            primary_color,
            secondary_color,
            saved_colors,
            current_brush_style: file.current_brush_style, // Use migrated/loaded style
            eraser_size: file.eraser_size,
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
            line_start: None,
            line_end: None,
            is_drawing_line: false,
            is_first_click_line: true,
            has_unsaved_changes: false,
            last_save_path: None,
            save_dialog: SaveDialog::Hidden,
            language,
        }
    }

    // New method to detect file format from path
    fn detect_format(path: &str) -> FileFormat {
        PathBuf::from(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(FileFormat::from_extension)
            .unwrap_or(FileFormat::Unknown)
    }
    
    // Unified save method
    fn save_file(&mut self, path: &str) -> Result<(), String> {
        let format = Self::detect_format(path);
        
        match format {
            FileFormat::Rustiq => self.save_as_rustiq(path),
            FileFormat::Unknown => {
                Err(format!("{}: {}", get_text("format_not_supported", self.language), path))
            },
            _ => {
                // Handle image formats
                if let Some(image_format) = format.get_image_format() {
                    self.save_as_image(path, image_format)
                } else {
                    Err(format!("{}: {}", get_text("format_not_supported", self.language), path))
                }
            }
        }
    }
    
    // Save as image with any supported format
    fn save_as_image(&mut self, path: &str, format: ImageFormat) -> Result<(), String> {
        let width = self.current_state.width;
        let height = self.current_state.height;
        let mut img = ImageBuffer::new(width as u32, height as u32);
        
        // Process rows one by one
        for y in 0..height {
            for x in 0..width {
                let color = self.current_state.get(x, y).unwrap_or(Color32::TRANSPARENT);
                img.put_pixel(x as u32, y as u32, Rgba([color.r(), color.g(), color.b(), color.a()]));
            }
        }

        match img.save_with_format(path, format) {
            Ok(_) => {
                self.has_unsaved_changes = false;
                self.last_save_path = Some(path.to_string());
                Ok(())
            },
            Err(e) => Err(format!("{}: {}", get_text("error_saving_image", self.language), e)),
        }
    }
    
    // Open any supported file
    fn open_file(path: &str, language: Language) -> Result<Self, String> {
        let format = Self::detect_format(path);
        
        match format {
            FileFormat::Rustiq => {
                // Open Rustiq file
                match fs::read_to_string(path) {
                    Ok(content) => {
                        // Use the new deserialize_with_migration function
                        match RustiqueFile::deserialize_with_migration(&content) {
                            Ok(rustiq_file_data) => {
                                let mut app = Self::from_rustiq_file(rustiq_file_data, language);
                                app.last_save_path = Some(path.to_string());
                                Ok(app)
                            },
                            Err(e) => Err(format!("{}: {}", get_text("error_reading_rustiq", language), e)),
                        }
                    },
                    Err(e) => Err(format!("{}: {}", get_text("error_reading_file", language), e))
                }
            },
            FileFormat::Unknown => {
                Err(format!("{}: {}", get_text("format_not_supported", language), path))
            },
            _ => {
                // Open image file
                match image::open(path) {
                    Ok(img) => {
                        let width = img.width() as usize;
                        let height = img.height() as usize;
                        let mut canvas = CanvasState::new(width, height);
                        
                        let rgba_img = img.to_rgba8();
                        for y in 0..height {
                            for x in 0..width {
                                let pixel = rgba_img.get_pixel(x as u32, y as u32);
                                if pixel[3] > 0 { // Not fully transparent
                                    let color = Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]);
                                    canvas.set(x, y, Some(color));
                                }
                            }
                        }
                        
                        let mut app = Self {
                            current_state: canvas,
                            undo_stack: Vec::new(),
                            redo_stack: Vec::new(),
                            current_changes: Vec::new(),
                            current_tool: Tool::AdvancedBrush, // Use the new variant
                            primary_color: Color32::BLACK,
                            secondary_color: Color32::WHITE,
                            saved_colors: Vec::new(),
                            current_brush_style: BrushStyle::default(), // Initialize new field
                            eraser_size: 3,
                            last_position: None,
                            is_drawing: false,
                            last_action_time: Instant::now(),
                            texture: None,
                            texture_dirty: true,
                            zoom: 1.0,
                            pan: Vec2::ZERO,
                            line_start: None,
                            line_end: None,
                            is_drawing_line: false,
                            is_first_click_line: true,
                            has_unsaved_changes: false,
                            last_save_path: Some(path.to_string()),
                            save_dialog: SaveDialog::Hidden,
                            language,
                        };
                        
                        Ok(app)
                    },
                    Err(e) => Err(format!("{}: {}", get_text("unable_to_open_image", language), e))
                }
            }
        }
    }
    
    // Create a PaintApp from a PNG file (deprecated but kept for backward compatibility)
    fn from_png_file(path: &str, language: Language) -> Option<Self> {
        match Self::open_file(path, language) {
            Ok(app) => Some(app),
            Err(_) => None
        }
    }

    // Save the current image as a .rustiq file
    fn save_as_rustiq(&mut self, path: &str) -> Result<(), String> {
        let mut layers = Vec::with_capacity(self.current_state.layers.len());
        
        for layer in &self.current_state.layers {
            let mut layer_data = Vec::with_capacity(layer.data.len());
            
            for &pixel_opt in &layer.data {
                match pixel_opt {
                    Some(color) => {
                        layer_data.push(Some([color.r(), color.g(), color.b(), color.a()]));
                    },
                    None => {
                        layer_data.push(None);
                    }
                }
            }
            
            layers.push(LayerData {
                name: layer.name.clone(),
                data: layer_data,
                visible: layer.visible,
            });
        }
        
        let mut saved_colors = Vec::with_capacity(self.saved_colors.len());
        for &color in &self.saved_colors {
            saved_colors.push([color.r(), color.g(), color.b(), color.a()]);
        }
        
        let rustiq_file = RustiqueFile {
            width: self.current_state.width,
            height: self.current_state.height,
            layers,
            active_layer_index: self.current_state.active_layer_index,
            primary_color: [self.primary_color.r(), self.primary_color.g(), self.primary_color.b(), self.primary_color.a()],
            secondary_color: [self.secondary_color.r(), self.secondary_color.g(), self.secondary_color.b(), self.secondary_color.a()],
            saved_colors,
            current_brush_style: self.current_brush_style.clone(), // Save new field
            eraser_size: self.eraser_size,
        };
        
        // Sérialiser avec gestion d'erreur
        let json = match serde_json::to_string(&rustiq_file) {
            Ok(json) => json,
            Err(e) => return Err(format!("Erreur de sérialisation: {}", e)),
        };
        
        // Écrire avec gestion d'erreur
        match fs::File::create(path) {
            Ok(mut file) => {
                match file.write_all(json.as_bytes()) {
                    Ok(_) => {
                        self.has_unsaved_changes = false;
                        self.last_save_path = Some(path.to_string());
                        Ok(())
                    },
                    Err(e) => Err(format!("Erreur d'écriture: {}", e)),
                }
            },
            Err(e) => Err(format!("Erreur de création du fichier: {}", e)),
        }
    }
    
    // Save as PNG (deprecated but kept for backward compatibility)
    fn save_as_png(&mut self, path: &str) -> Result<(), String> {
        // Make sure path has .png extension
        let path_with_ext = if !path.to_lowercase().ends_with(".png") {
            format!("{}.png", path)
        } else {
            path.to_string()
        };

        self.save_as_image(&path_with_ext, ImageFormat::Png)
    }
    
    // Save the current canvas using the last save path
    fn quick_save(&mut self) -> Result<(), String> {
        if let Some(path) = &self.last_save_path {
            let path_clone = path.clone(); // Clone the path to end the immutable borrow
            self.save_file(&path_clone)    // Use the cloned path
        } else {
            Err(get_text("no_previous_path", self.language))
        }
    }

    // Layer management functions
    fn add_layer(&mut self, name: String) {
        self.current_state.layers.push(Layer {
            name,
            data: vec![None; self.current_state.width * self.current_state.height],
            visible: true,
        });
        self.current_state.active_layer_index = self.current_state.layers.len() - 1;
        self.texture_dirty = true;
        self.has_unsaved_changes = true;
    }
    
    fn remove_layer(&mut self, index: usize) {
        if self.current_state.layers.len() > 1 && index < self.current_state.layers.len() {
            self.current_state.layers.remove(index);
            if self.current_state.active_layer_index >= self.current_state.layers.len() {
                self.current_state.active_layer_index = self.current_state.layers.len() - 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn move_layer_up(&mut self, index: usize) {
        if index > 0 && index < self.current_state.layers.len() {
            self.current_state.layers.swap(index, index - 1);
            if self.current_state.active_layer_index == index {
                self.current_state.active_layer_index -= 1;
            } else if self.current_state.active_layer_index == index - 1 {
                self.current_state.active_layer_index += 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn move_layer_down(&mut self, index: usize) {
        if index < self.current_state.layers.len() - 1 {
            self.current_state.layers.swap(index, index + 1);
            if self.current_state.active_layer_index == index {
                self.current_state.active_layer_index += 1;
            } else if self.current_state.active_layer_index == index + 1 {
                self.current_state.active_layer_index -= 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn toggle_layer_visibility(&mut self, index: usize) {
        if index < self.current_state.layers.len() {
            self.current_state.layers[index].visible = !self.current_state.layers[index].visible;
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn set_active_layer(&mut self, index: usize) {
        if index < self.current_state.layers.len() {
            self.current_state.active_layer_index = index;
        }
    }
    
    fn rename_layer(&mut self, index: usize, name: String) {
        if index < self.current_state.layers.len() {
            self.current_state.layers[index].name = name;
            self.has_unsaved_changes = true;
        }
    }
    
    // Color management functions
    fn add_saved_color(&mut self, color: Color32) {
        // Avoid duplicates
        if !self.saved_colors.contains(&color) {
            if self.saved_colors.len() >= MAX_SAVED_COLORS {
                self.saved_colors.remove(0); // Remove oldest color
            }
            self.saved_colors.push(color);
        }
    }
    
    fn remove_saved_color(&mut self, index: usize) {
        if index < self.saved_colors.len() {
            self.saved_colors.remove(index);
        }
    }
    
    fn set_primary_color_from_saved(&mut self, index: usize) {
        if index < self.saved_colors.len() {
            self.primary_color = self.saved_colors[index];
        }
    }
    
    fn set_secondary_color_from_saved(&mut self, index: usize) {
        if index < self.saved_colors.len() {
            self.secondary_color = self.saved_colors[index];
        }
    }

    // Record a pixel change for undo/redo
    fn record_change(&mut self, x: usize, y: usize, new_color: Option<Color32>) {
        if x < self.current_state.width && y < self.current_state.height {
            let old_color = self.current_state.get_from_active_layer(x, y);
            if old_color != new_color {
                self.current_changes.push(CanvasChange {
                    x, 
                    y, 
                    layer_index: self.current_state.active_layer_index,
                    old_color, 
                    new_color
                });
                self.current_state.set(x, y, new_color);
                self.has_unsaved_changes = true;
            }
        }
    }

    // Save the current state for undo functionality
    fn save_state(&mut self) {
        if !self.current_changes.is_empty() {
            self.undo_stack.push(std::mem::take(&mut self.current_changes));
            self.current_changes = Vec::new();
            if self.undo_stack.len() > MAX_UNDO_STEPS {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.is_drawing = false;
            self.has_unsaved_changes = true;
        }
    }

    // Undo the last action
    fn undo(&mut self) {
        if let Some(changes) = self.undo_stack.pop() {
            let mut redo_changes = Vec::with_capacity(changes.len());
            
            // Apply changes in reverse
            for change in changes.iter().rev() {
                let layer_index_backup = self.current_state.active_layer_index;
                self.current_state.active_layer_index = change.layer_index;
                
                // Store the original change for redo
                redo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    layer_index: change.layer_index,
                    old_color: change.old_color,  // Keep the original old color
                    new_color: change.new_color,  // Keep the original new color
                });
                
                self.current_state.set(change.x, change.y, change.old_color);
                self.current_state.active_layer_index = layer_index_backup;
            }
            
            self.redo_stack.push(redo_changes);
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }

    // Redo the last undone action
    fn redo(&mut self) {
        if let Some(changes) = self.redo_stack.pop() {
            let mut undo_changes = Vec::with_capacity(changes.len());
            
            // Apply changes in reverse
            for change in changes.iter().rev() {
                let layer_index_backup = self.current_state.active_layer_index;
                self.current_state.active_layer_index = change.layer_index;
                
                let current_color = self.current_state.get_from_active_layer(change.x, change.y);
                undo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    layer_index: change.layer_index,
                    old_color: current_color,
                    new_color: change.new_color,
                });
                
                self.current_state.set(change.x, change.y, change.new_color);
                self.current_state.active_layer_index = layer_index_backup;
            }
            
            self.undo_stack.push(undo_changes);
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }

    // Draw a line between two points
    fn draw_line(&mut self, start: (i32, i32), end: (i32, i32), color: Color32) {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        // For large brush sizes, collect points
        let _size = if self.current_tool == Tool::Eraser { self.eraser_size } else { self.current_brush_style.size as i32 };
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
        
        // Draw points with the specified color
        let fill_color = if self.current_tool == Tool::Eraser { None } else { Some(color) };
        for &(px, py) in &points {
            self.draw_point_with_color(px, py, fill_color);
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    // Draw a single point with optimized circular brush
    fn draw_point(&mut self, x: i32, y: i32, use_secondary: bool) {
        let color = if use_secondary { self.secondary_color } else { self.primary_color };
        let fill_color = if self.current_tool == Tool::Eraser { None } else { Some(color) };
        self.draw_point_with_color(x, y, fill_color);
    }
    
    // Helper function for drawing a point with a specific color
    fn draw_point_with_color(&mut self, x: i32, y: i32, fill_color: Option<Color32>) {
        let canvas_width_i32 = self.current_state.width as i32;
        let canvas_height_i32 = self.current_state.height as i32;

        // Ensure active layer is visible before drawing
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }

        if self.current_tool == Tool::Eraser {
            let size = self.eraser_size; // Eraser uses its own size
            let size_squared = size * size;
            let mut pixels_to_erase = Vec::new();

            for dy_loop in -size..=size {
                for dx_loop in -size..=size {
                    if dx_loop * dx_loop + dy_loop * dy_loop <= size_squared { // Circular eraser
                        let px_x = x + dx_loop;
                        let px_y = y + dy_loop;
                        if px_x >= 0 && px_x < canvas_width_i32 && px_y >= 0 && px_y < canvas_height_i32 {
                            pixels_to_erase.push((px_x as usize, px_y as usize));
                        }
                    }
                }
            }
            for (px_x, px_y) in pixels_to_erase {
                self.record_change(px_x, px_y, None); // Eraser sets color to None
            }
            self.texture_dirty = true;

        } else if self.current_tool == Tool::AdvancedBrush {
            match self.current_brush_style.brush_type {
                BrushType::Round => {
                    let brush_diameter = self.current_brush_style.size;
                    let brush_radius = brush_diameter / 2.0;
                    let size_i32 = brush_radius.ceil() as i32; // Iterate up to the radius

                    let mut pixels_to_draw = Vec::new();

                    for dy_loop in -size_i32..=size_i32 {
                        for dx_loop in -size_i32..=size_i32 {
                            let dist_sq = (dx_loop * dx_loop + dy_loop * dy_loop) as f32;
                            if dist_sq <= brush_radius * brush_radius {
                                let px_x = x + dx_loop;
                                let px_y = y + dy_loop;
                                if px_x >= 0 && px_x < canvas_width_i32 && px_y >= 0 && px_y < canvas_height_i32 {
                                    pixels_to_draw.push((px_x as usize, px_y as usize, (dist_sq.sqrt())));
                                }
                            }
                        }
                    }
                    
                    for (px_x, px_y, distance) in pixels_to_draw {
                        let mut actual_fill_color = fill_color;
                        if let Some(base_color) = actual_fill_color {
                            let hardness = self.current_brush_style.hardness;
                            let mut alpha_multiplier = 1.0;
                            let hard_radius = brush_radius * hardness;

                            if distance > hard_radius {
                                if brush_radius > hard_radius { // Avoid division by zero if hardness is 1.0
                                    alpha_multiplier = (brush_radius - distance) / (brush_radius - hard_radius);
                                    alpha_multiplier = alpha_multiplier.clamp(0.0, 1.0);
                                } else {
                                    alpha_multiplier = 0.0; // Outside hard radius when hardness is 1.0
                                }
                            }
                            let new_alpha = (base_color.a() as f32 * alpha_multiplier).round() as u8;
                            actual_fill_color = Some(base_color.with_a(new_alpha));
                        }
                        self.record_change(px_x, px_y, actual_fill_color);
                    }
                    self.texture_dirty = true;
                }
                BrushType::Bright => {
                    let brush_width = self.current_brush_style.size;
                    // Bright brush: fixed thickness relative to width (e.g., 20%), hardness does not affect thickness.
                    let brush_thickness = (brush_width * 0.20).max(1.0);

                    let brush_angle_rad = self.current_brush_style.angle.to_radians();
                    let cos_a = brush_angle_rad.cos();
                    let sin_a = brush_angle_rad.sin();

                    let bounding_box_dim = brush_width.hypot(brush_thickness);
                    let size_half = (bounding_box_dim / 2.0).ceil() as i32;

                    // canvas_width_i32 and canvas_height_i32 are already defined at the start of draw_point_with_color

                    for dy_loop in -size_half..=size_half {
                        for dx_loop in -size_half..=size_half {
                            let px_x_i32 = x + dx_loop;
                            let px_y_i32 = y + dy_loop;

                            if px_x_i32 >= 0 && px_x_i32 < canvas_width_i32 && px_y_i32 >= 0 && px_y_i32 < canvas_height_i32 {
                                let rel_dx = px_x_i32 as f32 - x as f32;
                                let rel_dy = px_y_i32 as f32 - y as f32;

                                let local_x = rel_dx * cos_a + rel_dy * sin_a;
                                let local_y = -rel_dx * sin_a + rel_dy * cos_a;

                                if local_x.abs() <= brush_width / 2.0 && local_y.abs() <= brush_thickness / 2.0 {
                                    self.record_change(px_x_i32 as usize, px_y_i32 as usize, fill_color);
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                BrushType::Fan => {
                    let num_bristles = self.current_brush_style.bristle_count.unwrap_or(10).max(2) as usize;
                    let bristle_length = self.current_brush_style.size.max(1.0);
                    let bristle_thickness = (self.current_brush_style.hardness * bristle_length * 0.05).max(1.0).min(bristle_length * 0.2);
            
                    let fan_spread_angle_deg = 90.0; 
                    let overall_rotation_rad = self.current_brush_style.angle.to_radians();
            
                    for i in 0..num_bristles {
                        let normalized_i = if num_bristles > 1 { i as f32 / (num_bristles - 1) as f32 } else { 0.5 }; 
                        let current_bristle_angle_offset_rad = (normalized_i - 0.5) * fan_spread_angle_deg.to_radians();
                        let bristle_abs_angle_rad = current_bristle_angle_offset_rad + overall_rotation_rad;
            
                        // Calculate center of this bristle (thin rectangle)
                        // Bristle extends from (x,y) outwards. Midpoint is (x + L/2*cos, y + L/2*sin)
                        let bristle_mid_x = x as f32 + (bristle_length / 2.0) * bristle_abs_angle_rad.cos();
                        let bristle_mid_y = y as f32 + (bristle_length / 2.0) * bristle_abs_angle_rad.sin();
            
                        let cos_b_rot = bristle_abs_angle_rad.cos();
                        let sin_b_rot = bristle_abs_angle_rad.sin();
            
                        let bristle_bounding_box_dim = bristle_length.hypot(bristle_thickness);
                        let bristle_size_half = (bristle_bounding_box_dim / 2.0).ceil() as i32;
            
                        for dy_loop in -bristle_size_half..=bristle_size_half {
                            for dx_loop in -bristle_size_half..=bristle_size_half {
                                let px_on_canvas_x = bristle_mid_x.round() as i32 + dx_loop;
                                let px_on_canvas_y = bristle_mid_y.round() as i32 + dy_loop;
            
                                if px_on_canvas_x >= 0 && px_on_canvas_x < canvas_width_i32 && px_on_canvas_y >= 0 && px_on_canvas_y < canvas_height_i32 {
                                    let rel_dx_to_bristle_mid = px_on_canvas_x as f32 - bristle_mid_x;
                                    let rel_dy_to_bristle_mid = px_on_canvas_y as f32 - bristle_mid_y;
            
                                    let local_x = rel_dx_to_bristle_mid * cos_b_rot + rel_dy_to_bristle_mid * sin_b_rot;
                                    let local_y = -rel_dx_to_bristle_mid * sin_b_rot + rel_dy_to_bristle_mid * cos_b_rot;
            
                                    if local_x.abs() <= bristle_length / 2.0 && local_y.abs() <= bristle_thickness / 2.0 {
                                        self.record_change(px_on_canvas_x as usize, px_on_canvas_y as usize, fill_color);
                                    }
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                BrushType::Rigger => {
                    let rigger_diameter = 2.0; 
                    let rigger_radius = rigger_diameter / 2.0;
                    // For a rigger, we typically want a hard edge, so no complex alpha falloff.
                    // The iteration below will create a small circular footprint.
                    
                    // Bounding box for a small circle.
                    // If rigger_radius is 1.0, size_half_i32 will be 1. Iteration: -1, 0, 1.
                    let size_half_i32 = rigger_radius.ceil() as i32;

                    for dy_loop in -size_half_i32..=size_half_i32 {
                        for dx_loop in -size_half_i32..=size_half_i32 {
                            let px_x_i32 = x + dx_loop;
                            let px_y_i32 = y + dy_loop;

                            if px_x_i32 >= 0 && px_x_i32 < canvas_width_i32 && px_y_i32 >= 0 && px_y_i32 < canvas_height_i32 {
                                // Check if within the small circle (distance from center of the loop area)
                                let dist_sq = (dx_loop * dx_loop + dy_loop * dy_loop) as f32;
                                if dist_sq <= rigger_radius * rigger_radius { // Using rigger_radius directly here, not squared
                                    self.record_change(px_x_i32 as usize, px_y_i32 as usize, fill_color);
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                BrushType::Filbert => {
                    let brush_major_axis = self.current_brush_style.size;
                    // Hardness controls roundness: 1.0 is rounder (closer to circle), 0.1 is flatter oval
                    let brush_minor_axis_factor = self.current_brush_style.hardness.max(0.1).min(1.0); 
                    let brush_minor_axis = brush_major_axis * brush_minor_axis_factor;

                    let oval_rotation_rad = self.current_brush_style.angle.to_radians();
                    let cos_rot = oval_rotation_rad.cos();
                    let sin_rot = oval_rotation_rad.sin();

                    let semi_major = brush_major_axis / 2.0;
                    let semi_minor = brush_minor_axis / 2.0;

                    // Prevent division by zero or extreme cases & handle tiny brushes
                    if semi_major < 0.5 || semi_minor < 0.5 { 
                        if x >=0 && x < canvas_width_i32 && y >= 0 && y < canvas_height_i32 {
                             self.record_change(x as usize, y as usize, fill_color);
                        }
                        self.texture_dirty = true;
                        return; 
                    }
                    
                    let square_semi_major = semi_major * semi_major;
                    let square_semi_minor = semi_minor * semi_minor;

                    // Bounding box for iteration based on the major axis, as it's the largest extent before rotation.
                    // Rotation is handled by transforming pixel coordinates, not by changing bounding box shape here.
                    let bounding_box_radius = (semi_major.max(semi_minor)).ceil() as i32;


                    for dy_loop in -bounding_box_radius..=bounding_box_radius {
                        for dx_loop in -bounding_box_radius..=bounding_box_radius {
                            let px_x_i32 = x + dx_loop;
                            let px_y_i32 = y + dy_loop;

                            if px_x_i32 >= 0 && px_x_i32 < canvas_width_i32 && px_y_i32 >= 0 && px_y_i32 < canvas_height_i32 {
                                let rel_dx = px_x_i32 as f32 - x as f32;
                                let rel_dy = px_y_i32 as f32 - y as f32;

                                // Rotate the pixel's relative coordinates to align with the ellipse's local axes
                                let local_x = rel_dx * cos_rot + rel_dy * sin_rot;
                                let local_y = -rel_dx * sin_rot + rel_dy * cos_rot;

                                // Ellipse equation: (local_x^2 / semi_major^2) + (local_y^2 / semi_minor^2) <= 1
                                if (local_x * local_x) / square_semi_major + (local_y * local_y) / square_semi_minor <= 1.0 {
                                    self.record_change(px_x_i32 as usize, px_y_i32 as usize, fill_color);
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                BrushType::Angle => {
                    let brush_width = self.current_brush_style.size;
                    let mut brush_thickness = self.current_brush_style.hardness * brush_width * 0.25; 
                    if brush_thickness < 1.0 { brush_thickness = 1.0; }

                    let overall_rotation_rad = self.current_brush_style.angle.to_radians();
                    let cos_rot = overall_rotation_rad.cos();
                    let sin_rot = overall_rotation_rad.sin();

                    let intrinsic_angle_rad = (45.0f32).to_radians(); // 45-degree intrinsic angle
                    let tan_intrinsic = intrinsic_angle_rad.tan();

                    // Bounding box needs to cover the brush at any rotation and shear
                    // A simple bounding box for the un-sheared rotated rectangle:
                    let base_bounding_box_dim = brush_width.hypot(brush_thickness);
                    // Account for additional extent due to shear: max_shear_offset = (brush_width / 2.0) * tan_intrinsic
                    // This component adds to the 'height' of the bounding box in one direction.
                    // A more generous bounding box can be simpler:
                    let size_half = (base_bounding_box_dim / 2.0 + (brush_width / 2.0 * tan_intrinsic).abs()).ceil() as i32;


                    for dy_loop in -size_half..=size_half {
                        for dx_loop in -size_half..=size_half {
                            let px_x_i32 = x + dx_loop;
                            let px_y_i32 = y + dy_loop;

                            if px_x_i32 >= 0 && px_x_i32 < canvas_width_i32 && px_y_i32 >= 0 && px_y_i32 < canvas_height_i32 {
                                let rel_dx = px_x_i32 as f32 - x as f32;
                                let rel_dy = px_y_i32 as f32 - y as f32;

                                // Rotate to align with user-defined overall brush angle
                                let rotated_dx = rel_dx * cos_rot + rel_dy * sin_rot;
                                let rotated_dy = -rel_dx * sin_rot + rel_dy * cos_rot;

                                // Now check against the intrinsic shape (parallelogram/sheared rectangle)
                                // Condition 1: Check if within the main width (along the brush's rotated x-axis)
                                if rotated_dx.abs() <= brush_width / 2.0 {
                                    // Condition 2: Check if within the sheared thickness
                                    let y_offset_due_to_shear = rotated_dx * tan_intrinsic;
                                    if (rotated_dy - y_offset_due_to_shear).abs() <= brush_thickness / 2.0 {
                                        self.record_change(px_x_i32 as usize, px_y_i32 as usize, fill_color);
                                    }
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                BrushType::Flat => {
                    let brush_width = self.current_brush_style.size;
                    let mut brush_thickness = self.current_brush_style.hardness * brush_width * 0.25;
                    if brush_thickness < 1.0 { brush_thickness = 1.0; }

                    let brush_angle_rad = self.current_brush_style.angle.to_radians();
                    let cos_a = brush_angle_rad.cos();
                    let sin_a = brush_angle_rad.sin();

                    // Bounding box for iteration - should be large enough for rotated brush
                    let size_half = (brush_width.hypot(brush_thickness) / 2.0).ceil() as i32;

                    for dy_loop in -size_half..=size_half {
                        for dx_loop in -size_half..=size_half {
                            let px_x_i32 = x + dx_loop;
                            let px_y_i32 = y + dy_loop;

                            if px_x_i32 >= 0 && px_x_i32 < canvas_width_i32 && px_y_i32 >= 0 && px_y_i32 < canvas_height_i32 {
                                let rel_dx = px_x_i32 as f32 - x as f32;
                                let rel_dy = px_y_i32 as f32 - y as f32;

                                let local_x = rel_dx * cos_a + rel_dy * sin_a;
                                let local_y = -rel_dx * sin_a + rel_dy * cos_a;

                                if local_x.abs() <= brush_width / 2.0 && local_y.abs() <= brush_thickness / 2.0 {
                                    self.record_change(px_x_i32 as usize, px_y_i32 as usize, fill_color);
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
                _ => {
                    // Fallback for other brush types (e.g., could default to Round or do nothing)
                    // For now, let's make it behave like a simple Round brush of fixed size without hardness
                    let size = (self.current_brush_style.size / 2.0).ceil() as i32;
                    let size_squared = size * size;
                    for dy_loop in -size..=size {
                        for dx_loop in -size..=size {
                            if dx_loop * dx_loop + dy_loop * dy_loop <= size_squared {
                                let px_x = x + dx_loop;
                                let px_y = y + dy_loop;
                                if px_x >= 0 && px_x < canvas_width_i32 && px_y >= 0 && px_y < canvas_height_i32 {
                                    self.record_change(px_x as usize, px_y as usize, fill_color);
                                }
                            }
                        }
                    }
                    self.texture_dirty = true;
                }
            }
        }
        // Other tools (PaintBucket, ColorPicker, Line) don't use draw_point_with_color directly for their main effect.
        // Line tool calls draw_line, which calls draw_point_with_color, so it will use the AdvancedBrush logic.
    }

    // Optimized paint bucket fill
    fn paint_bucket(&mut self, x: usize, y: usize, use_secondary: bool) {
        if x >= self.current_state.width || y >= self.current_state.height {
            return;
        }
        
        // Ensure active layer is visible before filling
        if self.current_state.active_layer_index < self.current_state.layers.len() && 
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        let target_color = self.current_state.get_from_active_layer(x, y);
        let color = if use_secondary { self.secondary_color } else { self.primary_color };
        let fill_color = if self.current_tool == Tool::Eraser {
            None
        } else {
            Some(color)
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
            if visited[idx] || self.current_state.get_from_active_layer(cx, cy) != target_color {
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
    fn pick_color(&mut self, x: usize, y: usize, use_secondary: bool) {
        if let Some(color) = self.current_state.get(x, y) {
            if use_secondary {
                self.secondary_color = color;
            } else {
                self.primary_color = color;
            }
        }
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
    
    // Show save dialog
    fn show_save_dialog(&mut self, return_to_menu: bool) {
        self.save_dialog = SaveDialog::AskingSave { 
            return_to_menu
        };
    }
    
    // Set language
    fn set_language(&mut self, language: Language) {
        self.language = language;
    }
}

// Action à effectuer après fermeture de dialogues
enum PendingAction {
    None,
    ReturnToMenu,
    HandleLayerAction(LayerAction),
    UndoAction,
    RedoAction,
}

// Action à effectuer sur les calques
enum LayerAction {
    ToggleVisibility(usize),
    SetActive(usize),
    Edit(usize),
}

// Main application struct
struct MyApp {
    state: AppState,
    error_message: Option<String>,
    show_error: bool,
    new_layer_name: String,
    rename_layer_index: Option<usize>,
    rename_layer_name: String,
    pending_action: PendingAction,
    language: Language,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu(MainMenu::new(Language::French)),
            error_message: None,
            show_error: false,
            new_layer_name: "New Layer".to_string(),
            rename_layer_index: None,
            rename_layer_name: String::new(),
            pending_action: PendingAction::None,
            language: Language::French,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process keyboard shortcuts
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);
        
        // Show error message dialog if needed
        if self.show_error {
            egui::Window::new(get_text("error", self.language))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(self.error_message.as_deref().unwrap_or(&get_text("an_error_occurred", self.language)));
                    if ui.button("OK").clicked() {
                        self.show_error = false;
                    }
                });
        }
        
        // Process rename layer dialog
        if let Some(layer_idx) = self.rename_layer_index {
            egui::Window::new(get_text("rename_layer", self.language))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.text_edit_singleline(&mut self.rename_layer_name);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() && !self.rename_layer_name.is_empty() {
                            if let AppState::Canvas(paint_app) = &mut self.state {
                                paint_app.rename_layer(layer_idx, self.rename_layer_name.clone());
                            }
                            self.rename_layer_index = None;
                        }
                        if ui.button(get_text("cancel", self.language)).clicked() {
                            self.rename_layer_index = None;
                        }
                    });
                });
        }
        
        // Process pending actions
        match &self.pending_action {
            PendingAction::ReturnToMenu => {
                self.state = AppState::MainMenu(MainMenu::new(self.language));
                self.pending_action = PendingAction::None;
            },
            PendingAction::HandleLayerAction(action) => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    match action {
                        LayerAction::ToggleVisibility(idx) => {
                            paint_app.toggle_layer_visibility(*idx);
                        },
                        LayerAction::SetActive(idx) => {
                            paint_app.set_active_layer(*idx);
                        },
                        LayerAction::Edit(idx) => {
                            if let Some(layer) = paint_app.current_state.layers.get(*idx) {
                                self.rename_layer_index = Some(*idx);
                                self.rename_layer_name = layer.name.clone();
                            }
                        }
                    }
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::UndoAction => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    paint_app.undo();
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::RedoAction => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    paint_app.redo();
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::None => {}
        }
        
        match &mut self.state {
            AppState::MainMenu(menu) => {
                if let Some(result) = menu.show(ctx) {
                    match result {
                        main_menu::MenuResult::Action(action) => {
                            match action {
                                main_menu::MenuAction::NewCanvas(width, height) => {
                                    self.state = AppState::Canvas(PaintApp::new(width, height, self.language));
                                },
                                main_menu::MenuAction::OpenFile => {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                        .add_filter("PNG Image", &["png"])
                                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                                        .add_filter("BMP Image", &["bmp"])
                                        .add_filter("TIFF Image", &["tiff", "tif"])
                                        .add_filter("GIF Image", &["gif"])
                                        .add_filter("WebP Image", &["webp"])
                                        .add_filter("Rustique File", &["rustiq"])
                                        .set_directory("/")
                                        .pick_file() {
                                        match PaintApp::open_file(path.to_str().unwrap(), self.language) {
                                            Ok(app) => self.state = AppState::Canvas(app),
                                            Err(e) => {
                                                self.error_message = Some(e);
                                                self.show_error = true;
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        main_menu::MenuResult::LanguageChanged(language) => {
                            self.language = language;
                            if let AppState::Canvas(paint_app) = &mut self.state {
                                paint_app.set_language(language);
                            }
                            self.state = AppState::MainMenu(MainMenu::new(language));
                        }
                    }
                }
            }
            AppState::Canvas(paint_app) => {
                // Handle keyboard shortcuts
                if ctrl {
                    if ctx.input(|i| i.key_pressed(egui::Key::Z)) && !shift {
                        self.pending_action = PendingAction::UndoAction;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Y)) || 
                        shift && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                        self.pending_action = PendingAction::RedoAction;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                        if let Some(_) = &paint_app.last_save_path {
                            match paint_app.quick_save() {
                                Ok(_) => {},
                                Err(e) => {
                                    self.error_message = Some(e);
                                    self.show_error = true;
                                }
                            }
                        } else {
                            // Show save dialog
                            if let Some(path) = FileDialog::new()
                                .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                .add_filter("PNG Image", &["png"])
                                .add_filter("JPEG Image", &["jpg", "jpeg"])
                                .add_filter("BMP Image", &["bmp"])
                                .add_filter("TIFF Image", &["tiff", "tif"])
                                .add_filter("GIF Image", &["gif"])
                                .add_filter("WebP Image", &["webp"])
                                .add_filter("Rustique File", &["rustiq"])
                                .set_directory("/")
                                .save_file() {
                                match paint_app.save_file(path.to_str().unwrap()) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        self.error_message = Some(e);
                                        self.show_error = true;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Handle save dialog
                match &mut paint_app.save_dialog {
                    SaveDialog::Hidden => {},
                    SaveDialog::AskingSave { return_to_menu } => {
                        let return_to_menu_val = *return_to_menu;
                        egui::Window::new(get_text("save_changes", self.language))
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.label(get_text("want_to_save_changes", self.language));
                                ui.horizontal(|ui| {
                                    if ui.button(get_text("yes", self.language)).clicked() {
                                        // Open save dialog
                                        let result = if let Some(path) = FileDialog::new()
                                            .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                            .add_filter("PNG Image", &["png"])
                                            .add_filter("JPEG Image", &["jpg", "jpeg"])
                                            .add_filter("BMP Image", &["bmp"])
                                            .add_filter("TIFF Image", &["tiff", "tif"])
                                            .add_filter("GIF Image", &["gif"])
                                            .add_filter("WebP Image", &["webp"])
                                            .add_filter("Rustique File", &["rustiq"])
                                            .set_directory("/")
                                            .save_file() {
                                            paint_app.save_file(path.to_str().unwrap())
                                        } else {
                                            // User canceled the save dialog
                                            Ok(())
                                        };
                                        
                                        match result {
                                            Ok(_) => {
                                                paint_app.save_dialog = SaveDialog::Hidden;
                                                if return_to_menu_val {
                                                    self.pending_action = PendingAction::ReturnToMenu;
                                                }
                                            },
                                            Err(e) => {
                                                self.error_message = Some(e);
                                                self.show_error = true;
                                            }
                                        }
                                    }
                                    if ui.button(get_text("no", self.language)).clicked() {
                                        paint_app.save_dialog = SaveDialog::Hidden;
                                        if return_to_menu_val {
                                            self.pending_action = PendingAction::ReturnToMenu;
                                        }
                                    }
                                    if ui.button(get_text("cancel", self.language)).clicked() {
                                        paint_app.save_dialog = SaveDialog::Hidden;
                                    }
                                });
                            });
                    }
                }
                
                paint_app.update_texture(ctx);

                egui::SidePanel::left("layers_panel").show(ctx, |ui| {
                    ui.set_min_width(180.0);
                    ui.vertical(|ui| {
                        ui.heading(get_text("layers", self.language));
                        ui.separator();
                        
                        // Layer controls
                        ui.horizontal(|ui| {
                            if ui.button("+").clicked() {
                                paint_app.add_layer(format!("{} {}", get_text("layer", self.language), paint_app.current_state.layers.len() + 1));
                            }
                            if ui.button("-").clicked() && paint_app.current_state.layers.len() > 1 {
                                paint_app.remove_layer(paint_app.current_state.active_layer_index);
                            }
                            ui.add_space(5.0);
                            if ui.button(get_text("up", self.language)).clicked() {
                                paint_app.move_layer_up(paint_app.current_state.active_layer_index);
                            }
                            if ui.button(get_text("down", self.language)).clicked() {
                                paint_app.move_layer_down(paint_app.current_state.active_layer_index);
                            }
                        });
                        
                        ui.separator();
                        
                        // Layer list - clone the data to avoid borrowing issues
                        // This avoids the conflict between immutable and mutable borrows
                        let layers_info: Vec<(usize, String, bool, bool)> = paint_app.current_state.layers
                            .iter()
                            .enumerate()
                            .map(|(i, layer)| (i, layer.name.clone(), layer.visible, i == paint_app.current_state.active_layer_index))
                            .collect();
                        
                        // Display the layers in reverse order (top layer first visually)
                        for (i, name, visible, is_active) in layers_info.iter().rev() {
                            ui.horizontal(|ui| {
                                let visible_text = if *visible { "👁" } else { "⊘" };
                                if ui.button(visible_text).clicked() {
                                    self.pending_action = PendingAction::HandleLayerAction(
                                        LayerAction::ToggleVisibility(*i)
                                    );
                                }
                                
                                if ui.selectable_label(*is_active, name).clicked() {
                                    self.pending_action = PendingAction::HandleLayerAction(
                                        LayerAction::SetActive(*i)
                                    );
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✏").clicked() {
                                        self.pending_action = PendingAction::HandleLayerAction(
                                            LayerAction::Edit(*i)
                                        );
                                    }
                                });
                            });
                        }
                    });
                });

                egui::SidePanel::right("tools_panel").show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(get_text("tools", self.language));
                        // Removed the old "Brush" button here
                        if ui.button(get_text("eraser", self.language)).clicked() {
                            paint_app.current_tool = Tool::Eraser;
                        }
                        if ui.button(get_text("paint_bucket", self.language)).clicked() {
                            paint_app.current_tool = Tool::PaintBucket;
                        }
                        if ui.button(get_text("color_picker", self.language)).clicked() {
                            paint_app.current_tool = Tool::ColorPicker;
                        }
                        if ui.button(get_text("line", self.language)).clicked() {
                            paint_app.current_tool = Tool::Line;
                        }
                        
                        ui.separator();

                        // New "Brush Types" section
                        ui.heading(get_text("brush_types", self.language)); // Assuming "brush_types" key exists or using placeholder
                        for brush_variant in BrushType::all_variants() {
                            if ui.button(format!("{:?}", brush_variant)).clicked() {
                                paint_app.current_tool = Tool::AdvancedBrush;
                                let old_style = paint_app.current_brush_style.clone();
                                paint_app.current_brush_style.brush_type = brush_variant;
                                paint_app.current_brush_style.size = old_style.size;
                                paint_app.current_brush_style.hardness = old_style.hardness;

                                match brush_variant {
                                    BrushType::Round | BrushType::Mop | BrushType::Rigger | 
                                    BrushType::Filbert | BrushType::Bright | BrushType::Fan => {
                                        paint_app.current_brush_style.angle = 0.0;
                                    }
                                    BrushType::Flat | BrushType::Angle => {
                                        // Preserve angle for these types, or set to old_style.angle
                                        // If old_style.angle was specific to a non-angle brush,
                                        // it might be better to reset to 0.0 too.
                                        // For now, preserving as per refined instruction for Flat/Angle.
                                        paint_app.current_brush_style.angle = old_style.angle;
                                    }
                                }
                                paint_app.current_brush_style.bristle_count = None;
                                paint_app.current_brush_style.taper_strength = None;
                            }
                        }
                        ui.separator();

                        ui.label(get_text("save_options", self.language));
                        if ui.button(get_text("save_file", self.language)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                .add_filter("PNG Image", &["png"])
                                .add_filter("JPEG Image", &["jpg", "jpeg"])
                                .add_filter("BMP Image", &["bmp"])
                                .add_filter("TIFF Image", &["tiff", "tif"])
                                .add_filter("GIF Image", &["gif"])
                                .add_filter("WebP Image", &["webp"])
                                .add_filter("Rustique File", &["rustiq"])
                                .set_directory("/")
                                .save_file() {
                                match paint_app.save_file(path.to_str().unwrap()) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        self.error_message = Some(e);
                                        self.show_error = true;
                                    }
                                }
                            }
                        }
                        
                        ui.separator();
                        
                        ui.add_space(10.0);
                        ui.label(get_text("brush_settings", self.language));
                        // Display current brush type:
                        ui.label(format!("{}: {:?}", get_text("brush_type", self.language), paint_app.current_brush_style.brush_type));
                        
                        ui.horizontal(|ui| {
                            ui.label(get_text("brush_size", self.language)); // Ensure this key exists
                            ui.add(egui::DragValue::new(&mut paint_app.current_brush_style.size).speed(0.1).clamp_range(1.0..=500.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(get_text("brush_angle", self.language)); // Ensure this key exists
                            ui.add(egui::DragValue::new(&mut paint_app.current_brush_style.angle).speed(1.0).clamp_range(-180.0..=180.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(get_text("brush_hardness", self.language)); // Ensure this key exists
                            ui.add(egui::Slider::new(&mut paint_app.current_brush_style.hardness, 0.0..=1.0));
                        });
                        
                        ui.add_space(10.0);
                        ui.label(get_text("eraser_size", self.language)); // Ensure this key exists
                        ui.add(egui::DragValue::new(&mut paint_app.eraser_size).speed(0.1).clamp_range(1..=500));
                        
                        ui.add_space(10.0);
                        ui.label(get_text("colors", self.language));
                        ui.horizontal(|ui| {
                            ui.label(get_text("primary", self.language));
                            ui.color_edit_button_srgba(&mut paint_app.primary_color);
                            if ui.button("+").clicked() {
                                paint_app.add_saved_color(paint_app.primary_color);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(get_text("secondary", self.language));
                            ui.color_edit_button_srgba(&mut paint_app.secondary_color);
                            if ui.button("+").clicked() {
                                paint_app.add_saved_color(paint_app.secondary_color);
                            }
                        });
                        
                        ui.add_space(10.0);
                        ui.label(get_text("zoom", self.language));
                        ui.add(egui::Slider::new(&mut paint_app.zoom, 0.1..=10.0).logarithmic(true));
                        
                        // Saved colors palette
                        if !paint_app.saved_colors.is_empty() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.label(get_text("saved_colors", self.language));
                            
                            let available_width = ui.available_width();
                            let color_size = 24.0;
                            let colors_per_row = (available_width / color_size).floor() as usize;
                            let colors_count = paint_app.saved_colors.len();
                            
                            for i in 0..(colors_count / colors_per_row + (if colors_count % colors_per_row > 0 { 1 } else { 0 })) {
                                ui.horizontal(|ui| {
                                    for j in 0..colors_per_row {
                                        let idx = i * colors_per_row + j;
                                        if idx < colors_count {
                                            let color = paint_app.saved_colors[idx];
                                            let btn = ui.add(egui::Button::new("").fill(color).min_size(Vec2::new(color_size, color_size)));
                                            
                                            if btn.clicked() {
                                                paint_app.primary_color = color;
                                            }
                                            if btn.clicked_by(egui::PointerButton::Secondary) {
                                                paint_app.secondary_color = color;
                                            }
                                            if btn.clicked_by(egui::PointerButton::Middle) {
                                                paint_app.remove_saved_color(idx);
                                                break;
                                            }
                                            
                                            btn.on_hover_text(format!("{}\n{}\n{}",
                                                get_text("left_click_primary", self.language),
                                                get_text("right_click_secondary", self.language),
                                                get_text("middle_click_delete", self.language)
                                            ));
                                        }
                                    }
                                });
                            }
                        }
                    });
                });

                // Top panel for buttons
                let (undo_clicked, redo_clicked, return_to_menu_clicked) = egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    let mut return_clicked = false;
                    let mut undo_clicked = false;
                    let mut redo_clicked = false;
                    
                    ui.horizontal(|ui| {
                        // Return to menu button
                        if ui.button(get_text("return_to_menu", self.language)).clicked() {
                            return_clicked = true;
                        }
                        
                        if ui.button(get_text("undo", self.language)).clicked() {
                            undo_clicked = true;
                        }
                        if ui.button(get_text("redo", self.language)).clicked() {
                            redo_clicked = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(get_text("shortcuts_info", self.language));
                        });
                    });
                    (undo_clicked, redo_clicked, return_clicked)
                }).inner;
                
                // Handle button actions outside of the panel to avoid borrow issues
                if undo_clicked {
                    self.pending_action = PendingAction::UndoAction;
                }
                
                if redo_clicked {
                    self.pending_action = PendingAction::RedoAction;
                }
                
                // Handle the return to menu request after all panels to avoid borrowing issues
                if return_to_menu_clicked {
                    if paint_app.has_unsaved_changes {
                        paint_app.show_save_dialog(true);
                    } else {
                        self.pending_action = PendingAction::ReturnToMenu;
                    }
                }

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

                    // Improved panning with middle button
                    if response.dragged_by(egui::PointerButton::Middle) {
                        paint_app.pan += response.drag_delta();
                    }

                    // Handle line tool
                    if paint_app.current_tool == Tool::Line {
                        // First click sets the start point, second click sets the endpoint and draws the line
                        if response.clicked() && !response.clicked_by(egui::PointerButton::Middle) {
                            let is_secondary = response.clicked_by(egui::PointerButton::Secondary);
                            if let Some(pos) = response.interact_pointer_pos() {
                                let canvas_pos = to_canvas.transform_pos(pos);
                                let x = canvas_pos.x as i32;
                                let y = canvas_pos.y as i32;
                                
                                if paint_app.is_first_click_line {
                                    // First click: set start point
                                    paint_app.line_start = Some((x, y));
                                    paint_app.line_end = Some((x, y));
                                    paint_app.is_drawing_line = true;
                                    paint_app.is_first_click_line = false;
                                } else {
                                    // Second click: draw the line
                                    if let (Some(start), Some(_)) = (paint_app.line_start, paint_app.line_end) {
                                        let color = if is_secondary { paint_app.secondary_color } else { paint_app.primary_color };
                                        paint_app.draw_line(start, (x, y), color);
                                        paint_app.is_drawing_line = false;
                                        paint_app.line_start = None;
                                        paint_app.line_end = None;
                                        paint_app.is_first_click_line = true;
                                        paint_app.save_state();
                                    }
                                }
                            }
                        }
                        
                        // Update preview line when moving the mouse
                        if paint_app.is_drawing_line && !paint_app.is_first_click_line {
                            if let Some(pos) = response.hover_pos() {
                                let canvas_pos = to_canvas.transform_pos(pos);
                                paint_app.line_end = Some((canvas_pos.x as i32, canvas_pos.y as i32));
                                
                                // Draw the preview line
                                if let (Some(start), Some(end)) = (paint_app.line_start, paint_app.line_end) {
                                    let start_pos = Pos2::new(
                                        start.0 as f32 * canvas_rect.width() / canvas_width + canvas_rect.min.x,
                                        start.1 as f32 * canvas_rect.height() / canvas_height + canvas_rect.min.y
                                    );
                                    let end_pos = Pos2::new(
                                        end.0 as f32 * canvas_rect.width() / canvas_width + canvas_rect.min.x,
                                        end.1 as f32 * canvas_rect.height() / canvas_height + canvas_rect.min.y
                                    );
                                    
                                    // Use the right mouse button state to determine color
                                    let is_secondary = response.ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));
                                    let color = if is_secondary { paint_app.secondary_color } else { paint_app.primary_color };
                                    let size = match paint_app.current_tool {
                                        Tool::AdvancedBrush => paint_app.current_brush_style.size,
                                        // Eraser does not draw a preview line with its own size in this way,
                                        // but if it did, it would be paint_app.eraser_size.
                                        // Line tool uses brush settings for its appearance.
                                        _ => paint_app.current_brush_style.size,
                                    };
                                    
                                    painter.line_segment([start_pos, end_pos], Stroke::new(size * paint_app.zoom, color));
                                }
                            }
                        }
                        
                        // Cancel the line by pressing the middle mouse button
                        if response.clicked_by(egui::PointerButton::Middle) {
                            paint_app.is_drawing_line = false;
                            paint_app.line_start = None;
                            paint_app.line_end = None;
                            paint_app.is_first_click_line = true;
                        }
                        
                        // Also allow Escape key to cancel
                        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            paint_app.is_drawing_line = false;
                            paint_app.line_start = None;
                            paint_app.line_end = None;
                            paint_app.is_first_click_line = true;
                        }
                    } else {
                        // Handle other tools with left/right mouse buttons
                        if (response.clicked_by(egui::PointerButton::Primary) || 
                            response.clicked_by(egui::PointerButton::Secondary)) && 
                           !response.clicked_by(egui::PointerButton::Middle) {
                            paint_app.is_drawing = true;
                            paint_app.save_state();
                        }

                        if (response.dragged() || response.clicked()) && 
                           !(response.dragged_by(egui::PointerButton::Middle) || 
                             response.clicked_by(egui::PointerButton::Middle)) {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let canvas_pos = to_canvas.transform_pos(pos);
                                let x = canvas_pos.x as usize;
                                let y = canvas_pos.y as usize;
                                let is_secondary = response.dragged_by(egui::PointerButton::Secondary) || 
                                                 response.clicked_by(egui::PointerButton::Secondary);
                                
                                if x < paint_app.current_state.width && y < paint_app.current_state.height {
                                    match paint_app.current_tool {
                                        Tool::PaintBucket => paint_app.paint_bucket(x, y, is_secondary),
                                        Tool::ColorPicker => paint_app.pick_color(x, y, is_secondary),
                                        Tool::AdvancedBrush | Tool::Eraser => {
                                            let (x_i32, y_i32) = (canvas_pos.x as i32, canvas_pos.y as i32);
                                            let color_to_use = if paint_app.current_tool == Tool::Eraser {
                                                // Eraser logic is handled in draw_point_with_color (None color)
                                                // For draw_line, it needs a color; the actual erasing happens in draw_point_with_color
                                                paint_app.primary_color // Dummy, won't be used directly if erasing
                                            } else {
                                                if is_secondary { paint_app.secondary_color } else { paint_app.primary_color }
                                            };

                                            if let Some(last_pos) = paint_app.last_position {
                                                paint_app.draw_line(last_pos, (x_i32, y_i32), color_to_use);
                                            } else {
                                                paint_app.draw_point(x_i32, y_i32, is_secondary);
                                            }
                                            paint_app.last_position = Some((x_i32, y_i32));
                                        }
                                        _ => { /* Tool::Line is handled by its own logic above */ }
                                    }
                                    paint_app.is_drawing = true;
                                }
                            }
                        } else {
                            paint_app.save_state();
                            paint_app.last_position = None;
                        }
                    }

                    // Improved zooming with mouse wheel
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