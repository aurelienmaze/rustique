use egui::Color32;

#[derive(Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Option<Color32>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![None; width * height],
        }
    }
    
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.data[idx]
        } else {
            None
        }
    }
    
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, color: Option<Color32>) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.data[idx] = color;
        }
    }
}