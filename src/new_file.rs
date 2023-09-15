use eframe::egui;

pub struct NewFileWinodw {
    open: bool,
    pub width: usize,
    pub height: usize,
    pub file_name: String,
}

impl NewFileWinodw {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        canvas: &mut crate::Canvas,
        editing_file_name: &mut Option<String>,
    ) -> bool {
        use rust_i18n::t;
        let mut created = false;
        egui::Window::new(t!("new_file"))
            .resizable(false)
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("new-file")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t!("file_name"));
                        ui.text_edit_singleline(&mut self.file_name);
                        if ui.button(t!("browse")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("json", &["json"])
                                .save_file()
                            {
                                if let Some(string) = path.to_str() {
                                    self.file_name = string.to_string();
                                }
                            }
                        }
                        ui.end_row();
                        ui.label(t!("width"));
                        ui.add(
                            egui::DragValue::new(&mut self.width)
                                .clamp_range(core::ops::RangeInclusive::new(1, std::isize::MAX)),
                        );
                        ui.end_row();
                        ui.label(t!("height"));
                        ui.add(
                            egui::DragValue::new(&mut self.height)
                                .clamp_range(core::ops::RangeInclusive::new(1, std::isize::MAX)),
                        );
                        ui.end_row();
                        if ui.button(t!("create")).clicked() && self.file_name.len() > 0 {
                            let mut path = std::path::PathBuf::new();
                            path.push(&self.file_name);
                            path.set_extension("json");
                            if let Some(string) = path.to_str() {
                                *canvas = crate::Canvas::with_size(self.width, self.height);
                                *editing_file_name = Some(string.to_string());
                                created = true;
                            }
                        }
                    });
            });
        if created {
            self.open = false;
        }
        created
    }
}

impl Default for NewFileWinodw {
    fn default() -> Self {
        Self {
            open: false,
            width: 16,
            height: 16,
            file_name: "".to_string(),
        }
    }
}
