use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use crate::canvas::Canvas;

use serde::Serialize;
pub fn load_canvas_from_file(path: &Path) -> Result<Canvas, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u: Canvas = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn write_canvas_to_file(u: &impl Serialize, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, u)?;
    Ok(())
}

use eframe::egui;

pub fn write_palette(palette: &Vec<egui::Color32>, path: &Path) -> Result<(), Box<dyn Error>> {
    fn to_serializable_palette(p: &Vec<egui::Color32>) -> Vec<(u8, u8, u8)> {
        let mut p1 = Vec::new();
        for c in p {
            let (r, g, b, _) = c.to_tuple();
            p1.push((r, g, b));
        }
        p1
    }
    let palette = to_serializable_palette(palette);
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &palette)?;
    Ok(())
}

pub fn load_palette(path: &Path) -> Result<Vec<egui::Color32>, Box<dyn Error>> {
    fn to_color_vec(vec: Vec<(u8, u8, u8)>) -> Vec<egui::Color32> {
        let mut p = Vec::with_capacity(vec.len());
        for (r, g, b) in vec {
            p.push(egui::Color32::from_rgb(r, g, b));
        }
        p
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let palette: Vec<(u8, u8, u8)> = serde_json::from_reader(reader)?;
    Ok(to_color_vec(palette))
}
