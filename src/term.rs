use console::{style, StyledObject, Term};
use once_cell::sync::Lazy;

use crate::{
    tetris::{CellType, TetrisGameState},
    tetris_pair::TetrisPairState,
    Field,
};

pub struct TetrisTermDraw {
    field: Field,
    preview: Field,
    game_over: bool,
    align_right: bool,
}

impl TetrisTermDraw {
    pub fn new(cols: usize, rows: usize, align_right: bool) -> TetrisTermDraw {
        TetrisTermDraw {
            field: Field::new(cols, rows),
            preview: Field::new(4, 4),
            game_over: false,
            align_right,
        }
    }

    pub fn refresh(&self, term: &Term) {
        let (_, width) = term.size();
        let width = width as usize;
        let x_field = if self.align_right {
            width - self.field.cols() * 2 - 2
        } else {
            0
        };
        let x_preview = if self.align_right {
            x_field - 4 * 2 - 2 - 2
        } else {
            self.field.cols() * 2 + 2 + 2
        };

        draw_field_on_term(term, &self.field, x_field, 0, false);
        draw_field_on_term(term, &self.preview, x_preview, 0, true);
    }

    pub fn update(&mut self, term: &Term, state: &TetrisGameState) {
        if state.field != self.field
            || state.preview != self.preview
            || state.game_over != self.game_over
        {
            self.field = state.field.clone();
            self.preview = state.preview.clone();
            self.game_over = state.game_over;
            self.refresh(term);
        }
    }
}

pub struct TetrisPairTermDraw {
    player: TetrisTermDraw,
    opponent: TetrisTermDraw,
    term_width: usize,
}

impl TetrisPairTermDraw {
    pub fn new(cols: usize, rows: usize) -> TetrisPairTermDraw {
        let term = Term::stdout();
        let (_, width) = term.size();
        let width = width as usize;
        let player = TetrisTermDraw::new(cols, rows, false);
        let opponent = TetrisTermDraw::new(cols, rows, true);
        TetrisPairTermDraw {
            player,
            opponent,
            term_width: width,
        }
    }

    pub fn refresh(&self, term: &Term) {
        self.player.refresh(term);
        self.opponent.refresh(term);
    }

    pub fn update(&mut self, term: &Term, state: &TetrisPairState) {
        let (_, width) = term.size();
        let width = width as usize;
        if width != self.term_width {
            self.term_width = width;
            term.clear_screen().unwrap();
            self.refresh(term);
        } else {
            self.player.update(term, &state.player);
            self.opponent.update(term, &state.opponent);
        }
    }
}

impl TetrisPairTermDraw {}

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
