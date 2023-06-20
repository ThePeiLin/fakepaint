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
    Ok(u.into())
}

pub fn write_canvas_to_file(u: &impl Serialize, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, u)?;
    Ok(())
}
