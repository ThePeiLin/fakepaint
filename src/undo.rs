#[derive(Clone)]
pub struct FillPos {
    c: Option<TileState>,
    x: usize,
    y: usize,
    cells: Vec<Vec<bool>>,
}

impl PartialEq for FillPos {
    fn eq(&self, other: &Self) -> bool {
        self.c == other.c && self.x == other.x && self.y == other.y
    }
}

#[derive(Clone, PartialEq)]
pub enum Command {
    Point {
        c: Option<TileState>,
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
    Fill(FillPos),
}

impl Default for Command {
    fn default() -> Self {
        Self::Point {
            c: None,
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

use crate::{
    canvas::TileState,
    color_editer::{PencilState, ToolEnum},
    Canvas,
};

fn compute_contigeous_cell(canvas: &Canvas, x: usize, y: usize) -> Vec<Vec<bool>> {
    let mut retval: Vec<Vec<bool>> = Vec::with_capacity(canvas.height);
    for _ in 0..canvas.height {
        let mut row: Vec<bool> = Vec::with_capacity(canvas.width);
        row.resize(canvas.width, false);
        retval.push(row);
    }
    let &target_tile = canvas.get_cell(x, y);
    let mut unchecked: Vec<(usize, usize)> = Vec::with_capacity(canvas.width * canvas.height);
    unchecked.push((x, y));
    let end_x = canvas.width - 1;
    let end_y = canvas.height - 1;
    while !unchecked.is_empty() {
        let (x, y) = unchecked.pop().unwrap();
        retval[y][x] = true;
        if x > 0 && !retval[y][x - 1] && target_tile == *canvas.get_cell(x - 1, y) {
            unchecked.push((x - 1, y));
        }
        if x < end_x && !retval[y][x + 1] && target_tile == *canvas.get_cell(x + 1, y) {
            unchecked.push((x + 1, y));
        }
        if y > 0 && !retval[y - 1][x] && target_tile == *canvas.get_cell(x, y - 1) {
            unchecked.push((x, y - 1));
        }
        if y < end_y && !retval[y + 1][x] && target_tile == *canvas.get_cell(x, y + 1) {
            unchecked.push((x, y + 1));
        }
    }

    retval
}

impl Command {
    pub fn new(
        x: usize,
        y: usize,
        pen: &PencilState,
        cur_tile: &Option<TileState>,
        need_swap: bool,
        canvas: &Canvas,
    ) -> Self {
        let tile = if need_swap {
            pen.swap_color_and_into()
        } else {
            pen.into_tile_state(cur_tile)
        };
        match pen.tool {
            ToolEnum::Pencil => Self::Point { c: tile, x, y },
            ToolEnum::Fill => Self::Fill(FillPos {
                c: tile,
                cells: compute_contigeous_cell(canvas, x, y),
                x,
                y,
            }),
        }
    }
}

fn excute_painting_command_to_canvas_mut(canvas: &mut Canvas, commands: &[Command]) {
    for command in commands {
        match command.clone() {
            Command::Point { c, x, y } => {
                *canvas.get_cell_mut(x, y) = c;
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
            Command::Fill(FillPos {
                c,
                cells,
                x: _,
                y: _,
            }) => {
                for (y, row) in cells.into_iter().enumerate() {
                    for (x, need_filled) in row.into_iter().enumerate() {
                        if need_filled {
                            *canvas.get_cell_mut(x, y) = c;
                        }
                    }
                }
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
