use crate::canvas::TileState;
use eframe::egui;
use palette::FromColor;

#[derive(Clone, PartialEq)]
enum ColorEditerState {
    RGB,
    HSV,
    HSL,
    GRAY,
}

#[derive(Clone)]
pub struct PencilState {
    pub idx: usize,
    pub fc: egui::Color32,
    pub bc: egui::Color32,
    open: bool,
    state: ColorEditerState,
}

impl PencilState {
    pub fn into_tile_state(&self) -> TileState {
        TileState {
            idx: self.idx,
            fc: self.fc,
            bc: self.bc,
        }
    }

    pub fn swap_color_and_into(&self) -> TileState {
        TileState {
            idx: self.idx,
            fc: self.bc,
            bc: self.fc,
        }
    }
}

impl Default for PencilState {
    fn default() -> Self {
        Self {
            idx: 0,
            fc: egui::Color32::WHITE,
            bc: egui::Color32::BLACK,
            open: false,
            state: ColorEditerState::RGB,
        }
    }
}

pub struct ColorEditer<'c> {
    color: &'c mut egui::Color32,
    open: &'c mut bool,
    state: &'c mut ColorEditerState,
}

impl<'c> ColorEditer<'c> {
    pub fn new(pen: &'c mut PencilState) -> Self {
        Self {
            color: &mut pen.fc,
            open: &mut pen.open,
            state: &mut pen.state,
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
impl egui::Widget for ColorEditer<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let res = ui.toggle_value(self.open, "编辑");
        egui::Window::new("编辑颜色")
            .collapsible(false)
            .resizable(false)
            .default_pos(res.rect.center())
            .open(self.open)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
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

                    if ui
                        .selectable_label(*self.state == ColorEditerState::GRAY, "GRAY")
                        .clicked()
                    {
                        *self.state = ColorEditerState::GRAY;
                    }
                });
                let (r, g, b, _) = self.color.to_tuple();
                match *self.state {
                    ColorEditerState::GRAY => {
                        let mut gray = rgb_to_gray(r, g, b);
                        ui.horizontal(|ui| {
                            ui.label("V");
                            ui.add(
                                egui::DragValue::new(&mut gray)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 255.0))
                                    .fixed_decimals(0),
                            );
                        });
                        let (r, g, b) = gray_to_rgb(gray);
                        *self.color = egui::Color32::from_rgb(r, g, b);
                    }
                    ColorEditerState::RGB => {
                        let mut r = r as f32;
                        let mut g = g as f32;
                        let mut b = b as f32;
                        ui.horizontal(|ui| {
                            ui.label("R");
                            ui.add(
                                egui::DragValue::new(&mut r)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 255.0)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("G");
                            ui.add(
                                egui::DragValue::new(&mut g)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 255.0)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("B");
                            ui.add(
                                egui::DragValue::new(&mut b)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 255.0)),
                            );
                        });
                        *self.color = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
                    }
                    ColorEditerState::HSV => {
                        let (mut h, mut s, mut v) = rgb_to_hsv(r, g, b);
                        ui.horizontal(|ui| {
                            ui.label("H");
                            ui.add(
                                egui::DragValue::new(&mut h)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 360.0))
                                    .fixed_decimals(0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("S");
                            ui.add(
                                egui::DragValue::new(&mut s)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 100.0))
                                    .fixed_decimals(0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("V");
                            ui.add(
                                egui::DragValue::new(&mut v)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 100.0))
                                    .fixed_decimals(0),
                            );
                        });
                        let (r, g, b) = hsv_to_rgb(h, s, v);
                        *self.color = egui::Color32::from_rgb(r, g, b);
                    }
                    ColorEditerState::HSL => {
                        let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);
                        ui.horizontal(|ui| {
                            ui.label("H");
                            ui.add(
                                egui::DragValue::new(&mut h)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 360.0))
                                    .fixed_decimals(0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("S");
                            ui.add(
                                egui::DragValue::new(&mut s)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 100.0))
                                    .fixed_decimals(0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("L");
                            ui.add(
                                egui::DragValue::new(&mut l)
                                    .clamp_range(core::ops::RangeInclusive::new(0.0, 100.0))
                                    .fixed_decimals(0),
                            );
                        });
                        let (r, g, b) = hsl_to_rgb(h, s, l);
                        *self.color = egui::Color32::from_rgb(r, g, b);
                    }
                }
            });
        res
    }
}
