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

pub struct PencilState {
    pub idx: usize,
    pub fc: egui::Color32,
    pub bc: egui::Color32,
    fc_activate: bool,
    bc_activate: bool,
    state: ColorEditerState,
    old_color: egui::Color32,
    text: String,
    edting: EditingColor,
    is_gray: bool,
}

impl PencilState {
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
        ui.add(ColorEditer::new(self))
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
            edting: EditingColor::FORE,
            is_gray: false,
        }
    }
}

struct ColorEditer<'c> {
    color: &'c mut egui::Color32,
    other_color: egui::Color32,
    state: &'c mut ColorEditerState,
    old_color: &'c mut egui::Color32,
    text: &'c mut String,
    editing: &'c mut EditingColor,
    is_gray: &'c mut bool,
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

impl<'c> ColorEditer<'c> {
    pub fn new(pen: &'c mut PencilState) -> Self {
        let (color, other_color) = if pen.edting == EditingColor::FORE {
            (&mut pen.fc, pen.bc)
        } else {
            (&mut pen.bc, pen.fc)
        };
        Self {
            old_color: &mut pen.old_color,
            color,
            other_color,
            state: &mut pen.state,
            text: &mut pen.text,
            editing: &mut pen.edting,
            is_gray: &mut pen.is_gray,
        }
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

impl egui::Widget for ColorEditer<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let res = ui
            .menu_button("编辑", |ui| {
                ui.horizontal(|ui| {
                    show_old_and_new_color(ui, *self.old_color, *self.color);
                    let res = ui.add(egui::widgets::TextEdit::singleline(self.text).char_limit(6));
                    if res.lost_focus() {
                        if self.text.len() == 6 {
                            let r = u8::from_str_radix(&self.text[0..2], 16);
                            let g = u8::from_str_radix(&self.text[2..4], 16);
                            let b = u8::from_str_radix(&self.text[4..6], 16);
                            if r.is_ok() && g.is_ok() && b.is_ok() {
                                *self.color =
                                    egui::Color32::from_rgb(r.unwrap(), g.unwrap(), b.unwrap());
                                update_text(self.text, self.color);
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
                                *self.color =
                                    egui::Color32::from_rgb(r.unwrap(), g.unwrap(), b.unwrap());
                                update_text(self.text, self.color);
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_enabled(!*self.is_gray);
                    if ui
                        .selectable_label(*self.state == ColorEditerState::RGB, "RGB")
                        .clicked()
                    {
                        *self.state = ColorEditerState::RGB;
                    }

                    if ui
                        .selectable_label(*self.state == ColorEditerState::HSV, "HSV")
                        .clicked()
                    {
                        *self.state = ColorEditerState::HSV;
                    }

                    if ui
                        .selectable_label(*self.state == ColorEditerState::HSL, "HSL")
                        .clicked()
                    {
                        *self.state = ColorEditerState::HSL;
                    }

                    if ui.button("＋").on_hover_text("添加到调色板").clicked() {
                        ui.close_menu();
                        *self.color = *self.old_color;
                    }

                    if ui.button("取消").clicked() {
                        ui.close_menu();
                        *self.color = *self.old_color;
                    }
                });
                let (r, g, b, _) = self.color.to_tuple();
                ui.horizontal(|ui| {
                    match *self.state {
                        ColorEditerState::RGB => {
                            rgb_editer(ui, !*self.is_gray, self.color, r, g, b)
                        }
                        ColorEditerState::HSV => {
                            hsv_editer(ui, !*self.is_gray, self.color, r, g, b)
                        }
                        ColorEditerState::HSL => {
                            hsl_editer(ui, !*self.is_gray, self.color, r, g, b)
                        }
                    };
                    ui.vertical(|ui| {
                        let res = if *self.editing == EditingColor::FORE {
                            ui.selectable_value(self.editing, EditingColor::BACK, "前景色")
                        } else {
                            ui.selectable_value(self.editing, EditingColor::FORE, "背景色")
                        };
                        if res.changed() {
                            *self.old_color = self.other_color;
                        }
                        if ui.toggle_value(self.is_gray, "GRAY").clicked() {
                            let (r, g, b, _) = self.color.to_tuple();
                            let (r, g, b) = gray_to_rgb(rgb_to_gray(r, g, b));
                            *self.color = egui::Color32::from_rgb(r, g, b);
                        }
                        ui.set_enabled(*self.is_gray);
                        gray_editer(ui, self.color, r, g, b);
                    });
                });
            })
            .response;
        if res.clicked() {
            update_text(self.text, self.color);
            *self.old_color = *self.color;
        }
        res
    }
}
