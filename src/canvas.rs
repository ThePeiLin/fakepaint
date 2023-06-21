use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub size_x: usize,
    pub size_y: usize,
    pub cells: Vec<Option<TileState>>,
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
