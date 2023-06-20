#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod canvas;
mod file;
mod image_button;
mod setup;
mod tile;

use eframe::egui;
use file::write_canvas_to_file;
use tile::TileSet;

use crate::file::load_canvas_from_file;

const TILE_SIZE: f32 = 16.0;

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

#[derive(Copy, Clone)]
struct PencilState {
    idx: usize,
    fc: egui::Color32,
    bc: egui::Color32,
}

impl PencilState {
    pub fn swap_color_and_into(self) -> canvas::TileState {
        canvas::TileState {
            idx: self.idx,
            fc: self.bc,
            bc: self.fc,
        }
    }
}

impl Into<canvas::TileState> for PencilState {
    fn into(self) -> canvas::TileState {
        canvas::TileState {
            idx: self.idx,
            fc: self.fc,
            bc: self.bc,
        }
    }
}

struct FakePaint {
    tile: TileSet,
    pencil_state: PencilState,
    canvas: canvas::Canvas,
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

fn get_grid_x_y(rect: egui::Rect, pos: egui::Pos2, size: egui::Vec2) -> (usize, usize) {
    let min = rect.min;
    let x = (pos.x - min.x) / size.x;
    let y = (pos.y - min.y) / size.y;
    (x.floor() as usize, y.floor() as usize)
}

fn compute_grid_rect(rect: egui::Rect, grid_size: egui::Vec2, x: usize, y: usize) -> egui::Rect {
    let egui::Pos2 {
        x: left_top_x,
        y: left_top_y,
    } = rect.left_top();
    egui::Rect::from_min_size(
        egui::pos2(
            left_top_x + grid_size.x * x as f32,
            left_top_y + grid_size.y * y as f32,
        ),
        grid_size,
    )
}

impl FakePaint {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup::custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let canvas: canvas::Canvas;
        if let Ok(cc) = load_canvas_from_file(std::path::Path::new("output.json")) {
            canvas = cc;
        } else {
            let size_x: usize = 16;
            let size_y: usize = 16;
            let size = size_x * size_x;
            let mut cells = Vec::with_capacity(size);
            cells.resize(size, None);
            canvas = canvas::Canvas {
                cells,
                size_x,
                size_y,
            }
        }
        Self {
            tile: TileSet::new(
                tile::load_texture(&cc.egui_ctx).unwrap(),
                16,
                16,
                egui::vec2(TILE_SIZE, TILE_SIZE),
            ),
            pencil_state: PencilState {
                idx: 0,
                fc: egui::Color32::WHITE,
                bc: egui::Color32::BLACK,
            },
            canvas,
        }
    }

    fn draw_pencil_state(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("画笔：").size(24.0));
            let (rect, _) = ui.allocate_exact_size(
                egui::Vec2::splat(TILE_SIZE + 8.0),
                egui::Sense::focusable_noninteractive(),
            );
            if ui.is_rect_visible(rect) {
                ui.painter()
                    .rect_filled(rect, egui::Rounding::none(), self.pencil_state.bc);
                self.tile.paint_in_rect(
                    ui,
                    get_center_rect(&rect, egui::Vec2::splat(TILE_SIZE)),
                    self.pencil_state.idx,
                    self.pencil_state.fc,
                    None,
                );
            }
        });
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui) {
        let (rect, res) = ui.allocate_exact_size(
            egui::Vec2::splat(TILE_SIZE * self.canvas.size_x as f32),
            egui::Sense::drag(),
        );

        let hover_pos = res.hover_pos();
        if ui.is_rect_visible(rect) {
            for y in 0..self.canvas.size_y {
                for x in 0..self.canvas.size_x {
                    let rect = compute_grid_rect(rect, egui::Vec2::splat(TILE_SIZE), x, y);
                    let cell = self.canvas.get_cell(x, y);

                    if hover_pos != None && rect.contains(hover_pos.unwrap()) {
                        self.tile.paint_in_rect(
                            ui,
                            rect,
                            self.pencil_state.idx,
                            self.pencil_state.fc,
                            Some(self.pencil_state.bc),
                        );
                    } else if let Some(c) = cell {
                        let idx = c.idx;
                        let bc = c.bc;
                        let fc = c.fc;
                        self.tile.paint_in_rect(ui, rect, idx, fc, Some(bc));
                    } else if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::none(),
                            if (y + x) % 2 == 0 {
                                egui::Color32::GRAY
                            } else {
                                egui::Color32::DARK_GRAY
                            },
                        );
                    }
                }
            }
        }

        if res.hovered() && res.dragged() && hover_pos != None && rect.contains(hover_pos.unwrap())
        {
            let (x, y) = get_grid_x_y(rect, hover_pos.unwrap(), egui::Vec2::splat(TILE_SIZE));
            let ctx = ui.ctx();
            if ctx.input(|i| i.pointer.primary_down()) {
                *(self.canvas.get_cell_mut(x, y)) = Some(self.pencil_state.into());
            } else if ctx.input(|i| i.pointer.secondary_down()) {
                *(self.canvas.get_cell_mut(x, y)) = Some(self.pencil_state.swap_color_and_into());
            }
        }
    }

    fn draw_pencil_colors(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("pencil-colors")
            .min_col_width(TILE_SIZE)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("前景色：");
                let (rect, res) =
                    ui.allocate_exact_size(egui::Vec2::splat(TILE_SIZE), egui::Sense::click());
                if ui.is_rect_visible(rect) {
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), self.pencil_state.fc);
                }
                if res.clicked() {
                    std::mem::swap(&mut self.pencil_state.fc, &mut self.pencil_state.bc);
                }

                ui.end_row();
                ui.label("背景色：");
                let (rect, res) =
                    ui.allocate_exact_size(egui::Vec2::splat(TILE_SIZE), egui::Sense::click());
                if ui.is_rect_visible(rect) {
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), self.pencil_state.bc);
                }
                if res.clicked() {
                    std::mem::swap(&mut self.pencil_state.fc, &mut self.pencil_state.bc);
                }
                ui.end_row();
            });
    }

    fn char_preview(&self, idx: usize, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                self.tile
                    .to_image(idx, egui::Vec2::splat(TILE_SIZE * 1.5))
                    .bg_fill(self.pencil_state.bc)
                    .tint(self.pencil_state.fc),
            );
            ui.label(
                egui::RichText::new(idx.to_string())
                    .size(TILE_SIZE * 1.5)
                    .strong()
                    .heading(),
            );
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
            .min_col_width(TILE_SIZE)
            .min_row_height(TILE_SIZE)
            .show(ui, |ui| {
                let mut idx = 0;
                for _ in 0..self.tile.rows {
                    for _ in 0..self.tile.columns {
                        let res = ui
                            .add(
                                ImageButton::new(
                                    Some(self.tile.tex.id()),
                                    egui::Vec2::splat(TILE_SIZE),
                                )
                                .selected(self.pencil_state.idx == idx)
                                .frame(false)
                                .uv(self.tile.uv(idx))
                                .tint(egui::Color32::DARK_GRAY)
                                .bg_fill(egui::Color32::TRANSPARENT)
                                .selected_tint(egui::Color32::WHITE)
                                .selected_bg_fill(egui::Color32::TRANSPARENT)
                                .sense(egui::Sense::click_and_drag())
                                .rounding(false),
                            )
                            .on_hover_ui(|ui| {
                                self.char_preview(idx, ui);
                            });

                        if res.clicked() || res.dragged() {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_canvas(ui);
        });
    }

    fn on_close_event(&mut self) -> bool {
        let res = write_canvas_to_file(&self.canvas, std::path::Path::new("output.json"));
        if let Err(_) = res {
            false
        } else {
            true
        }
    }
}
