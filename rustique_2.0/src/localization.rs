use std::collections::HashMap;

// Legacy enum kept for compatibility but only English is now used
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    English,
}

pub fn get_text(key: &str, _language: Language) -> String {
    get_english_text(key)
}

fn get_english_text(key: &str) -> String {
    let translations: HashMap<&str, &str> = [
        // Main menu
        ("language", "Language"),
        ("canvas_dimensions", "Canvas Dimensions"),
        ("width", "Width:"),
        ("height", "Height:"),
        ("create_new_canvas", "Create New Canvas"),
        ("open_png", "Open PNG File"),
        ("open_rustiq", "Open Rustiq File"),
        ("open_file", "Open File"),
        ("website", "Website"),
        
        // Tabs and panels
        ("layers", "Layers"),
        ("tools", "Tools"),
        ("save_options", "Save Options"),
        
        // Layer buttons
        ("layer", "Layer"),
        ("up", "Up"),
        ("down", "Down"),
        
        // Tools
        ("brush", "Brush"),
        ("eraser", "Eraser"),
        ("paint_bucket", "Paint Bucket"),
        ("color_picker", "Color Picker"),
        ("line", "Line"),
        
        // Options
        ("brush_size", "Brush Size:"),
        ("eraser_size", "Eraser Size:"),
        ("colors", "Colors:"),
        ("primary", "Primary:"),
        ("secondary", "Secondary:"),
        ("zoom", "Zoom:"),
        ("saved_colors", "Saved Colors:"),
        
        // Interactions
        ("left_click_primary", "Left-click: set as primary color"),
        ("right_click_secondary", "Right-click: set as secondary color"),
        ("middle_click_delete", "Middle-click: delete"),
        
        // Action buttons
        ("return_to_menu", "Return to Menu"),
        ("undo", "Undo"),
        ("redo", "Redo"),
        ("save_png", "Save PNG"),
        ("save_rustiq", "Save Rustiq"),
        ("save_file", "Save File"),
        
        // Info messages
        ("shortcuts_info", "Right-click: secondary color | Ctrl+Z: Undo | Ctrl+Y: Redo | Ctrl+S: Save"),
        
        // Dialogs
        ("error", "Error"),
        ("an_error_occurred", "An error occurred"),
        ("save_changes", "Save Changes?"),
        ("want_to_save_changes", "Do you want to save changes?"),
        ("yes", "Yes"),
        ("no", "No"),
        ("cancel", "Cancel"),
        ("rename_layer", "Rename Layer"),
        
        // Errors
        ("format_not_supported", "File format not supported"),
        ("no_previous_path", "No previous save path"),
        ("unable_to_open_png", "Unable to open PNG image"),
        ("unable_to_open_image", "Unable to open image"),
        ("error_reading_rustiq", "Error reading Rustiq file"),
        ("error_reading_file", "Error reading file"),
        ("error_saving_png", "Error saving PNG"),
        ("error_saving_image", "Error saving image"),
    ].iter().cloned().collect();
    
    translations.get(key).unwrap_or(&key).to_string()
}