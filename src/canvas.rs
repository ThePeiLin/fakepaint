use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tile::TileSet;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    LeftTop,
    TopMiddle,
    RightTop,
    Left,
    Center,
    Right,
    LeftBottom,
    BottomMiddle,
    RightBottom,
}
pub struct CanvasSizeEditWindow {
    open: bool,
    origin_width: usize,
    origin_height: usize,
    width: usize,
    height: usize,
    direct: Direction,
    start_x: usize,
    start_y: usize,
    to_x: usize,
    to_y: usize,
}

impl Default for CanvasSizeEditWindow {
    fn default() -> Self {
        Self {
            open: false,
            width: 16,
            height: 16,
            origin_width: 16,
            origin_height: 16,
            direct: Direction::Center,
            start_x: 0,
            start_y: 0,
            to_x: 0,
            to_y: 0,
        }
    }
}

fn compute_start_to_xy(
    origin_width: usize,
    origin_height: usize,
    target_width: usize,
    target_height: usize,
    direct: Direction,
) -> (usize, usize, usize, usize) {
    let copy_start_x: usize;
    let copy_start_y: usize;

    let copy_to_x: usize;
    let copy_to_y: usize;

    fn compute_start_middle_pos(target_width: usize, origin_width: usize) -> usize {
        if target_width < origin_width {
            (origin_width - target_width) / 2
        } else {
            0
        }
    }

    fn compute_start_right_pos(target_width: usize, origin_width: usize) -> usize {
        if target_width < origin_width {
            origin_width - target_width
        } else {
            0
        }
    }

    fn compute_to_middle_pos(target_width: usize, origin_width: usize) -> usize {
        if target_width < origin_width {
            0
        } else {
            (target_width - origin_width) / 2
        }
    }

    fn compute_to_right_pos(target_width: usize, origin_width: usize) -> usize {
        if target_width < origin_width {
            0
        } else {
            target_width - origin_width
        }
    }

    match direct {
        Direction::LeftTop => {
            copy_start_x = 0;
            copy_start_y = 0;
            copy_to_x = 0;
            copy_to_y = 0;
        }
        Direction::TopMiddle => {
            copy_start_x = compute_start_middle_pos(target_width, origin_width);
            copy_start_y = 0;
            copy_to_x = compute_to_middle_pos(target_width, origin_width);
            copy_to_y = 0;
        }

        Direction::RightTop => {
            copy_start_x = compute_start_right_pos(target_width, origin_width);
            copy_start_y = 0;
            copy_to_x = compute_to_right_pos(target_width, origin_width);
            copy_to_y = 0;
        }
        Direction::Left => {
            copy_start_x = 0;
            copy_start_y = compute_start_middle_pos(target_height, origin_height);
            copy_to_x = 0;
            copy_to_y = compute_to_middle_pos(target_height, origin_height);
        }

        Direction::Center => {
            copy_start_x = compute_start_middle_pos(target_width, origin_width);
            copy_start_y = compute_start_middle_pos(target_height, origin_height);
            copy_to_x = compute_to_middle_pos(target_width, origin_width);
            copy_to_y = compute_to_middle_pos(target_height, origin_height);
        }

        Direction::Right => {
            copy_start_x = compute_start_right_pos(target_width, origin_width);
            copy_start_y = compute_start_middle_pos(target_height, origin_height);
            copy_to_x = compute_to_right_pos(target_width, origin_width);
            copy_to_y = compute_to_middle_pos(target_height, origin_height);
        }

        Direction::LeftBottom => {
            copy_start_x = 0;
            copy_start_y = compute_start_right_pos(target_height, origin_height);
            copy_to_x = 0;
            copy_to_y = compute_to_right_pos(target_height, origin_height);
        }

        Direction::BottomMiddle => {
            copy_start_x = compute_start_middle_pos(target_width, origin_width);
            copy_start_y = compute_start_right_pos(target_height, origin_height);
            copy_to_x = compute_to_middle_pos(target_width, origin_width);
            copy_to_y = compute_to_right_pos(target_height, origin_height);
        }

        Direction::RightBottom => {
            copy_start_x = compute_start_right_pos(target_width, origin_width);
            copy_start_y = compute_start_right_pos(target_height, origin_height);
            copy_to_x = compute_to_right_pos(target_width, origin_width);
            copy_to_y = compute_to_right_pos(target_height, origin_height);
        }
    };
    (copy_start_x, copy_start_y, copy_to_x, copy_to_y)
}

use crate::undo::Command;
impl CanvasSizeEditWindow {
    pub fn open(&mut self, width: usize, height: usize) {
        self.open = true;
        self.width = width;
        self.height = height;
        self.origin_width = width;
        self.origin_height = height;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<Command> {
        use rust_i18n::t;

        let mut cmd: Option<Command> = None;

        let mut close_windows = false;
        egui::Window::new(t!("canvas_size"))
            .open(&mut self.open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(t!("size"));

                let width_before_drag = self.width;
                let height_before_drag = self.height;
                egui::Grid::new("canvas-size")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
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
                    });

                let origin_direct = self.direct;

                egui::Grid::new("canvas-size-direct")
                    .striped(true)
                    .spacing(egui::Vec2::ZERO)
                    .num_columns(3)
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        ui.selectable_value(&mut self.direct, Direction::LeftTop, "⭦");
                        ui.selectable_value(&mut self.direct, Direction::TopMiddle, "⭡");
                        ui.selectable_value(&mut self.direct, Direction::RightTop, "⭧");
                        ui.end_row();

                        ui.selectable_value(&mut self.direct, Direction::Left, "⭠");
                        ui.selectable_value(&mut self.direct, Direction::Center, "⭘");
                        ui.selectable_value(&mut self.direct, Direction::Right, "⭢");
                        ui.end_row();

                        ui.selectable_value(&mut self.direct, Direction::LeftBottom, "⭩");
                        ui.selectable_value(&mut self.direct, Direction::BottomMiddle, "⭣");
                        ui.selectable_value(&mut self.direct, Direction::RightBottom, "⭨");
                    });

                if origin_direct != self.direct
                    || width_before_drag != self.width
                    || height_before_drag != self.height
                {
                    let (start_x, start_y, to_x, to_y) = compute_start_to_xy(
                        self.origin_width,
                        self.origin_height,
                        self.width,
                        self.height,
                        self.direct,
                    );
                    self.start_x = start_x;
                    self.start_y = start_y;
                    self.to_x = to_x;
                    self.to_y = to_y;
                }

                ui.separator();
                let start_x = self.start_x as isize;
                let start_y = self.start_y as isize;
                let to_x = self.to_x as isize;
                let to_y = self.to_y as isize;

                let border_left_before_drag = to_x - start_x;
                let border_top_before_drag = to_y - start_y;
                let border_right_before_drag =
                    self.width as isize - (border_left_before_drag + self.origin_width as isize);
                let border_bottom_before_drag =
                    self.height as isize - (border_top_before_drag + self.origin_height as isize);

                let mut border_left = border_left_before_drag;
                let mut border_top = border_top_before_drag;
                let mut border_right = border_right_before_drag;
                let mut border_bottom = border_bottom_before_drag;

                ui.heading(t!("border"));
                egui::Grid::new("canvas-size-border")
                    .striped(true)
                    .num_columns(4)
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        ui.label(t!("left"));
                        ui.add(egui::DragValue::new(&mut border_left));
                        ui.label(t!("top"));
                        ui.add(egui::DragValue::new(&mut border_top));
                        ui.end_row();

                        ui.label(t!("right"));
                        ui.add(egui::DragValue::new(&mut border_right));
                        ui.label(t!("bottom"));
                        ui.add(egui::DragValue::new(&mut border_bottom));
                    });

                if border_left_before_drag != border_left
                    || border_top_before_drag != border_top
                    || border_right_before_drag != border_right
                    || border_bottom_before_drag != border_bottom
                {
                    self.start_x = if border_left > 0 {
                        0
                    } else {
                        (-border_left) as usize
                    };
                    self.start_y = if border_top > 0 {
                        0
                    } else {
                        (-border_top) as usize
                    };

                    self.to_x = if border_left > 0 {
                        border_left as usize
                    } else {
                        0
                    };

                    self.to_y = if border_top > 0 {
                        border_top as usize
                    } else {
                        0
                    };

                    let width = border_left + border_right + self.origin_width as isize;
                    let height = border_top + border_bottom + self.origin_height as isize;
                    self.width = if width < 0 { 1 } else { width as usize };
                    self.height = if height < 0 { 1 } else { height as usize };
                }

                if ui.button("Ok").clicked() {
                    cmd = Some(Command::ChangeCanvasSize {
                        width: self.width,
                        height: self.height,
                        start_x: self.start_x,
                        start_y: self.start_y,
                        to_x: self.to_x,
                        to_y: self.to_y,
                    });
                    self.start_x = 0;
                    self.start_y = 0;
                    self.to_x = 0;
                    self.to_y = 0;
                    close_windows = true;
                }
            });
        if close_windows {
            self.open = false;
        }
        cmd
    }
}
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TileState {
    pub idx: usize,
    #[serde(
        serialize_with = "serialize_color32",
        deserialize_with = "deserialize_color32"
    )]
    pub fc: egui::Color32,
    #[serde(
        serialize_with = "serialize_color32",
        deserialize_with = "deserialize_color32"
    )]
    pub bc: egui::Color32,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ser() -> Result<(), serde_json::error::Error> {
        let tile_state = TileState {
            idx: 0,
            fc: egui::Color32::WHITE,
            bc: egui::Color32::BLACK,
        };
        let str = serde_json::to_string(&tile_state)?;
        assert_eq!(str, r#"{"idx":0,"fc":[255,255,255],"bc":[0,0,0]}"#);
        Ok(())
    }
    #[test]
    fn test_de() -> Result<(), serde_json::error::Error> {
        let tile_state: TileState =
            serde_json::from_str(r#"{"idx":0,"fc":[255,255,255],"bc":[0,0,0]}"#)?;
        assert_eq!(
            tile_state,
            TileState {
                idx: 0,
                fc: egui::Color32::WHITE,
                bc: egui::Color32::BLACK,
            }
        );
        Ok(())
    }
}

fn serialize_color32<S>(color: &egui::Color32, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_bytes(&color.to_array()[0..3])
}

fn deserialize_color32<'de, D>(deser: D) -> Result<egui::Color32, D::Error>
where
    D: Deserializer<'de>,
{
    let a = <[u8; 3]>::deserialize(deser)?;
    Ok(egui::Color32::from_rgb(a[0], a[1], a[2]))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Option<TileState>>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::with_size(16, 16)
    }
}

impl Canvas {
    pub fn export_as_image(&self, tile: &TileSet, path: &str, scale: u32) {
        use crate::TILE_SIZE;
        use crate::TILE_SIZE_VEC2;

        use image::{GenericImageView, ImageBuffer, RgbaImage};
        let tile_image = &tile.image_data;
        let mut img: RgbaImage = ImageBuffer::new(
            self.width as u32 * TILE_SIZE as u32,
            self.height as u32 * TILE_SIZE as u32,
        );
        for y in 0..self.height {
            for x in 0..self.width {
                let cur = self.cells[x + y * self.width].clone();
                if let Some(cur_tile) = cur {
                    let uv = tile.uv(cur_tile.idx).left_top();
                    let mut tile_sub_img = tile_image
                        .view(
                            (uv.x * tile_image.width() as f32) as u32,
                            (uv.y * tile_image.height() as f32) as u32,
                            TILE_SIZE_VEC2.x as u32,
                            TILE_SIZE_VEC2.y as u32,
                        )
                        .to_image();

                    let target_x = x as u32 * TILE_SIZE_VEC2.x as u32;
                    let target_y = y as u32 * TILE_SIZE_VEC2.y as u32;

                    let target_rect = imageproc::rect::Rect::at(target_x as i32, target_y as i32)
                        .of_size(TILE_SIZE_VEC2.x as u32, TILE_SIZE_VEC2.y as u32);

                    let bc = image::Rgba(cur_tile.bc.to_array());

                    let fc = cur_tile.fc.to_normalized_gamma_f32();

                    tile_sub_img.pixels_mut().for_each(|cur| {
                        *cur = image::Rgba([
                            (cur.0[0] as f32 * fc[0]) as u8,
                            (cur.0[1] as f32 * fc[1]) as u8,
                            (cur.0[2] as f32 * fc[2]) as u8,
                            (cur.0[3] as f32 * fc[3]) as u8,
                        ]);
                    });

                    imageproc::drawing::draw_filled_rect_mut(&mut img, target_rect, bc);
                    image::imageops::overlay(
                        &mut img,
                        &tile_sub_img,
                        target_x.into(),
                        target_y.into(),
                    );
                }
            }
        }

        if scale != 1 {
            image::imageops::resize(
                &img,
                img.width() * scale,
                img.height() * scale,
                image::imageops::Nearest,
            )
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();
        } else {
            img.save_with_format(path, image::ImageFormat::Png).unwrap();
        }
    }

    pub fn with_size(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut cells = Vec::with_capacity(size);
        cells.resize(size, None);
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Option<TileState> {
        &mut self.cells[y * self.width + x]
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Option<TileState> {
        &self.cells[y * self.width + x]
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn change_canvas_size(
        &mut self,
        width: usize,
        height: usize,
        copy_start_x: usize,
        copy_start_y: usize,
        copy_to_x: usize,
        copy_to_y: usize,
    ) {
        let mut new_canvas = Self::with_size(width, height);

        for y in 0..std::cmp::min(height - copy_to_y, self.height - copy_start_y) {
            for x in 0..std::cmp::min(width - copy_to_x, self.width - copy_start_x) {
                *new_canvas.get_cell_mut(copy_to_x + x, copy_to_y + y) =
                    *self.get_cell(copy_start_x + x, copy_start_y + y);
            }
        }
        *self = new_canvas;
    }
}
