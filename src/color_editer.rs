use eframe::egui;

pub struct ColorPicker<'c>(&'c mut egui::Color32);

impl<'c> ColorPicker<'c> {
    pub fn new(color: &'c mut egui::Color32) -> Self {
        ColorPicker(color)
    }
}
