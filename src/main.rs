use cpal::traits::StreamTrait;
use output::output_stream;
use save_data::SongData;
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
mod process_time_warp;
mod save_data;
use audio_stream::{AudioStream, Digits};
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    process_speed: Option<Vec<f32>>,

    file_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let filename = match args.file_path {
        Some(path) => path,
        None => {
            eprintln!("Please provide a file path.");
            return Ok(());
        }
    };

    match args.process_speed {
        None => (),
        Some(speed) => {
            for speed in speed {
                let updated_song_data = SongData::from_wave_file(&filename);
                println!("Processing speed version: {}", speed);
                let result = process_time_warp::process(&updated_song_data, speed);
                match result {
                    Ok(()) => println!("Done!"),
                    Err(message) => {
                        eprintln!("Error processing speed version: {}", message);
                    }
                }
            }
            return Ok(());
        }
    }

    let audio_stream = Arc::new(Mutex::new(AudioStream::from_wave_file(&filename)));
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
                KeyCode::Char('.') => self.stream.lock().unwrap().set_next_fastest_speed(),
                KeyCode::Char(',') => self.stream.lock().unwrap().set_next_slowest_speed(),
                _ => {
                    dbg!("Unhandled key event: {:?}", key_event);
                }
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
        let mode_instructions = match self.mode {
            Mode::Normal => Line::from(loop_instructions),
            Mode::SetBookmark => Line::from(bookmark_instructions),
        };

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let output_data = self.stream.lock().unwrap().output_data();

        let next_fastest_output = match output_data.next_fastest_speed {
            Some(speed) => format!("[>] {}", speed.speed),
            None => "".to_string(),
        };

        let next_slowest_output = match output_data.next_slowest_speed {
            Some(speed) => format!("[<] {}", speed.speed),
            None => "".to_string(),
        };

        let mode_display = match self.mode {
            Mode::Normal => "Normal".red(),
            Mode::SetBookmark => "Bookmark".red(),
        };

        let counter_text = Text::from(vec![
            Line::from(vec!["Position: ".into(), output_data.current_time.red()]),
            Line::from(vec![
                "Speed: ".into(),
                output_data.current_speed.speed.to_string().red(),
            ]),
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
            Line::from(vec![next_slowest_output.into(), next_fastest_output.into()]),
            mode_instructions,
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
