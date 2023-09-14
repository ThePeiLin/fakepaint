use eframe::egui;

pub struct ExportImageWindow {
    open: bool,
    pub scale: u32,
    pub file_name: String,
}

impl ExportImageWindow {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, canvas: &crate::Canvas, tile: &crate::TileSet) {
        use rust_i18n::t;
        let mut created = false;
        egui::Window::new(t!("export_image"))
            .resizable(false)
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("export-image")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t!("file_name"));
                        ui.text_edit_singleline(&mut self.file_name);
                        if ui.button(t!("browse")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("png", &["png"])
                                .save_file()
                            {
                                if let Some(string) = path.to_str() {
                                    self.file_name = string.to_string();
                                }
                            }
                        }
                        ui.end_row();
                        ui.label(t!("scale"));
                        ui.add(
                            egui::DragValue::new(&mut self.scale)
                                .clamp_range(core::ops::RangeInclusive::new(1, 16)),
                        );
                        ui.end_row();
                        if ui.button(t!("export")).clicked() && self.file_name.len() > 0 {
                            let mut path = std::path::PathBuf::new();
                            path.push(&self.file_name);
                            path.set_extension("png");
                            if let Some(string) = path.to_str() {
                                canvas.export_as_image(tile, string, self.scale);
                                created = true;
                            }
                        }
                    });
            });
        if created {
            self.open = false;
        }
    }
}

impl Default for ExportImageWindow {
    fn default() -> Self {
        Self {
            open: false,
            file_name: "".to_string(),
            scale: 1,
        }
    }
}
