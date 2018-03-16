// A grid looks like this:
// ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
// │312← 2 │   │   │   │   │   │   │   │   │   │   │   │
// ├─↓─┼─↑─┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤
// │   →   │   │   │   │   │   │   │   │   │   │   │   │
// ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤
// │   │   │   │   │   │   │   │   │   │   │   │   │   │
// ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤
// │   │   │   │   │   │   │ 2 │ 15│123│   │   │   │   │
// ├───┼───┼───┼───┼───┼───┼───┏━━━┓───┼───┼───┼───┼───┤
// │   │   │   │   │   │   │ 4 ┃228┃ 1 │   │   │   │   │
// ├───┼───┼───┼───┼───┼───┼───┗━━━┛───┼───╔═══╗───┼───┤
// │   │   │   │   │   │   │   │⚔92│ 1 │ 1 ║♔25║   │   │
// ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───╚═══╝───┼───┤
// │   │   │   │   │   │   │   │⛰⛰ │   │   │   │   │   │
// └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
use termion::cursor;
use termion::raw::RawTerminal;
use std::io::{self, Write};

const CELL_WIDTH: u8 = 4;
const CELL_HEIGHT: u8 = 2;
const GRID_WIDTH: u8 = 20;
const GRID_HEIGHT: u8 = 20;
const MOUNTAIN: &str = "⛰";
const KING: &str = "♔";
const SWORD: &str = "⚔";

struct CellBorder {
    vertical: &'static str,
    horizontal: &'static str,
    top_left_corner: &'static str,
    top_right_corner: &'static str,
    bottom_left_corner: &'static str,
    bottom_right_corner: &'static str,
}

struct ComplexBorders {
    horizontal: &'static str,
    vertical: &'static str,

    top_left_corner: &'static str,
    top_right_corner: &'static str,

    bottom_left_corner: &'static str,
    bottom_right_corner: &'static str,

    bottom_corner: &'static str,
    top_corner: &'static str,
    left_corner: &'static str,
    right_corner: &'static str,

    corner: &'static str,
}

#[allow(non_snake_case)]
mod THIN_BORDER {
    pub const HORIZONTAL: &str = "───";
    pub const VERTICAL: &str = "│";
    pub const TOP_LEFT_CORNER: &str = "┌";
    pub const TOP_RIGHT_CORNER: &str = "┐";
    pub const BOTTOM_LEFT_CORNER: &str = "└";
    pub const BOTTOM_RIGHT_CORNER: &str = "┘";
    pub const BOTTOM_CORNER: &str = "┴";
    pub const TOP_CORNER: &str = "┬";
    pub const LEFT_CORNER: &str = "├";
    pub const RIGHT_CORNER: &str = "┤";
    pub const CORNER: &str = "┼";
}

const BOLD_BORDER: CellBorder = CellBorder {
    horizontal: "━━━",
    vertical: "┃",
    top_left_corner: "┏",
    top_right_corner: "┓",
    bottom_left_corner: "┗",
    bottom_right_corner: "┛",
};

const DOUBLE_BORDER: CellBorder = CellBorder {
    horizontal: "═",
    vertical: "║",
    top_left_corner: "╔",
    top_right_corner: "╗",
    bottom_left_corner: "╚",
    bottom_right_corner: "╝",
};

enum CellState {
    Active,
    Visible,
    Invisible,
}

enum CellType {
    General,
    Mountain,
    Fortress,
    Normal,
}

struct Cell {
    units: Option<u16>,
    // color: Option<?????????>,
    state: CellState,
    celltype: CellType,
    x: u8,
    y: u8,
}

impl Cell {
    fn new(row_number: u8, cell_number: u8) -> Self {
        Cell {
            units: None,
            // color: None,
            state: CellState::Invisible,
            celltype: CellType::Normal,
            x: cell_number,
            y: row_number,
        }
    }

    fn draw_units(&self, stdout: &mut RawTerminal<io::Stdout>) -> io::Result<()> {
        let (x, y) = self.coordinates();
        if let Some(nb) = self.units {
            match nb {
                //nb @ 0..=9 => write!(stdout, "{} {}", cursor::Goto(x + 1, y + 1), nb),
                //nb @ 10..=999 => write!(stdout, "{}{}", cursor::Goto(x + 1, y + 1), nb),
                nb @ 0...9 => write!(stdout, "{} {}", cursor::Goto(x + 1, y + 1), nb),
                nb @ 10...999 => write!(stdout, "{}{}", cursor::Goto(x + 1, y + 1), nb),
                _ => write!(stdout, "{}max", cursor::Goto(x + 1, y + 1)),
            }
        } else {
            Ok(())
        }
    }

    fn draw(&self, stdout: &mut RawTerminal<io::Stdout>) {
        let (x, y) = self.coordinates();
        let border = self.get_cell_border();
        write!(
            stdout,
            "{}{}{}{}",
            cursor::Goto(x, y),
            border.top_left_corner,
            border.horizontal,
            border.top_right_corner
        ).unwrap();
        write!(
            stdout,
            "{}{}   {}",
            cursor::Goto(x, y + 1),
            border.vertical,
            border.vertical
        ).unwrap();
        write!(
            stdout,
            "{}{}{}{}{}",
            cursor::Goto(x, y + 2),
            border.bottom_left_corner,
            border.horizontal,
            border.bottom_right_corner,
            cursor::Goto(1, y + 3)
        ).unwrap();
        stdout.flush().unwrap();
    }

    fn get_cell_border(&self) -> CellBorder {
        match (&self.state, &self.celltype) {
            (&CellState::Active, _) => BOLD_BORDER,
            (_, &CellType::General) => DOUBLE_BORDER,
            _ => self.get_thin_cell_border(),
        }
    }

    fn coordinates(&self) -> (u16, u16) {
        (
            u16::from(self.x * CELL_WIDTH) + 1,
            u16::from(self.y * CELL_HEIGHT) + 1,
        )
    }

    fn get_thin_cell_border(&self) -> CellBorder {
        use self::THIN_BORDER::*;
        // The default boxes looks like this:
        //
        // ┼───┼
        // │   │
        // ┼───┼
        //
        let mut border = CellBorder {
            horizontal: HORIZONTAL,
            vertical: VERTICAL,

            top_left_corner: CORNER,
            top_right_corner: CORNER,

            bottom_left_corner: CORNER,
            bottom_right_corner: CORNER,
        };
        // we overwrite the corners for edge cases for the regular cells
        // (the bold and doubled cells don't change)
        // ┌───> x
        // │
        // v y
        if self.x == 0 {
            if self.y == 0 {
                // ┌───┬
                // │   │
                // ├───┼
                border.top_left_corner = TOP_LEFT_CORNER;
                border.bottom_left_corner = LEFT_CORNER;
                border.top_right_corner = TOP_CORNER;
            } else if self.y == (GRID_HEIGHT - 1) {
                // ├───┼
                // │   │
                // └───┴
                border.top_left_corner = LEFT_CORNER;
                border.bottom_left_corner = BOTTOM_LEFT_CORNER;
                border.bottom_right_corner = BOTTOM_CORNER;
            } else {
                // ├───┼
                // │   │
                // ├───┼
                border.top_left_corner = LEFT_CORNER;
                border.bottom_left_corner = LEFT_CORNER;
            }
        } else if self.x == (GRID_WIDTH - 1) {
            if self.y == 0 {
                // ┬───┐
                // │   │
                // ┼───┤
                border.top_left_corner = TOP_CORNER;
                border.top_right_corner = TOP_RIGHT_CORNER;
                border.bottom_right_corner = RIGHT_CORNER;
            } else if self.y == (GRID_HEIGHT - 1) {
                // ┼───┤
                // │   │
                // ┴───┘
                border.top_right_corner = RIGHT_CORNER;
                border.bottom_right_corner = BOTTOM_RIGHT_CORNER;
                border.bottom_left_corner = BOTTOM_CORNER;
            } else {
                // ┼───┤
                // │   │
                // ┼───┤
                border.top_right_corner = RIGHT_CORNER;
                border.bottom_right_corner = RIGHT_CORNER;
            }
        } else {
            if self.y == 0 {
                // ┬───┬
                // │   │
                // ┼───┼
                border.top_left_corner = TOP_CORNER;
                border.top_right_corner = TOP_CORNER;
            } else if self.y == (GRID_HEIGHT - 1) {
                // ┼───┼
                // │   │
                // ┴───┴
                border.bottom_left_corner = BOTTOM_CORNER;
                border.bottom_right_corner = BOTTOM_CORNER;
            }
        }
        border
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Coordinates {
    pub x: usize,
    pub y: usize,
}

impl Coordinates {
    pub fn new(x: usize, y: usize) -> Self {
        Coordinates { x, y }
    }
}

impl From<(u16, u16)> for Coordinates {
    fn from(coordinates: (u16, u16)) -> Self {
        Coordinates {
            x: coordinates.0 as usize,
            y: coordinates.1 as usize,
        }
    }
}
impl From<(usize, usize)> for Coordinates {
    fn from(coordinates: (usize, usize)) -> Self {
        Coordinates::new(coordinates.0, coordinates.1)
    }
}

struct Row {
    cells: Vec<Cell>,
}

impl Row {
    fn new(size: u8, row_number: u8) -> Self {
        let mut cells = Vec::with_capacity(size as usize);
        for cell_number in 0..size {
            cells.push(Cell::new(row_number, cell_number));
        }
        Row { cells }
    }

    fn draw(&self, stdout: &mut RawTerminal<io::Stdout>) {
        for cell in &self.cells {
            cell.draw(stdout)
        }
    }
}

pub struct Grid {
    rows: Vec<Row>,
}

type Terminal = RawTerminal<io::Stdout>;

impl Grid {
    pub fn new() -> Self {
        let mut rows = Vec::with_capacity(GRID_HEIGHT as usize);
        for row_number in 0..GRID_HEIGHT {
            rows.push(Row::new(GRID_WIDTH, row_number));
        }
        Grid { rows }
    }

    pub fn draw(&self, mut stdout: &mut Terminal) {
        for row in &self.rows {
            row.draw(&mut stdout);
        }
    }
}
