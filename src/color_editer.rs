use std::collections::HashMap;

use crate::canvas::TileState;
use eframe::egui;
use palette::FromColor;

#[derive(PartialEq)]
enum ColorEditerState {
    RGB,
    HSV,
    HSL,
}

#[derive(PartialEq)]
enum EditingColor {
    FORE,
    BACK,
}

pub struct Palette {
    pub palette: Vec<egui::Color32>,
    color_index_hash_map: HashMap<egui::Color32, usize>,
    editing: bool,
}

impl Palette {
    fn get_color(&self, idx: usize) -> egui::Color32 {
        self.palette[idx]
    }

    pub fn add_color(&mut self, color: egui::Color32) {
        if !self.contains_color(color) {
            self.palette.push(color);
            self.color_index_hash_map.insert(color, self.palette.len());
        }
    }

    pub fn delete_color(&mut self, idx: usize) {
        let color = self.palette.remove(idx);
        self.color_index_hash_map.remove(&color);
        for (i, c) in self.palette[idx..self.palette.len()].iter().enumerate() {
            self.color_index_hash_map.insert(*c, i);
        }
    }

    pub fn contains_color(&self, color: egui::Color32) -> bool {
        self.color_index_hash_map.contains_key(&color)
    }
}

impl Default for Palette {
    fn default() -> Self {
        let palette = vec![egui::Color32::WHITE, egui::Color32::BLACK];
        let mut color_index_hash_map = HashMap::with_capacity(palette.len());
        for (i, c) in palette.iter().enumerate() {
            color_index_hash_map.insert(*c, i);
        }
        Self {
            palette,
            color_index_hash_map,
            editing: false,
        }
    }
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct StoragePen {
    idx: usize,
    fc: [u8; 3],
    bc: [u8; 3],
}

impl Default for StoragePen {
    fn default() -> Self {
        StoragePen {
            idx: 0,
            fc: [255, 255, 255],
            bc: [0, 0, 0],
        }
    }
}

pub struct PencilState {
    pub idx: usize,
    pub fc: egui::Color32,
    pub bc: egui::Color32,
    pub palette: Palette,
    fc_activate: bool,
    bc_activate: bool,
    state: ColorEditerState,
    old_color: egui::Color32,
    text: String,
    editing: EditingColor,
    is_gray: bool,
}

const PALETTE_X: usize = 6;
const PALETTE_Y: usize = 4;

impl From<StoragePen> for PencilState {
    fn from(value: StoragePen) -> Self {
        let mut r = Self::default();
        r.idx = value.idx;
        r.fc = egui::Color32::from_rgb(value.fc[0], value.fc[1], value.fc[2]);
        r.bc = egui::Color32::from_rgb(value.bc[0], value.bc[1], value.bc[2]);
        r
    }
}

impl From<&PencilState> for StoragePen {
    fn from(value: &PencilState) -> Self {
        let fc = value.fc.to_opaque();
        let bc = value.bc.to_opaque();
        Self {
            idx: value.idx,
            fc: [fc.r(), fc.g(), fc.b()],
            bc: [bc.r(), bc.g(), bc.b()],
        }
    }
}

impl From<PencilState> for StoragePen {
    fn from(value: PencilState) -> Self {
        let fc = value.fc.to_opaque();
        let bc = value.bc.to_opaque();
        Self {
            idx: value.idx,
            fc: [fc.r(), fc.g(), fc.b()],
            bc: [bc.r(), bc.g(), bc.b()],
        }
    }
}

impl PencilState {
    pub fn palette_vec_ref(&self) -> &Vec<egui::Color32> {
        &self.palette.palette
    }
    pub fn draw_palette(&mut self, ui: &mut egui::Ui) {
        use crate::{TILE_SIZE, TILE_SIZE_VEC2};
        use egui::containers::scroll_area::ScrollBarVisibility;
        const SCALE_FACT: f32 = 1.25;
        fn fill_rest(mut last: usize, mut rest: usize, ui: &mut egui::Ui) {
            const DEFAULT_PALETTE_GRID_SIZE: usize = PALETTE_X * PALETTE_Y;
            let tile_size = TILE_SIZE_VEC2 * SCALE_FACT;
            if last == 0 && rest == DEFAULT_PALETTE_GRID_SIZE {
                for _ in last..PALETTE_X {
                    let (rect, _) = ui.allocate_exact_size(tile_size, egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::none(),
                            egui::Color32::TRANSPARENT,
                        );
                    }
                }
                last = PALETTE_X;
                rest -= PALETTE_X;
            }
            for i in last..rest + last {
                if i % PALETTE_X == 0 {
                    ui.end_row()
                }
                let (rect, _) = ui.allocate_exact_size(tile_size, egui::Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::none(),
                        egui::Color32::TRANSPARENT,
                    );
                }
            }
        }

        fn draw_color(
            color: egui::Color32,
            ui: &mut egui::Ui,
            stroke: egui::Stroke,
            fc: egui::Color32,
            bc: egui::Color32,
        ) -> egui::Response {
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
            let (rect, res) =
                ui.allocate_exact_size(TILE_SIZE_VEC2 * SCALE_FACT, egui::Sense::click());
            if ui.is_rect_visible(rect) {
                ui.painter()
                    .rect_filled(rect, egui::Rounding::none(), egui::Color32::DARK_GRAY);
                ui.painter().rect_filled(
                    crate::get_center_rect(&rect, TILE_SIZE_VEC2),
                    egui::Rounding::none(),
                    color,
                );
                if res.hovered() {
                    ui.painter()
                        .rect_stroke(rect, egui::Rounding::none(), stroke)
                }
                if fc == color {
                    let rect = egui::Rect::from_min_size(rect.min, TILE_SIZE_VEC2 * 0.5);
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), compute_color(color));
                }
                if bc == color {
                    let rect = egui::Rect::from_min_max(rect.max - TILE_SIZE_VEC2 * 0.5, rect.max);
                    ui.painter()
                        .rect_filled(rect, egui::Rounding::none(), compute_color(color));
                }
            }
            res
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("调色板");
                self.palette_state_toggle(ui);
            });
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    egui::Grid::new("palette-colors-grid")
                        .spacing(egui::Vec2::ZERO)
                        .striped(true)
                        .num_columns(8)
                        .min_col_width(TILE_SIZE)
                        .min_row_height(TILE_SIZE)
                        .show(ui, |ui| {
                            let len = self.palette_len();
                            let need_fill_rest = len < PALETTE_X;
                            let mut i = 0;
                            let first_line = if need_fill_rest { len } else { PALETTE_X };
                            let stroke = ui.style().visuals.selection.stroke;
                            let mut delete = Vec::new();
                            while i < first_line {
                                let c = self.get_color(i);
                                let res = draw_color(c, ui, stroke, self.fc, self.bc);
                                if res.clicked() {
                                    if let Some(idx) = self.do_click_color_action(i) {
                                        delete.push(idx);
                                    }
                                } else if res.secondary_clicked() {
                                    if let Some(idx) = self.do_secondary_click_color_action(i) {
                                        delete.push(idx);
                                    }
                                }
                                i += 1;
                            }
                            while i < len {
                                let c = self.get_color(i);
                                if i % PALETTE_X == 0 {
                                    ui.end_row();
                                }
                                let res = draw_color(c, ui, stroke, self.fc, self.bc);
                                if res.clicked() {
                                    if let Some(idx) = self.do_click_color_action(i) {
                                        delete.push(idx);
                                    }
                                } else if res.secondary_clicked() {
                                    if let Some(idx) = self.do_secondary_click_color_action(i) {
                                        delete.push(idx);
                                    }
                                }
                                i += 1;
                            }

                            for idx in delete {
                                self.delete_color(idx);
                            }

                            let y = len / PALETTE_X;
                            if y < PALETTE_Y {
                                let last_x = len % PALETTE_X;
                                fill_rest(
                                    last_x,
                                    (PALETTE_X - last_x) + (PALETTE_Y - 1 - y) * PALETTE_X,
                                    ui,
                                );
                            }
                        });
                });
        });
    }
    pub fn swap_fc_bc(&mut self) {
        std::mem::swap(&mut self.fc, &mut self.bc);
        self.state = ColorEditerState::RGB;
    }

    pub fn get_fc_bc(&self, cell: &Option<TileState>) -> Option<(egui::Color32, egui::Color32)> {
        if !self.fc_activate && !self.bc_activate {
            None
        } else if *cell == None {
            Some((self.fc, self.bc))
        } else {
            Some((
                if self.fc_activate {
                    self.fc
                } else {
                    cell.unwrap().fc
                },
                if self.bc_activate {
                    self.bc
                } else {
                    cell.unwrap().bc
                },
            ))
        }
    }

    pub fn delete_color(&mut self, idx: usize) {
        self.palette.delete_color(idx);
    }
    pub fn do_click_color_action(&mut self, idx: usize) -> Option<usize> {
        if self.palette.editing {
            Some(idx)
        } else {
            self.fc = self.palette.get_color(idx);
            None
        }
    }

    pub fn do_secondary_click_color_action(&mut self, idx: usize) -> Option<usize> {
        if self.palette.editing {
            Some(idx)
        } else {
            self.bc = self.palette.get_color(idx);
            None
        }
    }

    pub fn palette_state_toggle(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.toggle_value(&mut self.palette.editing, "删除")
    }

    pub fn get_color(&self, idx: usize) -> egui::Color32 {
        self.palette.get_color(idx)
    }

    pub fn palette_len(&self) -> usize {
        self.palette.palette.len()
    }

    pub fn into_tile_state(&self, origin_state: &Option<TileState>) -> Option<TileState> {
        if let Some((fc, bc)) = self.get_fc_bc(origin_state) {
            Some(TileState {
                idx: self.idx,
                fc,
                bc,
            })
        } else {
            None
        }
    }

    pub fn fore_color_checkbox(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.fc_activate, "前景色：")
    }

    pub fn back_color_checkbox(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.bc_activate, "背景色：")
    }

    pub fn swap_color_and_into(&self) -> Option<TileState> {
        Some(TileState {
            idx: self.idx,
            fc: self.bc,
            bc: self.fc,
        })
    }

    pub fn color_editer(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (color, other_color) = if self.editing == EditingColor::FORE {
            (&mut self.fc, self.bc)
        } else {
            (&mut self.bc, self.fc)
        };
        let res = ui
            .menu_button("编辑", |ui| {
                ui.horizontal(|ui| {
                    show_old_and_new_color(ui, self.old_color, *color);
                    let res =
                        ui.add(egui::widgets::TextEdit::singleline(&mut self.text).char_limit(6));
                    if res.lost_focus() {
                        if self.text.len() == 6 {
                            let r = u8::from_str_radix(&self.text[0..2], 16);
                            let g = u8::from_str_radix(&self.text[2..4], 16);
                            let b = u8::from_str_radix(&self.text[4..6], 16);
                            if r.is_ok() && g.is_ok() && b.is_ok() {
                                *color =
                                    egui::Color32::from_rgb(r.unwrap(), g.unwrap(), b.unwrap());
                                update_text(&mut self.text, color);
                            }
                        } else if self.text.len() == 3 {
                            let r = u8::from_str_radix(
                                &format!("{}{}", &self.text[0..1], &self.text[0..1]),
                                16,
                            );
                            let g = u8::from_str_radix(
                                &format!("{}{}", &self.text[1..2], &self.text[1..2]),
                                16,
                            );
                            let b = u8::from_str_radix(
                                &format!("{}{}", &self.text[2..3], &self.text[2..3]),
                                16,
                            );
                            if r.is_ok() && g.is_ok() && b.is_ok() {
                                *color =
                                    egui::Color32::from_rgb(r.unwrap(), g.unwrap(), b.unwrap());
                                update_text(&mut self.text, color);
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_enabled(!self.is_gray);
                    if ui
                        .selectable_label(self.state == ColorEditerState::RGB, "RGB")
                        .clicked()
                    {
                        self.state = ColorEditerState::RGB;
                    }

                    if ui
                        .selectable_label(self.state == ColorEditerState::HSV, "HSV")
                        .clicked()
                    {
                        self.state = ColorEditerState::HSV;
                    }

                    if ui
                        .selectable_label(self.state == ColorEditerState::HSL, "HSL")
                        .clicked()
                    {
                        self.state = ColorEditerState::HSL;
                    }

                    if ui.button("＋").on_hover_text("添加到调色板").clicked() {
                        self.palette.add_color(*color);
                    }

                    if ui.button("取消").clicked() {
                        ui.close_menu();
                        *color = self.old_color;
                    }
                });
                let (r, g, b, _) = color.to_tuple();
                ui.horizontal(|ui| {
                    match self.state {
                        ColorEditerState::RGB => rgb_editer(ui, !self.is_gray, color, r, g, b),
                        ColorEditerState::HSV => hsv_editer(ui, !self.is_gray, color, r, g, b),
                        ColorEditerState::HSL => hsl_editer(ui, !self.is_gray, color, r, g, b),
                    };
                    ui.vertical(|ui| {
                        let res = if self.editing == EditingColor::FORE {
                            ui.selectable_value(&mut self.editing, EditingColor::BACK, "前景色")
                        } else {
                            ui.selectable_value(&mut self.editing, EditingColor::FORE, "背景色")
                        };
                        if res.changed() {
                            self.old_color = other_color;
                        }
                        if ui.toggle_value(&mut self.is_gray, "GRAY").clicked() {
                            let (r, g, b, _) = color.to_tuple();
                            let (r, g, b) = gray_to_rgb(rgb_to_gray(r, g, b));
                            *color = egui::Color32::from_rgb(r, g, b);
                        }
                        ui.set_enabled(self.is_gray);
                        gray_editer(ui, color, r, g, b);
                    });
                });
            })
            .response;
        if res.clicked() {
            update_text(&mut self.text, color);
            self.old_color = *color;
        }
        res
    }
}

impl From<Vec<egui::Color32>> for Palette {
    fn from(value: Vec<egui::Color32>) -> Self {
        let mut color_index_hash_map = HashMap::with_capacity(value.len());
        for (i, c) in value.iter().enumerate() {
            color_index_hash_map.insert(*c, i);
        }
        Self {
            palette: value,
            color_index_hash_map,
            editing: false,
        }
    }
}

impl From<&Vec<egui::Color32>> for Palette {
    fn from(value: &Vec<egui::Color32>) -> Self {
        let mut color_index_hash_map = HashMap::with_capacity(value.len());
        for (i, c) in value.iter().enumerate() {
            color_index_hash_map.insert(*c, i);
        }
        Self {
            palette: value.clone(),
            color_index_hash_map,
            editing: false,
        }
    }
}

impl Default for PencilState {
    fn default() -> Self {
        Self {
            idx: 0,
            fc: egui::Color32::WHITE,
            bc: egui::Color32::BLACK,
            fc_activate: true,
            bc_activate: true,
            state: ColorEditerState::RGB,
            old_color: egui::Color32::WHITE,
            text: "".to_string(),
            editing: EditingColor::FORE,
            is_gray: false,
            palette: Palette::default(),
        }
    }
}

fn rgb_editer(ui: &mut egui::Ui, enabled: bool, color: &mut egui::Color32, r: u8, g: u8, b: u8) {
    let mut b = b as f32;
    let mut r = r as f32;
    let mut g = g as f32;
    ui.vertical(|ui| {
        ui.set_enabled(enabled);
        color_components_edit(ui, &mut r, "R", 0.0, 255.0);
        color_components_edit(ui, &mut g, "G", 0.0, 255.0);
        color_components_edit(ui, &mut b, "B", 0.0, 255.0);
    });
    *color = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
}

fn hsv_editer(ui: &mut egui::Ui, enabled: bool, color: &mut egui::Color32, r: u8, g: u8, b: u8) {
    let (mut h, mut s, mut v) = rgb_to_hsv(r, g, b);
    ui.vertical(|ui| {
        ui.set_enabled(enabled);
        color_components_edit(ui, &mut h, "H", 0.0, 360.0);
        color_components_edit(ui, &mut s, "S", 0.0, 100.0);
        color_components_edit(ui, &mut v, "V", 0.0, 100.0);
    });
    let (r, g, b) = hsv_to_rgb(h, s, v);
    *color = egui::Color32::from_rgb(r, g, b);
}

fn hsl_editer(ui: &mut egui::Ui, enabled: bool, color: &mut egui::Color32, r: u8, g: u8, b: u8) {
    let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);
    ui.vertical(|ui| {
        ui.set_enabled(enabled);
        color_components_edit(ui, &mut h, "H", 0.0, 360.0);
        color_components_edit(ui, &mut s, "S", 0.0, 100.0);
        color_components_edit(ui, &mut l, "L", 0.0, 100.0);
    });
    let (r, g, b) = hsl_to_rgb(h, s, l);
    *color = egui::Color32::from_rgb(r, g, b);
}

fn gray_editer(ui: &mut egui::Ui, color: &mut egui::Color32, r: u8, g: u8, b: u8) {
    let mut gray = rgb_to_gray(r, g, b);
    ui.vertical(|ui| {
        color_components_edit(ui, &mut gray, "V", 0.0, 255.0);
    });
    if ui.is_enabled() {
        let (r, g, b) = gray_to_rgb(gray);
        *color = egui::Color32::from_rgb(r, g, b);
    }
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let hsv = palette::Hsv::from_color(palette::LinSrgb::new(r, g, b).into_format::<f32>());
    (
        hsv.hue.into_degrees(),
        hsv.saturation * 100.0,
        hsv.value * 100.0,
    )
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let rgb = palette::LinSrgb::from_color(palette::hsv::Hsv::new(h, s / 100.0, v / 100.0))
        .into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let hsl = palette::Hsl::from_color(palette::LinSrgb::new(r, g, b).into_format::<f32>());
    (
        hsl.hue.into_degrees(),
        hsl.saturation * 100.0,
        hsl.lightness * 100.0,
    )
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let rgb = palette::LinSrgb::from_color(palette::Hsl::new(h, s / 100.0, l / 100.0))
        .into_format::<u8>();
    (rgb.red, rgb.green, rgb.blue)
}

fn rgb_to_gray(r: u8, g: u8, b: u8) -> u8 {
    (palette::Hsl::from_color(palette::LinSrgb::new(r, g, b).into_format::<f32>()).lightness
        * 255.0) as u8
}

fn gray_to_rgb(luma: u8) -> (u8, u8, u8) {
    (luma, luma, luma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv() {
        let (mut h, mut s, mut v) = rgb_to_hsv(255, 87, 87);
        h = h.floor();
        s = s.floor();
        v = v.floor();
        assert_eq!((h, s, v), (0.0, 65.0, 100.0));
    }

    #[test]
    fn test_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(0.0, 66.0, 100.0), (255, 87, 87));
    }

    #[test]
    fn test_rgb_to_hsl() {
        let (mut h, mut s, mut l) = rgb_to_hsl(255, 87, 87);
        h = h.floor();
        s = s.floor();
        l = l.floor();
        assert_eq!((h, s, l), (0.0, 100.0, 67.0));
    }

    #[test]
    fn test_hsl_to_rgb() {
        assert_eq!(hsl_to_rgb(0.0, 100.0, 67.0), (255, 87, 87));
    }

    #[test]
    fn test_rgb_to_gray() {
        assert_eq!(rgb_to_gray(255, 87, 87), 171);
    }

    #[test]
    fn test_gray_to_rgb() {
        assert_eq!(gray_to_rgb(171), (171, 171, 171));
    }
}

fn color_components_edit<Num: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    r: &mut Num,
    name: &str,
    start: f32,
    end: f32,
) -> egui::Response {
    ui.horizontal_wrapped(|ui| {
        ui.label(name);
        ui.add(
            egui::DragValue::new(r)
                .clamp_range(core::ops::RangeInclusive::new(start, end))
                .fixed_decimals(0),
        );
    })
    .response
}

fn show_old_and_new_color(ui: &mut egui::Ui, old: egui::Color32, new: egui::Color32) {
    let rect_height = crate::TILE_SIZE * 1.5;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(rect_height * 2.0, rect_height),
        egui::Sense::hover(),
    );
    if ui.is_rect_visible(rect) {
        let rect_size = egui::Vec2::splat(rect_height);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, rect_size),
            egui::Rounding::none(),
            old,
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.center_top(), rect_size),
            egui::Rounding::none(),
            new,
        )
    }
}

fn update_text(text: &mut String, color: &egui::Color32) {
    let (r, g, b, _) = color.to_tuple();
    *text = format!("{:02X}{:02X}{:02X}", r, g, b);
}
