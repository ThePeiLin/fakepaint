use eframe::egui;

#[derive(Clone, PartialEq, Eq)]
pub enum PointContent {
    None,
    Tile {
        idx: usize,
        fc: egui::Color32,
        bc: egui::Color32,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub enum Command {
    Point {
        c: PointContent,
        x: usize,
        y: usize,
    },
    ChangeCanvasSize {
        width: usize,
        height: usize,
        start_x: usize,
        start_y: usize,
        to_x: usize,
        to_y: usize,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Point {
            c: PointContent::None,
            x: 0,
            y: 0,
        }
    }
}

const COMMAND_EXCUTE_GAP: usize = 16;
pub struct History {
    rendering_canvas: Option<Canvas>,
    edit_history: Vec<Command>,
    undo_history: Vec<Command>,
    last_command: usize,
}

use crate::{canvas::TileState, color_editer::PencilState, Canvas};
impl Command {
    pub fn new(
        x: usize,
        y: usize,
        pen: &PencilState,
        cur_tile: &Option<TileState>,
        need_swap: bool,
    ) -> Self {
        let tile = if need_swap {
            pen.swap_color_and_into()
        } else {
            pen.into_tile_state(cur_tile)
        };
        if let Some(TileState { idx, fc, bc }) = tile {
            Self::Point {
                c: PointContent::Tile { idx, fc, bc },
                x,
                y,
            }
        } else {
            Self::Point {
                c: PointContent::None,
                x,
                y,
            }
        }
    }
}

fn excute_painting_command_to_canvas_mut(canvas: &mut Canvas, commands: &[Command]) {
    for command in commands {
        match command.clone() {
            Command::Point { c, x, y } => {
                let target_tile = canvas.get_cell_mut(x, y);
                match c {
                    PointContent::None => *target_tile = None,
                    PointContent::Tile { idx, fc, bc } => {
                        *target_tile = Some(TileState { idx, fc, bc })
                    }
                }
            }
            Command::ChangeCanvasSize {
                width,
                height,
                start_x,
                start_y,
                to_x,
                to_y,
            } => {
                canvas.change_canvas_size(width, height, start_x, start_y, to_x, to_y);
            }
        }
    }
}

fn excute_painting_command_to_canvas(canvas: &Canvas, commands: &[Command]) -> Canvas {
    let mut canvas = canvas.clone();
    excute_painting_command_to_canvas_mut(&mut canvas, commands);
    canvas
}

impl History {
    pub fn new() -> Self {
        Self {
            edit_history: Vec::new(),
            undo_history: Vec::new(),
            last_command: 0,
            rendering_canvas: None,
        }
    }

    pub fn undo(&mut self) {
        if let Some(command) = self.edit_history.pop() {
            self.undo_history.push(command);
            let last_command = self.edit_history.len() / COMMAND_EXCUTE_GAP;
            if last_command < self.last_command {
                self.last_command = 0;
                self.rendering_canvas = None;
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(command) = self.undo_history.pop() {
            self.edit_history.push(command);
        }
    }

    pub fn clear_undo(&mut self) {
        self.undo_history.clear();
    }

    pub fn excute_on_canvas(&mut self, canvas: &Canvas) -> Canvas {
        if let None = self.rendering_canvas {
            self.rendering_canvas = Some(canvas.clone());
        }

        let last_command = self.edit_history.len() / COMMAND_EXCUTE_GAP;

        if last_command != self.last_command {
            excute_painting_command_to_canvas_mut(
                self.rendering_canvas.as_mut().unwrap(),
                &self.edit_history[self.last_command * COMMAND_EXCUTE_GAP
                    ..(self.last_command * COMMAND_EXCUTE_GAP)
                        + (last_command - self.last_command) * COMMAND_EXCUTE_GAP],
            );
            self.last_command = last_command;
        }

        let canvas = excute_painting_command_to_canvas(
            self.rendering_canvas.as_ref().unwrap(),
            &self.edit_history[self.last_command * COMMAND_EXCUTE_GAP..],
        );

        canvas
    }

    pub fn push(&mut self, command: Command) {
        let last = self.edit_history.last();
        if last == None || *last.unwrap() != command {
            self.edit_history.push(command);
            self.clear_undo();
        }
    }

    pub fn clear(&mut self) {
        self.last_command = 0;
        self.edit_history.clear();
        self.undo_history.clear();
        self.rendering_canvas = None;
    }
}
