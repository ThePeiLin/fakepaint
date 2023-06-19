use eframe::egui;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct JsonableTileState {
    pub idx: usize,
    pub fc: (u8, u8, u8, u8),
    pub bc: (u8, u8, u8, u8),
}

#[derive(Deserialize, Serialize)]
pub struct JsonableCanvas {
    pub cells: Vec<Option<JsonableTileState>>,
    pub size_x: usize,
    pub size_y: usize,
}

use crate::canvas::Canvas;
use crate::canvas::TileState;
impl Into<Canvas> for JsonableCanvas {
    fn into(self) -> Canvas {
        let mut cells = Vec::with_capacity(self.cells.len());
        for c in &self.cells {
            if let Some(c) = c {
                let fc = c.fc;
                let bc = c.bc;
                cells.push(Some(TileState {
                    idx: c.idx,
                    fc: egui::Color32::from_rgba_premultiplied(fc.0, fc.1, fc.2, fc.3),
                    bc: egui::Color32::from_rgba_premultiplied(bc.0, bc.1, bc.2, bc.3),
                }));
            } else {
                cells.push(None);
            }
        }
        Canvas {
            cells,
            size_x: self.size_x,
            size_y: self.size_y,
        }
    }
}

impl From<&Canvas> for JsonableCanvas {
    fn from(ca: &Canvas) -> Self {
        let mut cells = Vec::with_capacity(ca.cells.len());
        for c in &ca.cells {
            if let Some(c) = c {
                cells.push(Some(JsonableTileState {
                    idx: c.idx,
                    fc: c.fc.to_tuple(),
                    bc: c.bc.to_tuple(),
                }));
            } else {
                cells.push(None);
            }
        }
        JsonableCanvas {
            cells,
            size_x: ca.size_x,
            size_y: ca.size_y,
        }
    }
}

pub fn load_canvas_from_file(path: &Path) -> Result<Canvas, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u: JsonableCanvas = serde_json::from_reader(reader)?;
    Ok(u.into())
}

pub fn write_canvas_to_file(u: &impl Serialize, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, u)?;
    Ok(())
}
