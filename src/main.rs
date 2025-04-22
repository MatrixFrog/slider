use std::{array, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::{Rng, rng};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};
use ratatui::{DefaultTerminal, Frame};

const TILE_WIDTH: u16 = 6;
const TILE_HEIGHT: u16 = 3;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}

struct App {
    grid: [[Option<u8>; 4]; 4],
    exit: bool,
}

impl App {
    fn new() -> Self {
        App {
            grid: Self::new_grid(),
            exit: false,
        }
    }

    /// Create a new randomly shuffled grid.
    fn new_grid() -> [[Option<u8>; 4]; 4] {
        let mut numbers: [u8; 15] = array::from_fn(|i| (i + 1) as u8);

        // If you just shuffle the array, there's a 50% chance the puzzle is unsolvable.
        // Instead, do an even number of exchanges. According to
        // https://en.wikipedia.org/wiki/15_puzzle#Solvability this should produce a
        // solvable arrangement. For our even number, use 50 which should be high enough.
        for _ in 0..50 {
            let a = rng().random_range(0..15);
            let b = rng().random_range(0..15);
            numbers.swap(a, b);
        }

        let options: [Option<u8>; 16] = array::from_fn(|n| match n {
            15 => None,
            n => Some(numbers[n]),
        });

        [
            <[Option<u8>; 4]>::try_from(&options[0..4]).unwrap(),
            <[Option<u8>; 4]>::try_from(&options[4..8]).unwrap(),
            <[Option<u8>; 4]>::try_from(&options[8..12]).unwrap(),
            <[Option<u8>; 4]>::try_from(&options[12..16]).unwrap(),
        ]
    }

    /// Replace the grid with a new randomly shuffled grid.
    fn shuffle(&mut self) {
        self.grid = Self::new_grid();
    }

    /// Check if the puzzle is in a winning state.
    fn is_win(&self) -> bool {
        let arr = <[Option<u8>; 16]>::try_from(self.grid.concat()).unwrap();
        for i in 0..15 {
            if arr[i] != Some(i as u8 + 1) {
                return false;
            }
        }
        true
    }

    /// Make a move if possible. If the given direction doesn't work, do nothing.
    fn make_move(&mut self, (x, y): (i8, i8)) {
        let (blank_x, blank_y) = self.find_blank();
        let (tile_x, tile_y) = (blank_x + x, blank_y + y);
        if tile_x < 0 || tile_x >= 4 || tile_y < 0 || tile_y >= 4 {
            // Illegal move; just ignore it.
            return;
        }
        let tile = self.grid[tile_y as usize][tile_x as usize];
        self.grid[blank_y as usize][blank_x as usize] = tile;
        self.grid[tile_y as usize][tile_x as usize] = None;
    }

    /// Returns the location of the blank square.
    fn find_blank(&self) -> (i8, i8) {
        for x in 0..4 {
            for y in 0..4 {
                if self.grid[y][x].is_none() {
                    return (x as i8, y as i8);
                }
            }
        }
        unreachable!("There will always be a None in the grid somewhere.");
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_input(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(event) => match event.kind {
                KeyEventKind::Press => match event.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Char('r') => {
                        self.shuffle();
                    }
                    KeyCode::Up | KeyCode::Char('w') => {
                        self.make_move((0, 1));
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        self.make_move((0, -1));
                    }
                    KeyCode::Left | KeyCode::Char('a') => {
                        self.make_move((1, 0));
                    }
                    KeyCode::Right | KeyCode::Char('d') => {
                        self.make_move((-1, 0));
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Percentage(100),
        ]);
        let [title_area, instructions_area, main_area] = vertical_layout.areas(area);

        Line::from("Sliding Puzzle").bold().render(title_area, buf);
        Line::from("        Instructions: Arrows or WASD to move. R to restart. Q to quit.")
            .render(instructions_area, buf);

        let puzzle_area = Rect {
            x: main_area.x + 6,
            y: main_area.y + 2,
            width: TILE_WIDTH * 4 + 6,
            height: TILE_HEIGHT * 4 + 2,
        };

        let puzzle_border_color = if self.is_win() {
            Color::Green
        } else {
            Color::Red
        };

        let puzzle_block = Block::bordered()
            .border_style(Style::default().fg(puzzle_border_color))
            .border_set(border::THICK);
        puzzle_block.render(puzzle_area, buf);

        let mut area = Rect {
            x: puzzle_area.x + 3,
            y: puzzle_area.y + 1,
            width: TILE_WIDTH,
            height: TILE_HEIGHT,
        };
        for row in self.grid {
            for number in row {
                match number {
                    Some(n) => {
                        let color = if n % 2 == 0 { Color::Gray } else { Color::Blue };
                        let block = Block::bordered().style(Style::default().fg(color));
                        let text_area = block.inner(area);
                        block.render(area, buf);
                        Line::from(format!(" {:02}", n)).render(text_area, buf);
                    }
                    None => {}
                }

                area.x += TILE_WIDTH;
            }
            area.x = puzzle_area.x + 3;
            area.y += TILE_HEIGHT;
        }
    }
}
