#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod image_button;
mod setup;
mod tile;

use eframe::egui;
use tile::TileSet;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    let opts = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "fakepaint",
        opts,
        Box::new(|cc| Box::new(FakePaint::new(cc))),
    )
}

struct TileState {
    idx: usize,
    fc: egui::Color32,
    bc: egui::Color32,
}

struct PencilState {
    idx: usize,
    fc: egui::Color32,
    bc: egui::Color32,
}

struct FakePaint {
    tile: TileSet,
    pencil_state: PencilState,
    canvas_cells: Vec<Option<TileState>>,
    canvas_size_x: usize,
    canvas_size_y: usize,
}

impl FakePaint {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup::custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            tile: TileSet::new(
                tile::load_texture(&cc.egui_ctx).unwrap(),
                16,
                16,
                egui::vec2(16.0, 16.0),
            ),
            pencil_state: PencilState {
                idx: 8,
                fc: egui::Color32::WHITE,
                bc: egui::Color32::BLACK,
            },
            canvas_cells: Vec::with_capacity(256),
            canvas_size_x: 16,
            canvas_size_y: 16,
        }
    }

    pub fn access_cell(&self, x: usize, y: usize) -> &Option<TileState> {
        &self.canvas_cells[y * self.canvas_size_x + x]
    }

    fn get_center_rect(rect: &egui::Rect, size: egui::Vec2) -> egui::Rect {
        let rect_size = rect.size();
        let offset_x = (rect_size.x - size.x) / 2.0;
        let offset_y = (rect_size.y - size.y) / 2.0;
        let egui::Pos2 {
            x: rect_min_x,
            y: rect_min_y,
        } = rect.left_top();
        egui::Rect::from_min_size(
            egui::pos2(rect_min_x + offset_x, rect_min_y + offset_y),
            size,
        )
    }

    fn draw_pencil_state(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("画笔：").size(24.0));
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(24.0), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter()
                    .rect_filled(rect, egui::Rounding::none(), self.pencil_state.bc);
                self.tile
                    .to_image(self.pencil_state.idx, egui::Vec2::splat(16.0))
                    .tint(self.pencil_state.fc)
                    .paint_at(ui, Self::get_center_rect(&rect, egui::Vec2::splat(16.0)));
            }
        });
    }

    fn draw_pencil_colors(&self, ui: &mut egui::Ui) {
        egui::Grid::new("pencil-colors")
            .min_col_width(16.0)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("前景色：");
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), self.pencil_state.fc);
                }
                ui.end_row();
                ui.label("背景色：");
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), self.pencil_state.bc);
                }
                ui.end_row();
            });
    }

    fn char_selector(&mut self, ui: &mut egui::Ui) {
        use image_button::ImageButton;
        let idx = self.pencil_state.idx;
        let x = idx % self.tile.columns;
        let y = idx / self.tile.columns;
        ui.heading(format!("字符--({},{})", x, y));
        egui::Grid::new("char-selectors")
            .spacing(egui::Vec2::ZERO)
            .striped(true)
            .num_columns(16)
            .min_col_width(16.0)
            .min_row_height(16.0)
            .show(ui, |ui| {
                let mut idx = 0;
                for _ in 0..self.tile.rows {
                    for _ in 0..self.tile.columns {
                        if ui
                            .add(
                                ImageButton::new(self.tile.tex.id(), egui::Vec2::splat(16.0))
                                    .selected(self.pencil_state.idx == idx)
                                    .frame(false)
                                    .uv(self.tile.uv(idx))
                                    .tint(egui::Color32::DARK_GRAY)
                                    .selected_tint(egui::Color32::WHITE)
                                    .selected_bg_fill(egui::Color32::TRANSPARENT)
                                    .rounding(false),
                            )
                            .on_hover_text(egui::RichText::new(idx.to_string()).strong().heading())
                            .clicked()
                        {
                            self.pencil_state.idx = idx;
                        }
                        idx += 1;
                    }
                    ui.end_row();
                }
            });
    }
}

impl eframe::App for FakePaint {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                self.draw_pencil_state(ui);
                ui.separator();
                self.char_selector(ui);
                ui.separator();
                self.draw_pencil_colors(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
