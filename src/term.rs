use console::{style, Term};

use crate::tetris::CellType;

pub struct TetrisFieldTerm {
    field: Vec<Vec<CellType>>,
    x: usize,
    y: usize,
}

impl TetrisFieldTerm {
    pub fn new(x: usize, y: usize) -> TetrisFieldTerm {
        TetrisFieldTerm {
            field: Vec::new(),
            x,
            y,
        }
    }

    pub fn refresh(&self, term: &Term) {
        draw_field_on_term(term, &self.field, self.x, self.y);
    }

    pub fn update(&mut self, term: &Term, field: &Vec<Vec<CellType>>) {
        if field != &self.field {
            self.field = field.clone();
            self.refresh(term);
        }
    }
}

fn draw_field_on_term(term: &Term, field: &Vec<Vec<CellType>>, x: usize, y: usize) {
    for (i, row) in field.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            term.move_cursor_to(x + j * 2, y + i).unwrap();
            let s = match cell {
                CellType::Empty => "  ",
                CellType::Blasted => "@@",
                _ => "[]",
            };
            let s = match cell {
                CellType::Empty => style(s).dim().on_black(),
                CellType::Blasted => style(s).red().on_black(),
                CellType::I => style(s).white().on_cyan(),
                CellType::J => style(s).white().on_blue(),
                CellType::L => style(s).white().on_yellow(),
                CellType::O => style(s).white().on_green(),
                CellType::S => style(s).white().on_magenta(),
                CellType::T => style(s).white().on_red(),
                CellType::Z => style(s).white().on_yellow(),
            };
            let s = format!("{}", s);
            term.write_str(&s).unwrap();
        }
    }
}
