use std::{borrow::Cow, default, io::{self, Bytes, Read}, thread, time::Duration};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, text, widgets::{self, Block, Borders, Paragraph, Widget, Wrap}, Terminal
};
use crossterm::{self, event::{KeyEvent, KeyEventKind, KeyEventState}};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct TextStore {
    text: String,
    cursor_index: usize,
}

impl TextStore {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_index: 0
        }
    }
    pub fn push(&mut self, c: char) {
        self.text.insert(self.cursor_index, c);
        self.cursor_index += 1;
    }
    pub fn pop(&mut self) {
        if self.cursor_index == 0 {
            return;
        }
        self.text.remove(self.cursor_index - 1);
        self.cursor_index -= 1;
    }
    pub fn extend<T: Iterator<Item = char>>(&mut self, cs: T) {
        cs.for_each(|c| self.push(c));
    }
    pub fn render(&self) -> text::Text {
        Cow::from(&self.text).into()
    }
    pub fn cursor_right(&mut self) {
        let len: usize = self.text.chars().count();
        self.cursor_index += 1;
        self.cursor_index = self.cursor_index.min(len);
    }
    pub fn cursor_left(&mut self) {
        self.cursor_index = self.cursor_index.saturating_sub(1);

    }
    pub fn cursor_up(&mut self, width: u16) {
        let line_start_min = self.cursor_index.saturating_sub(width as usize);
        let line_start = self.text[line_start_min..self.cursor_index].chars()
            .enumerate()
            .filter(|&(i,c)| c == '\n')
            .last()
            .unwrap_or((0, '\n')).0 + line_start_min;
        self.cursor_index = line_start.saturating_sub(1)
    }
    pub fn cursor_down(&mut self, width: u16) {
        let len: usize = self.text.chars().count();
        let line_end_max = (self.cursor_index + (width as usize)).min(len);
        let line_end = self.text[self.cursor_index..line_end_max].chars()
            .enumerate()
            .filter(|&(_,c)| c == '\n')
            .next()
            .unwrap_or((0, '\n')).0 + line_end_max;
        self.cursor_index = (line_end_max + 1).min(len)
    }
    pub fn cursor_position(&self, width: u16) -> (u16, u16) {
        (0..self.cursor_index + 1).into_iter().zip(self.text.chars())
            .fold((0,0), |(w,h), (_,c)| {
                match c {
                    '\n' => (0,h+1),
                    _ => match w {
                        n if n+1 == width => (0,h+1),
                        _ => (w+1, h)
                    }
                }
            })
    }
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut input_buffer = TextStore::new();
    let mut last_width = 0u16;
    loop {
        // input_buffer.clear();
        let input = crossterm::event::read().unwrap();
        match input {
            Event::Key(KeyEvent {code, kind: KeyEventKind::Press, ..}) => {
                match code {
                    KeyCode::Backspace => {
                        input_buffer.pop()
                    }
                    KeyCode::Enter => {
                        input_buffer.push('\n')
                    }
                    KeyCode::Tab => {
                        input_buffer.push('\t')
                    }
                    KeyCode::Right => {
                        input_buffer.cursor_right()
                    }
                    KeyCode::Left => {
                        input_buffer.cursor_left()
                    }
                    KeyCode::Up => {
                        input_buffer.cursor_up(last_width)
                    }
                    KeyCode::Down => {
                        input_buffer.cursor_down(last_width)
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char(c) => input_buffer.push(c),
                    _ => ()
                }
            }
            _ => ()
        }
        terminal.draw(|f| {
            let size = f.size();
            last_width = size.width - 2;
            // let w = text.width() as u16;
            // let (x,y) = (w%(size.width-1), w/(size.width-1));
            let block = Paragraph::new(
                input_buffer.render()
            ).block(
                Block::default()
                    .borders(Borders::all())
                    .title("text")
            ).wrap(Wrap {trim: false});
            f.render_widget(block, size);
            let (x,y) = input_buffer.cursor_position(last_width);
            f.set_cursor(x+1, y+1)
        })?;

        // thread::sleep(Duration::from_millis(10));
    }

    


    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}