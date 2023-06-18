#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    let vec1 = vec!["foo".to_string()];
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

struct PancilState {
    idx: usize,
    fc: egui::Color32,
    bc: egui::Color32,
}

struct FakePaint {
    tile: TileSet,
    pancil_state: PancilState,
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
            pancil_state: PancilState {
                idx: 8,
                fc: egui::Color32::DEBUG_COLOR,
                bc: egui::Color32::GRAY,
            },
        }
    }
}

impl eframe::App for FakePaint {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(format!("Fonts-{}x{}", self.tile.rows, self.tile.columns));
                ui.image(self.tile.tex.id(), egui::vec2(256.0, 256.0));
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                self.tile
                    .to_image(self.pancil_state.idx, egui::vec2(32.0, 32.0))
                    .tint(self.pancil_state.fc)
                    .bg_fill(self.pancil_state.bc),
            );
        });
    }
}
