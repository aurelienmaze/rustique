use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    French,
    English,
}

pub fn get_text(key: &str, language: Language) -> String {
    match language {
        Language::French => get_french_text(key),
        Language::English => get_english_text(key),
    }
}

fn get_french_text(key: &str) -> String {
    let translations: HashMap<&str, &str> = [
        // Menu principal
        ("language", "Langue"),
        ("canvas_dimensions", "Dimensions du canevas"),
        ("width", "Largeur:"),
        ("height", "Hauteur:"),
        ("create_new_canvas", "Créer un nouveau canevas"),
        ("open_png", "Ouvrir un fichier PNG"),
        ("open_rustiq", "Ouvrir un fichier Rustiq"),
        ("open_file", "Ouvrir un fichier"),
        
        // Onglets et panneaux
        ("layers", "Calques"),
        ("tools", "Outils"),
        ("save_options", "Options de sauvegarde"),
        
        // Boutons de calques
        ("layer", "Calque"),
        ("up", "Haut"),
        ("down", "Bas"),
        
        // Outils
        ("brush", "Pinceau"),
        ("eraser", "Gomme"),
        ("paint_bucket", "Pot de peinture"),
        ("color_picker", "Pipette"),
        ("line", "Ligne"),
        
        // Options
        ("brush_size", "Taille du pinceau:"),
        ("eraser_size", "Taille de la gomme:"),
        ("colors", "Couleurs:"),
        ("primary", "Primaire:"),
        ("secondary", "Secondaire:"),
        ("zoom", "Zoom:"),
        ("saved_colors", "Couleurs sauvegardées:"),
        
        // Interactions
        ("left_click_primary", "Clic gauche: définir comme couleur primaire"),
        ("right_click_secondary", "Clic droit: définir comme couleur secondaire"),
        ("middle_click_delete", "Clic molette: supprimer"),
        
        // Boutons d'action
        ("return_to_menu", "Retour au menu"),
        ("undo", "Annuler"),
        ("redo", "Refaire"),
        ("save_png", "Sauvegarder PNG"),
        ("save_rustiq", "Sauvegarder Rustiq"),
        ("save_file", "Sauvegarder Fichier"),
        
        // Messages d'info
        ("shortcuts_info", "Clic droit: couleur secondaire | Ctrl+Z: Annuler | Ctrl+Y: Refaire | Ctrl+S: Sauvegarder"),
        
        // Dialogues
        ("error", "Erreur"),
        ("an_error_occurred", "Une erreur s'est produite"),
        ("save_changes", "Sauvegarder les modifications?"),
        ("want_to_save_changes", "Voulez-vous sauvegarder les modifications?"),
        ("yes", "Oui"),
        ("no", "Non"),
        ("cancel", "Annuler"),
        ("rename_layer", "Renommer le calque"),
        
        // Erreurs
        ("format_not_supported", "Format de fichier non pris en charge"),
        ("no_previous_path", "Aucun chemin de sauvegarde précédent"),
        ("unable_to_open_png", "Impossible d'ouvrir l'image PNG"),
        ("unable_to_open_image", "Impossible d'ouvrir l'image"),
        ("error_reading_rustiq", "Erreur lors de la lecture du fichier Rustiq"),
        ("error_reading_file", "Erreur lors de la lecture du fichier"),
        ("error_saving_png", "Erreur lors de la sauvegarde PNG"),
        ("error_saving_image", "Erreur lors de la sauvegarde de l'image"),
    ].iter().cloned().collect();
    
    translations.get(key).unwrap_or(&key).to_string()
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