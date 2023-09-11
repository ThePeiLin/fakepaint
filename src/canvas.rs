use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tile::TileSet;

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
}
