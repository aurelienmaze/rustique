use std::collections::HashMap;

pub fn get_text(key: &str) -> String {
    let translations: HashMap<&str, &str> = [
        // Main menu
        ("canvas_dimensions", "Canvas Dimensions"),
        ("width", "Width:"),
        ("height", "Height:"),
        ("create_new_canvas", "Create New Canvas"),
        ("open_file", "Open PNG File"),
        
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
        
        // Options
        ("brush_size", "Brush Size:"),
        ("eraser_size", "Eraser Size:"),
        ("color", "Color:"),
        ("zoom", "Zoom:"),
        
        // Action buttons
        ("return_to_menu", "Return to Menu"),
        ("undo", "Undo"),
        ("redo", "Redo"),
        ("save_png", "Save PNG"),
        
        // Info messages
        ("shortcuts_info", "Ctrl+Z: Undo | Ctrl+Y: Redo | Ctrl+S: Save"),
        
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
        ("unable_to_open_png", "Unable to open PNG image"),
        ("error_saving_png", "Error saving PNG"),
    ].iter().cloned().collect();
    
    translations.get(key).unwrap_or(&key).to_string()
}