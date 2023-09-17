#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod canvas;
mod color_editer;
mod export_image;
mod file;
mod image_button;
mod new_file;
mod setup;
mod tile;
mod undo;

use canvas::{Canvas, CanvasSizeEditWindow};
use color_editer::PencilState;
use eframe::egui;
use file::{load_canvas_from_file, write_canvas_to_file};
use rust_i18n::t;
use tile::TileSet;

rust_i18n::i18n!("locals", fallback = "zh-CN");

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

use undo::History;

use color_editer::ToolEnum;

struct FakePaint {
    tile: TileSet,
    pencil_state: PencilState,
    canvas: Canvas,
    rendering_scale: f32,
    editing_history: History,
    cur_cell: Option<(Option<canvas::TileState>, usize, usize)>,
    editing_file_path: Option<String>,
    new_file_window: new_file::NewFileWinodw,
    export_image_window: export_image::ExportImageWindow,
    canvas_size_window: CanvasSizeEditWindow,
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

fn compute_grid_rect(
    left_top: egui::Pos2,
    grid_size: egui::Vec2,
    x: usize,
    y: usize,
) -> egui::Rect {
    let egui::Pos2 {
        x: left_top_x,
        y: left_top_y,
    } = left_top;
    egui::Rect::from_min_size(
        egui::pos2(
            (left_top_x + grid_size.x * x as f32).floor(),
            (left_top_y + grid_size.y * y as f32).floor(),
        ),
        grid_size,
    )
}

impl FakePaint {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use color_editer::StoragePen;
        let pen: StoragePen;
        let canvas: Canvas;
        let palette: Vec<egui::Color32>;
        let scale: f32;
        let mut editing_file_path: Option<String>;

        if let Some(storage) = cc.storage {
            scale =
                serde_json::from_str(&storage.get_string("scale").unwrap_or_else(|| String::new()))
                    .unwrap_or_else(|_| 1.0);

            pen = serde_json::from_str(&storage.get_string("pen").unwrap_or_else(|| String::new()))
                .unwrap_or_else(|_| StoragePen::default());
            palette = serde_json::from_str(
                &storage
                    .get_string("palette")
                    .unwrap_or_else(|| String::new()),
            )
            .unwrap_or_else(|_| vec![egui::Color32::WHITE, egui::Color32::BLACK]);
            editing_file_path = serde_json::from_str(
                &storage
                    .get_string("editing_file_path")
                    .unwrap_or_else(|| String::new()),
            )
            .unwrap_or_else(|_| None);

            if let Some(path) = editing_file_path.clone() {
                if let Ok(cc) = load_canvas_from_file(&std::path::Path::new(&path)) {
                    canvas = cc;
                } else {
                    canvas = Canvas::default();
                    editing_file_path = None;
                }
            } else {
                canvas = serde_json::from_str(
                    &storage
                        .get_string("canvas")
                        .unwrap_or_else(|| String::new()),
                )
                .unwrap_or_else(|_| Canvas::default());
            }
        } else {
            scale = 1.0;
            pen = StoragePen::default();
            canvas = Canvas::default();
            palette = vec![egui::Color32::WHITE, egui::Color32::BLACK];
            editing_file_path = None;
        }

        setup::custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let (tex_handle, image_data) = tile::load_texture(&cc.egui_ctx);
        let mut r = Self {
            tile: TileSet::new(image_data, tex_handle.unwrap(), 16, 16, TILE_SIZE_VEC2),
            pencil_state: PencilState::from(pen),
            canvas,
            editing_history: History::new(),
            cur_cell: None,
            editing_file_path,
            rendering_scale: scale,
            new_file_window: new_file::NewFileWinodw::default(),
            export_image_window: export_image::ExportImageWindow::default(),
            canvas_size_window: CanvasSizeEditWindow::default(),
        };
        r.pencil_state.palette = color_editer::Palette::from(palette);
        r
    }

    fn draw_pencil_state(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{}: ", t!("pen"))).size(24.0));
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

    fn draw_nib(
        &mut self,
        ui: &mut egui::Ui,
        start_pos: egui::Pos2,
        rect: egui::Rect,
        x: usize,
        y: usize,
        rendering_canvas: &Canvas,
    ) {
        fn compute_color(c: egui::Color32) -> egui::Color32 {
            const ADD: u16 = 128;
            let (r, g, b, _) = c.to_tuple();
            let r = r as u16;
            let g = g as u16;
            let b = b as u16;
            let r = ((r + ADD) % (u8::MAX as u16)) as u8;
            let g = ((g + ADD) % (u8::MAX as u16)) as u8;
            let b = ((b + ADD) % (u8::MAX as u16)) as u8;
            egui::Color32::from_rgb(r, g, b)
        }
        let cell = rendering_canvas.get_cell(x, y);
        self.cur_cell = Some((*cell, x, y));
        let pencil = &self.pencil_state;
        if let Some((fc, bc)) = pencil.get_fc_bc(cell) {
            if pencil.tool == ToolEnum::RectFilled && pencil.start_xy != None {
                let cur_tile_size_vec2 = rect.size();
                let (mut start_x, mut start_y) = pencil.start_xy.unwrap();
                let mut to_x = x;
                let mut to_y = y;
                if to_x < start_x {
                    std::mem::swap(&mut start_x, &mut to_x)
                }
                if to_y < start_y {
                    std::mem::swap(&mut start_y, &mut to_y)
                }
                for y in start_y..=to_y {
                    for x in start_x..=to_x {
                        let rect = compute_grid_rect(start_pos, cur_tile_size_vec2, x, y);
                        self.tile.paint_in_rect(ui, rect, pencil.idx, fc, Some(bc));
                    }
                }
            } else {
                self.tile.paint_in_rect(ui, rect, pencil.idx, fc, Some(bc))
            }
        } else {
            let bc;
            if let &Some(tile) = cell {
                bc = tile.bc;
                self.tile
                    .paint_in_rect(ui, rect, tile.idx, tile.fc, Some(bc));
            } else {
                bc = Self::get_gray(x, y);
                ui.painter().rect_filled(rect, egui::Rounding::none(), bc)
            }
            if pencil.tool == ToolEnum::RectFilled && pencil.start_xy != None {
            } else {
                ui.painter().rect_stroke(
                    rect,
                    egui::Rounding::none(),
                    egui::Stroke::new(1.5, compute_color(bc)),
                );
            }
        }
    }

    fn draw_canvas(
        &mut self,
        ui: &mut egui::Ui,
        rendering_canvas: &Canvas,
        render_size: egui::Vec2,
    ) {
        let (rect, res) = ui.allocate_exact_size(render_size, egui::Sense::drag());
        let hover_pos = res.hover_pos();
        let cur_tile_size_vec2 = TILE_SIZE_VEC2 * self.rendering_scale;
        let mut hover_xy: Option<(usize, usize, egui::Rect)> = None;
        if ui.is_rect_visible(rect) {
            let left_top = rect.left_top();
            for y in 0..rendering_canvas.height {
                for x in 0..rendering_canvas.width {
                    let rect = compute_grid_rect(left_top, cur_tile_size_vec2, x, y);
                    let cell = rendering_canvas.get_cell(x, y);

                    if hover_pos != None && rect.contains(hover_pos.unwrap()) {
                        hover_xy = Some((x, y, rect));
                    }
                    if let Some(c) = cell {
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
            if let Some((x, y, rect)) = hover_xy {
                self.draw_nib(ui, left_top, rect, x, y, &rendering_canvas);
            }
        }

        use undo::Command;
        if res.hovered() && hover_pos != None && rect.contains(hover_pos.unwrap()) {
            let (x, y) = get_grid_x_y(rect, hover_pos.unwrap(), cur_tile_size_vec2);
            let cell_ref = rendering_canvas.get_cell(x, y);
            if self.pencil_state.tool == ToolEnum::RectFilled {
                if res.drag_started() {
                    self.pencil_state.start_xy = Some((x, y));
                } else if res.drag_released() {
                    self.pencil_state.to_xy = Some((x, y));
                    self.editing_history.push(Command::new(
                        x,
                        y,
                        &self.pencil_state,
                        cell_ref,
                        false,
                        rendering_canvas,
                    ));
                    self.pencil_state.start_xy = None;
                    self.pencil_state.to_xy = None;
                }
            } else {
                let ctx = ui.ctx();
                if ctx.input(|i| i.pointer.primary_down()) {
                    self.editing_history.push(Command::new(
                        x,
                        y,
                        &self.pencil_state,
                        cell_ref,
                        false,
                        rendering_canvas,
                    ));
                } else if ctx.input(|i| i.pointer.secondary_down()) {
                    self.editing_history.push(Command::new(
                        x,
                        y,
                        &self.pencil_state,
                        cell_ref,
                        true,
                        rendering_canvas,
                    ));
                }
            }
        }
    }

    fn draw_canvas_in_scroll_area(&mut self, ui: &mut egui::Ui, rendering_canvas: &Canvas) {
        self.cur_cell = None;

        let available_size = ui.available_size();

        let mut outer_size = egui::vec2(
            self.rendering_scale * TILE_SIZE * rendering_canvas.width as f32,
            self.rendering_scale * TILE_SIZE * rendering_canvas.height as f32,
        );

        let render_size = outer_size;

        if outer_size.x < available_size.x {
            outer_size.x = available_size.x * 2.0 - outer_size.x;
        }

        if outer_size.y < available_size.y {
            outer_size.y = available_size.y * 2.0 - outer_size.y;
        }

        let (_, outer_rect) = ui.allocate_space(outer_size);
        ui.allocate_ui_at_rect(outer_rect, |ui| {
            ui.centered_and_justified(|ui| {
                self.draw_canvas(ui, rendering_canvas, render_size);
            });
        });
    }

    fn draw_palette(&mut self, ui: &mut egui::Ui) {
        self.pencil_state.draw_palette(ui);
    }

    fn draw_pencil_colors(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(t!("color"));
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
            ui.horizontal(|ui| {
                if ui.button(t!("undo")).clicked() {
                    self.editing_history.undo();
                }
                if ui.button(t!("redo")).clicked() {
                    self.editing_history.redo();
                }
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

    fn current_canvas_info(&self, ui: &mut egui::Ui, rendering_canvas: &Canvas) {
        ui.heading(t!("info"));
        egui::Grid::new("info-grid").show(ui, |ui| {
            ui.label(format!("{}: ", t!("file")));
            if let Some(string) = &self.editing_file_path {
                let string = std::path::Path::new(string)
                    .file_stem()
                    .unwrap_or_else(|| &std::ffi::OsStr::new(""))
                    .to_string_lossy();
                if string.len() > 25 {
                    ui.label(format!("{:.25}...", string));
                } else {
                    ui.label(string);
                }
            }
            ui.end_row();
            ui.label(format!("{}: ", t!("canvas_size")));
            ui.label(format!(
                "{}x{}",
                rendering_canvas.width, rendering_canvas.height
            ));
            ui.end_row();

            fn draw_none_cell(ui: &mut egui::Ui) {
                ui.label("id: ");
                ui.label("_");
                ui.end_row();
                ui.label(format!("{}: ", t!("foreground_color")));
                ui.label("_");
                ui.end_row();
                ui.label(format!("{}: ", t!("background_color")));
                ui.label("_");
                ui.end_row();
            }
            if let Some((cell, x, y)) = self.cur_cell {
                ui.label("x,y: ");
                ui.label(format!("{}, {}", x, y));
                ui.end_row();

                if let Some(cell) = cell {
                    ui.label("id: ");
                    ui.label(format!("{}", cell.idx));
                    ui.end_row();
                    ui.label(format!("{}: ", t!("foreground_color")));
                    let (r, g, b, _) = cell.fc.to_tuple();
                    ui.label(format!("({:02X}, {:02X}, {:02X})", r, g, b));
                    ui.end_row();
                    ui.label(format!("{}: ", t!("background_color")));
                    let (r, g, b, _) = cell.bc.to_tuple();
                    ui.label(format!("({:02X}, {:02X}, {:02X})", r, g, b));
                    ui.end_row();
                } else {
                    draw_none_cell(ui);
                }
            } else {
                ui.label("x,y: ");
                ui.label("_, _");
                ui.end_row();

                draw_none_cell(ui);
            }
        });
    }

    fn char_selector(&mut self, ui: &mut egui::Ui) {
        use image_button::ImageButton;
        let idx = self.pencil_state.idx;
        let x = idx % self.tile.columns;
        let y = idx / self.tile.columns;
        ui.heading(format!("{}--({},{})", t!("char"), x, y));
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
        let rendering_canvas = self.editing_history.excute_on_canvas(&self.canvas);

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button(t!("file"), |ui| {
                    if ui.button(t!("open")).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title(t!("select_json"))
                            .add_filter("json", &["json"])
                            .pick_file()
                        {
                            if let Ok(cc) = load_canvas_from_file(std::path::Path::new(&path)) {
                                self.canvas = cc;
                                if let Some(string) = path.to_str() {
                                    self.editing_file_path = Some(string.to_string());
                                    self.editing_history.clear();
                                }
                            }
                            // else {
                            //     rfd::MessageDialog::new()
                            //         .set_description("加载错误")
                            //         .show();
                            // }
                        }
                        ui.close_menu();
                    }
                    if ui.button(t!("new")).clicked() {
                        self.new_file_window.open();
                        ui.close_menu();
                    }
                    if ui.button(t!("save")).clicked() {
                        self.canvas = self.editing_history.excute_on_canvas(&self.canvas);
                        self.editing_history.clear();
                        if let Some(path) = &self.editing_file_path {
                            let _ = write_canvas_to_file(&self.canvas, &std::path::Path::new(path));
                        } else {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title(t!("save"))
                                .add_filter("json", &["json"])
                                .save_file()
                            {
                                let _ = write_canvas_to_file(&self.canvas, &path);
                                if let Some(string) = path.to_str() {
                                    self.editing_file_path = Some(string.to_string());
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button(t!("save_as")).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title(t!("save"))
                            .add_filter("json", &["json"])
                            .save_file()
                        {
                            self.canvas = self.editing_history.excute_on_canvas(&self.canvas);
                            self.editing_history.clear();
                            let _ = write_canvas_to_file(&self.canvas, &path);
                            if let Some(string) = path.to_str() {
                                self.editing_file_path = Some(string.to_string());
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button(t!("export")).clicked() {
                        self.export_image_window.open();
                        ui.close_menu();
                    }
                });
                ui.menu_button(t!("edit"), |ui| {
                    if ui.button(t!("canvas_size")).clicked() {
                        self.canvas_size_window
                            .open(rendering_canvas.width, rendering_canvas.height);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            self.draw_pencil_state(ui);
            self.char_selector(ui);
            ui.separator();
            ui.horizontal(|ui| {
                self.draw_palette(ui);
                ui.separator();
                self.draw_pencil_colors(ui);
            });
            ui.separator();
            self.current_canvas_info(ui, &rendering_canvas);
        });

        egui::SidePanel::right("right_panel")
            .min_width(48.0)
            .show(ctx, |ui| {
                ui.heading(t!("tool"));
                ui.separator();
                ui.selectable_value(&mut self.pencil_state.tool, ToolEnum::Pencil, t!("pencil"));
                ui.selectable_value(&mut self.pencil_state.tool, ToolEnum::Eraser, t!("eraser"));
                ui.selectable_value(&mut self.pencil_state.tool, ToolEnum::Fill, t!("fill"));
                ui.selectable_value(
                    &mut self.pencil_state.tool,
                    ToolEnum::Replace,
                    t!("replace"),
                );
                ui.selectable_value(
                    &mut self.pencil_state.tool,
                    ToolEnum::RectFilled,
                    t!("rect_filled"),
                );
            });

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom_panel").show(
            ctx,
            |ui| {
                ui.add(
                    egui::widgets::Slider::new(&mut self.rendering_scale, 0.125..=8.0)
                        .logarithmic(true)
                        .step_by(0.125)
                        .text(t!("scale")),
                );
            },
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            self.export_image_window
                .show(ctx, &rendering_canvas, &self.tile);
            if let Some(cmd) = self.canvas_size_window.show(ctx) {
                self.editing_history.push(cmd);
            }
            if self
                .new_file_window
                .show(ctx, &mut self.canvas, &mut self.editing_file_path)
            {
                self.editing_history.clear();
            }
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    self.draw_canvas_in_scroll_area(ui, &rendering_canvas);
                });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(string) = serde_json::to_string(&self.rendering_scale) {
            storage.set_string("scale", string);
        }

        if let Ok(string) =
            serde_json::to_string(&color_editer::StoragePen::from(&self.pencil_state))
        {
            storage.set_string("pen", string);
        }

        if let Ok(string) = serde_json::to_string(&self.canvas) {
            storage.set_string("canvas", string);
        }

        if let Ok(string) = serde_json::to_string(self.pencil_state.palette_vec_ref()) {
            storage.set_string("palette", string);
        }

        if let Ok(string) = serde_json::to_string(&self.editing_file_path) {
            storage.set_string("editing_file_path", string);
        }
    }
}
