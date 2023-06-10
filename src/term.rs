use console::{style, StyledObject, Term};
use once_cell::sync::Lazy;

use crate::{tetris::CellType, Field};

pub struct TetrisTerm {
    field: Field,
    preview: Field,
    x: usize,
    y: usize,
}

impl TetrisTerm {
    pub fn new(cols: usize, rows: usize) -> TetrisTerm {
        TetrisTerm {
            field: Field::new(cols, rows),
            preview: Field::new(4, 4),
            x: cols,
            y: rows,
        }
    }

    pub fn refresh(&self, term: &Term) {
        draw_field_on_term(term, &self.field, self.x, self.y, false);
        draw_field_on_term(
            term,
            &self.preview,
            self.x + self.field.cols() * 2 + 2 + 2,
            self.y,
            true,
        );
    }

    pub fn update(&mut self, term: &Term, field: &Field, preview: &Field) {
        if field != &self.field || preview != &self.preview {
            self.field = field.clone();
            self.preview = preview.clone();
            self.refresh(term);
        }
    }
}

static BORDER_VERTICAL: Lazy<StyledObject<&str>> = Lazy::new(|| style("|").dim().on_black());
static BORDER_HORIZONTAL: Lazy<StyledObject<&str>> = Lazy::new(|| style("--").dim().on_black());
static CORNER: Lazy<StyledObject<&str>> = Lazy::new(|| style("+").dim().on_black());

fn draw_horizontal_border(term: &Term, x: usize, y: usize, width: usize) {
    term.move_cursor_to(x, y).unwrap();
    term.write_str(&format!("{}", *CORNER)).unwrap();
    for _ in 0..width {
        term.write_str(&format!("{}", *BORDER_HORIZONTAL)).unwrap();
    }
    term.write_str(&format!("{}", *CORNER)).unwrap();
}

fn draw_field_on_term(term: &Term, field: &Field, x: usize, y: usize, with_top_border: bool) {
    let mut y = y;
    if with_top_border {
        draw_horizontal_border(term, x, y, field.cols());
        y += 1;
    }

    for row in 0..field.rows() {
        term.move_cursor_to(x, y + row).unwrap();
        term.write_str(&format!("{}", *BORDER_VERTICAL)).unwrap();
        for col in 0..field.cols() {
            let cell = field.get_cell(col, row);
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
        term.write_str(&format!("{}", *BORDER_VERTICAL)).unwrap();
    }
    draw_horizontal_border(term, x, y + field.rows(), field.cols());
}
