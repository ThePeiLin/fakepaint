#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod canvas;
mod color_editer;
mod file;
mod image_button;
mod setup;
mod tile;

use canvas::Canvas;
use color_editer::PencilState;
use eframe::egui;
use file::load_canvas_from_file;
use file::write_canvas_to_file;
use tile::TileSet;

const PALETTE_X: usize = 8;
const PALETTE_Y: usize = 4;
const TILE_SIZE: f32 = 16.0;
const TILE_SIZE_VEC2: egui::Vec2 = egui::Vec2::splat(16.0);

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    let opts = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(550.0, 540.0)),
        ..Default::default()
    };
    eframe::run_native(
        "fakepaint",
        opts,
        Box::new(|cc| Box::new(FakePaint::new(cc))),
    )
}

struct FakePaint {
    tile: TileSet,
    pencil_state: PencilState,
    canvas: Canvas,
    cur_cell: Option<canvas::TileState>,
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
        let canvas: Canvas;
        if let Ok(cc) = load_canvas_from_file(std::path::Path::new("output.json")) {
            canvas = cc;
        } else {
            let size_x: usize = 16;
            let size_y: usize = 16;
            let size = size_x * size_x;
            let mut cells = Vec::with_capacity(size);
            cells.resize(size, None);
            canvas = Canvas {
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
                TILE_SIZE_VEC2,
            ),
            pencil_state: PencilState::default(),
            canvas,
            cur_cell: None,
        }
    }

    fn draw_pencil_state(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
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
                    get_center_rect(&rect, TILE_SIZE_VEC2),
                    self.pencil_state.idx,
                    self.pencil_state.fc,
                    None,
                );
            }
        });
    }

    fn get_gray(x: usize, y: usize) -> egui::Color32 {
        if (y + x) % 2 == 0 {
            egui::Color32::GRAY
        } else {
            egui::Color32::DARK_GRAY
        }
    }

    fn draw_nib(&mut self, ui: &mut egui::Ui, rect: egui::Rect, x: usize, y: usize) {
        let cell = self.canvas.get_cell(x, y);
        self.cur_cell = *cell;
        let pencil = &self.pencil_state;
        if let Some((fc, bc)) = pencil.get_fc_bc(cell) {
            self.tile.paint_in_rect(ui, rect, pencil.idx, fc, Some(bc))
        } else {
            ui.painter()
                .rect_filled(rect, egui::Rounding::none(), Self::get_gray(x, y))
        }
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui) {
        self.cur_cell = None;
        let (rect, res) = ui.allocate_exact_size(
            egui::Vec2::splat(TILE_SIZE * self.canvas.size_x as f32),
            egui::Sense::drag(),
        );

        let hover_pos = res.hover_pos();
        if ui.is_rect_visible(rect) {
            for y in 0..self.canvas.size_y {
                for x in 0..self.canvas.size_x {
                    let rect = compute_grid_rect(rect, TILE_SIZE_VEC2, x, y);
                    let cell = self.canvas.get_cell(x, y);

                    if hover_pos != None && rect.contains(hover_pos.unwrap()) {
                        self.draw_nib(ui, rect, x, y);
                    } else if let Some(c) = cell {
                        let idx = c.idx;
                        let bc = c.bc;
                        let fc = c.fc;
                        self.tile.paint_in_rect(ui, rect, idx, fc, Some(bc));
                    } else if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::none(),
                            Self::get_gray(x, y),
                        );
                    }
                }
            }
        }

        if res.hovered() && res.dragged() && hover_pos != None && rect.contains(hover_pos.unwrap())
        {
            let (x, y) = get_grid_x_y(rect, hover_pos.unwrap(), TILE_SIZE_VEC2);
            let ctx = ui.ctx();
            if ctx.input(|i| i.pointer.primary_down()) {
                let cell_ref = self.canvas.get_cell_mut(x, y);
                *cell_ref = self.pencil_state.into_tile_state(cell_ref);
            } else if ctx.input(|i| i.pointer.secondary_down()) {
                *(self.canvas.get_cell_mut(x, y)) = self.pencil_state.swap_color_and_into();
            }
        }
    }

    fn draw_palette(&mut self, ui: &mut egui::Ui) {
        self.pencil_state.draw_palette(ui);
    }

    fn draw_pencil_colors(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("颜色");
                self.pencil_state.color_editer(ui);
            });
            egui::Grid::new("pencil-colors")
                .min_col_width(TILE_SIZE)
                .num_columns(2)
                .show(ui, |ui| {
                    self.pencil_state.fore_color_checkbox(ui);
                    let (rect, res) = ui.allocate_exact_size(TILE_SIZE_VEC2, egui::Sense::click());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::none(),
                            self.pencil_state.fc,
                        );
                    }
                    if res.clicked() {
                        self.pencil_state.swap_fc_bc();
                    }

                    ui.end_row();
                    self.pencil_state.back_color_checkbox(ui);
                    let (rect, res) = ui.allocate_exact_size(TILE_SIZE_VEC2, egui::Sense::click());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::none(),
                            self.pencil_state.bc,
                        );
                    }
                    if res.clicked() {
                        std::mem::swap(&mut self.pencil_state.fc, &mut self.pencil_state.bc);
                    }
                    ui.end_row();
                });
        });
    }

    fn char_preview(&self, idx: usize, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(
                self.tile
                    .to_image(idx, egui::Vec2::splat(TILE_SIZE * 1.5))
                    .tint(self.pencil_state.fc)
                    .bg_fill(self.pencil_state.bc),
            );
            ui.label(
                egui::RichText::new(idx.to_string())
                    .size(TILE_SIZE * 1.5)
                    .strong()
                    .heading(),
            );
        });
    }

    fn current_cell_info(&self, ui: &mut egui::Ui) {
        ui.heading("信息");
        egui::Grid::new("info-grid").show(ui, |ui| {
            ui.label("画布尺寸：");
            ui.label(format!("{}x{}", self.canvas.size_x, self.canvas.size_y));
            ui.end_row();
            if let Some(cell) = self.cur_cell {
                ui.label("id：");
                ui.label(format!("{}", cell.idx));
                ui.end_row();
                ui.label("前景色：");
                let (r, g, b, _) = cell.fc.to_tuple();
                ui.label(format!("({:02X}, {:02X}, {:02X})", r, g, b));
                ui.end_row();
                ui.label("背景色：");
                let (r, g, b, _) = cell.bc.to_tuple();
                ui.label(format!("({:02X}, {:02X}, {:02X})", r, g, b));
                ui.end_row();
            } else {
                ui.label("id：");
                ui.label("_");
                ui.end_row();
                ui.label("前景色：");
                ui.label("_");
                ui.end_row();
                ui.label("背景色：");
                ui.label("_");
                ui.end_row();
            }
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
                                ImageButton::new(Some(self.tile.tex.id()), TILE_SIZE_VEC2)
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
                ui.horizontal(|ui| {
                    self.draw_palette(ui);
                    ui.separator();
                    self.draw_pencil_colors(ui);
                });
                ui.separator();
                self.current_cell_info(ui);
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
