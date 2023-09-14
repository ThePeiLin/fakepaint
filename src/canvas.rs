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
    LeftButtom,
    ButtomMiddle,
    RightButtom,
}
pub struct CanvasSizeEditWindow {
    open: bool,
    origin_width: usize,
    origin_height: usize,
    width: usize,
    height: usize,
    direct: Direction,
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
        }
    }
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
                egui::Grid::new("canvas-size")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t!("width"));
                        ui.add(
                            egui::DragValue::new(&mut self.width)
                                .clamp_range(core::ops::RangeInclusive::new(1, std::usize::MAX)),
                        );
                        ui.end_row();
                        ui.label(t!("height"));
                        ui.add(
                            egui::DragValue::new(&mut self.height)
                                .clamp_range(core::ops::RangeInclusive::new(1, std::usize::MAX)),
                        );
                    });
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

                        ui.selectable_value(&mut self.direct, Direction::LeftButtom, "⭩");
                        ui.selectable_value(&mut self.direct, Direction::ButtomMiddle, "⭣");
                        ui.selectable_value(&mut self.direct, Direction::RightButtom, "⭨");
                    });
                if ui.button("Ok").clicked() {
                    if self.origin_width != self.width || self.origin_height != self.height {
                        cmd = Some(Command::ChangeCanvasSize {
                            width: self.width,
                            height: self.height,
                            direct: self.direct,
                        });
                    }
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

    pub fn change_canvas_size(&mut self, width: usize, height: usize, direct: Direction) {
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
                copy_start_x = compute_start_middle_pos(width, self.width);
                copy_start_y = 0;
                copy_to_x = compute_to_middle_pos(width, self.width);
                copy_to_y = 0;
            }

            Direction::RightTop => {
                copy_start_x = compute_start_right_pos(width, self.width);
                copy_start_y = 0;
                copy_to_x = compute_to_right_pos(width, self.width);
                copy_to_y = 0;
            }
            Direction::Left => {
                copy_start_x = 0;
                copy_start_y = compute_start_middle_pos(height, self.height);
                copy_to_x = 0;
                copy_to_y = compute_to_middle_pos(height, self.height);
            }

            Direction::Center => {
                copy_start_x = compute_start_middle_pos(width, self.width);
                copy_start_y = compute_start_middle_pos(height, self.height);
                copy_to_x = compute_to_middle_pos(width, self.width);
                copy_to_y = compute_to_middle_pos(height, self.height);
            }

            Direction::Right => {
                copy_start_x = compute_start_right_pos(width, self.width);
                copy_start_y = compute_start_middle_pos(height, self.height);
                copy_to_x = compute_to_right_pos(width, self.width);
                copy_to_y = compute_to_middle_pos(height, self.height);
            }

            Direction::LeftButtom => {
                copy_start_x = 0;
                copy_start_y = compute_start_right_pos(width, self.height);
                copy_to_x = 0;
                copy_to_y = compute_to_right_pos(width, self.height);
            }

            Direction::ButtomMiddle => {
                copy_start_x = compute_start_middle_pos(width, self.width);
                copy_start_y = compute_start_right_pos(width, self.height);
                copy_to_x = compute_to_middle_pos(width, self.width);
                copy_to_y = compute_to_right_pos(width, self.height);
            }

            Direction::RightButtom => {
                copy_start_x = compute_start_right_pos(width, self.width);
                copy_start_y = compute_start_right_pos(width, self.height);
                copy_to_x = compute_to_right_pos(width, self.width);
                copy_to_y = compute_to_right_pos(width, self.height);
            }
        }

        let mut new_canvas = Self::with_size(width, height);

        for y in 0..std::cmp::min(height, self.height) {
            for x in 0..std::cmp::min(width, self.width) {
                *new_canvas.get_cell_mut(copy_to_x + x, copy_to_y + y) =
                    *self.get_cell(copy_start_x + x, copy_start_y + y);
            }
        }
        *self = new_canvas;
    }
}
