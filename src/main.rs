use cpal::traits::StreamTrait;
use output::output_stream;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
mod audio_stream;
mod output;
mod save_data;
use audio_stream::{AudioStream, Digits};
use save_data::SongData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).expect("no file given");

    let audio_stream = Arc::new(Mutex::new(AudioStream::from_wave_file(filename)));
    let stream = output_stream(audio_stream.clone());

    stream.play()?;

    let mut terminal = ratatui::init();
    let app_result = App {
        stream: audio_stream,
        exit: false,
        mode: Mode::Normal,
    }
    .run(&mut terminal);
    ratatui::restore();

    Ok(app_result?)
}

enum Mode {
    Normal,
    SetBookmark,
    SetSpeed,
    ProcessSpeed,
}

pub struct App {
    stream: Arc<Mutex<AudioStream>>,
    mode: Mode,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event);
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.mode {
            Mode::Normal => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('k') => self.stream.lock().unwrap().toggle_play(),
                KeyCode::Char('j') => self.stream.lock().unwrap().seek_backwards(5),
                KeyCode::Char('l') => self.stream.lock().unwrap().seek_forwards(5),
                KeyCode::Char('u') => self.stream.lock().unwrap().set_loop_start(),
                KeyCode::Char('o') => self.stream.lock().unwrap().set_loop_end(),
                KeyCode::Char('i') => self.stream.lock().unwrap().toggle_loop(),
                KeyCode::Char('1') => self.stream.lock().unwrap().seek_to_bookmark(Digits::One),
                KeyCode::Char('2') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Two),
                KeyCode::Char('3') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Three),
                KeyCode::Char('4') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Four),
                KeyCode::Char('5') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Five),
                KeyCode::Char('6') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Six),
                KeyCode::Char('7') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Seven),
                KeyCode::Char('8') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Eight),
                KeyCode::Char('9') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Nine),
                KeyCode::Char('0') => self.stream.lock().unwrap().seek_to_bookmark(Digits::Zero),
                KeyCode::Char('w') => self.stream.lock().unwrap().set_bookmark(Digits::One),
                KeyCode::Char('b') => self.mode = Mode::SetBookmark,
                KeyCode::Char('s') => self.mode = Mode::SetSpeed,
                KeyCode::Char('p') => self.mode = Mode::ProcessSpeed,
                _ => {}
            },
            Mode::SetBookmark => match key_event.code {
                KeyCode::Char('j') => self.stream.lock().unwrap().seek_backwards(5),
                KeyCode::Char('l') => self.stream.lock().unwrap().seek_forwards(5),
                KeyCode::Char('k') => self.stream.lock().unwrap().toggle_play(),
                KeyCode::Char('1') => self.stream.lock().unwrap().set_bookmark(Digits::One),
                KeyCode::Char('2') => self.stream.lock().unwrap().set_bookmark(Digits::Two),
                KeyCode::Char('3') => self.stream.lock().unwrap().set_bookmark(Digits::Three),
                KeyCode::Char('4') => self.stream.lock().unwrap().set_bookmark(Digits::Four),
                KeyCode::Char('5') => self.stream.lock().unwrap().set_bookmark(Digits::Five),
                KeyCode::Char('6') => self.stream.lock().unwrap().set_bookmark(Digits::Six),
                KeyCode::Char('7') => self.stream.lock().unwrap().set_bookmark(Digits::Seven),
                KeyCode::Char('8') => self.stream.lock().unwrap().set_bookmark(Digits::Eight),
                KeyCode::Char('9') => self.stream.lock().unwrap().set_bookmark(Digits::Nine),
                KeyCode::Char('0') => self.stream.lock().unwrap().set_bookmark(Digits::Zero),
                KeyCode::Char('b') => self.mode = Mode::Normal,
                _ => {}
            },
            Mode::SetSpeed => match key_event.code {
                KeyCode::Char('1') => {
                    if let Err(e) = self.stream.lock().unwrap().set_speed(2.0) {
                        eprintln!("Error setting speed: {}", e);
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('2') => {
                    if let Err(e) = self.stream.lock().unwrap().set_speed(1.33) {
                        eprintln!("Error setting speed: {}", e);
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('3') => {
                    if let Err(e) = self.stream.lock().unwrap().set_speed(1.0) {
                        eprintln!("Error setting speed: {}", e);
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('4') => {
                    if let Err(e) = self.stream.lock().unwrap().set_speed(0.8) {
                        eprintln!("Error setting speed: {}", e);
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('5') => {
                    if let Err(e) = self.stream.lock().unwrap().set_speed(0.67) {
                        eprintln!("Error setting speed: {}", e);
                    }
                    self.mode = Mode::Normal;
                }
                KeyCode::Char('r') => {
                    if let Err(e) = self.stream.lock().unwrap().reset_speed() {
                        eprintln!("Error resetting speed: {}", e);
                    }
                }
                KeyCode::Char('s') => self.mode = Mode::Normal,
                _ => {}
            },
            Mode::ProcessSpeed => match key_event.code {
                KeyCode::Char('1') => {
                    if let Err(e) = self.stream.lock().unwrap().process_speed_version(2.0) {
                        eprintln!("Error processing speed version: {}", e);
                    }
                }
                KeyCode::Char('2') => {
                    if let Err(e) = self.stream.lock().unwrap().process_speed_version(1.33) {
                        eprintln!("Error processing speed version: {}", e);
                    }
                }
                KeyCode::Char('3') => {
                    if let Err(e) = self.stream.lock().unwrap().process_speed_version(1.0) {
                        eprintln!("Error processing speed version: {}", e);
                    }
                }
                KeyCode::Char('4') => {
                    if let Err(e) = self.stream.lock().unwrap().process_speed_version(0.8) {
                        eprintln!("Error processing speed version: {}", e);
                    }
                }
                KeyCode::Char('5') => {
                    if let Err(e) = self.stream.lock().unwrap().process_speed_version(0.67) {
                        eprintln!("Error processing speed version: {}", e);
                    }
                }
                KeyCode::Char('p') => self.mode = Mode::Normal,
                _ => {}
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Transcriber ".bold());
        let common_instructions = Line::from(vec![
            " Toggle Pause ".into(),
            "<k>".blue().bold(),
            " Seek Backwards ".into(),
            "<j>".blue().bold(),
            " Seek Forwards ".into(),
            "<l>".blue().bold(),
            " Set Loop Start ".into(),
            "<u>".blue().bold(),
            " Set Loop End ".into(),
            "<o>".blue().bold(),
            " Toggle Looping ".into(),
            "<i>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
        ]);
        let loop_instructions = vec![
            " Set Loop Start ".into(),
            "<u>".blue().bold(),
            " Set Loop End ".into(),
            "<o>".blue().bold(),
            " Toggle Looping ".into(),
            "<i>".blue().bold(),
            " Jump to Bookmark ".into(),
            "<0-9>".blue().bold(),
            " Bookmark Mode ".into(),
            "<b>".blue().bold(),
            " Speed Mode ".into(),
            "<s>".blue().bold(),
        ];
        let bookmark_instructions = vec![
            " Set Bookmark ".into(),
            "<0-9>".blue().bold(),
            " Normal Mode ".into(),
            "<b>".blue().bold(),
        ];
        let speed_instructions = vec![
            " Set Speed ".into(),
            "<1>".blue().bold(),
            " 2.0x ".into(),
            "<2>".blue().bold(),
            " 1.33x ".into(),
            "<3>".blue().bold(),
            " 1.0x ".into(),
            "<4>".blue().bold(),
            " 0.8x ".into(),
            "<5>".blue().bold(),
            " 0.67x ".into(),
            " Reset ".into(),
            "<r>".blue().bold(),
            " Normal Mode ".into(),
            "<s>".blue().bold(),
        ];
        let process_speed_instructions = vec![
            " Process Speed Version ".into(),
            "<1>".blue().bold(),
            " 2.0x ".into(),
            "<2>".blue().bold(),
            " 1.33x ".into(),
            "<3>".blue().bold(),
            " 1.0x ".into(),
            "<4>".blue().bold(),
            " 0.8x ".into(),
            "<5>".blue().bold(),
            " 0.67x ".into(),
            " Normal Mode ".into(),
            "<p>".blue().bold(),
        ];
        let mode_instructions = match self.mode {
            Mode::Normal => Line::from(loop_instructions),
            Mode::SetBookmark => Line::from(bookmark_instructions),
            Mode::SetSpeed => Line::from(speed_instructions),
            Mode::ProcessSpeed => Line::from(process_speed_instructions),
        };

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let output_data = self.stream.lock().unwrap().output_data();
        let available_speeds = self.stream.lock().unwrap().get_available_speeds();

        let mode_display = match self.mode {
            Mode::Normal => "Normal".red(),
            Mode::SetBookmark => "Bookmark".red(),
            Mode::SetSpeed => "Speed".red(),
            Mode::ProcessSpeed => "Process Speed".red(),
        };

        let counter_text = Text::from(vec![
            Line::from(vec!["Position: ".into(), output_data.current_time.red()]),
            Line::from(vec!["Speed: ".into(), output_data.current_speed.red()]),
            Line::from(vec![
                "loop start: ".into(),
                output_data.loop_start.red(),
                " end: ".into(),
                output_data.loop_end.red(),
                " active: ".into(),
                output_data.is_looping.red(),
            ]),
            Line::from(vec!["Mode: ".into(), mode_display.into()]),
            Line::from(vec![
                "Available Speeds: ".into(),
                available_speeds.join(", ").red(),
            ]),
            Line::from(vec![
                "Bookmarks: [1] ".into(),
                output_data.bookmark_1.red(),
                " [2] ".into(),
                output_data.bookmark_2.red(),
                " [3] ".into(),
                output_data.bookmark_3.red(),
                " [4] ".into(),
                output_data.bookmark_4.red(),
                " [5] ".into(),
                output_data.bookmark_5.red(),
                " [6] ".into(),
                output_data.bookmark_6.red(),
                " [7] ".into(),
                output_data.bookmark_7.red(),
                " [8] ".into(),
                output_data.bookmark_8.red(),
                " [9] ".into(),
                output_data.bookmark_9.red(),
                " [0] ".into(),
                output_data.bookmark_0.red(),
            ]),
            common_instructions,
            mode_instructions,
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
