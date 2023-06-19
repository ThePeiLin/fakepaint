use eframe::egui;

#[derive(Copy, Clone)]
pub struct TileState {
    pub idx: usize,
    pub fc: egui::Color32,
    pub bc: egui::Color32,
}

pub struct Canvas {
    pub cells: Vec<Option<TileState>>,
    pub size_x: usize,
    pub size_y: usize,
}

impl Canvas {
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Option<TileState> {
        &mut self.cells[y * self.size_x + x]
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Option<TileState> {
        &self.cells[y * self.size_x + x]
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.size_x * self.size_y
    }
}
