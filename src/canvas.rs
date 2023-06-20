use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Serialize, Deserialize)]
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

fn serialize_color32<S>(color: &egui::Color32, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_bytes(&color.to_array())
}

fn deserialize_color32<'de, D>(deser: D) -> Result<egui::Color32, D::Error>
where
    D: Deserializer<'de>,
{
    let a = <[u8; 4]>::deserialize(deser)?;
    Ok(egui::Color32::from_rgba_premultiplied(
        a[0], a[1], a[2], a[3],
    ))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub cells: Vec<Option<TileState>>,
    pub size_x: usize,
    pub size_y: usize,
}

impl Canvas {
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Option<TileState> {
        &mut self.cells[y * self.size_x + x]
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Option<TileState> {
        &self.cells[y * self.size_x + x]
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.size_x * self.size_y
    }
}
