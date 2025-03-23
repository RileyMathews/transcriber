use cpal::traits::StreamTrait;
use k_board::{keyboard::Keyboard, keys::Keys};
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
use audio_stream::AudioStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).expect("no file given");
    println!("opening {}", filename);

    // Create a structure to hold the streaming state that can be shared across threads
    let audio_stream = Arc::new(Mutex::new(AudioStream::from_wave_file(filename)));

    println!("Audio stream created");

    let stream = output_stream(audio_stream.clone());

    println!("Output stream created");

    stream.play()?;

    let mut terminal = ratatui::init();
    let app_result = App {
        stream: audio_stream,
        exit: false,
    }
    .run(&mut terminal);
    ratatui::restore();

    // Keep the main thread alive until playback completes
    println!("Playing... Press Ctrl+C to stop");
    //loop {
    //    //std::thread::sleep(Duration::from_millis(500));
    //    // if j is pressed then seek backwards
    //    let keyboard = Keyboard::new();
    //    for key in keyboard {
    //        match key {
    //            Keys::Left => {
    //                let mut stream = audio_stream.lock().unwrap();
    //                stream.seek_backwards(5);
    //            }
    //            Keys::Right => {
    //                let mut stream = audio_stream.lock().unwrap();
    //                stream.seek_forwards(5);
    //            }
    //            Keys::Up => {
    //                let mut stream = audio_stream.lock().unwrap();
    //                stream.toggle_play();
    //            }
    //            _ => {}
    //        }
    //    }
    //
    //    let reader_guard = audio_stream.lock().unwrap();
    //    if reader_guard.at_end {
    //        println!("Playback complete");
    //        break;
    //    }
    //}

    Ok(app_result?)
}

pub struct App {
    stream: Arc<Mutex<AudioStream>>,
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
        if !event::poll(Duration::from_millis(25)).unwrap() {
            return Ok(());
        }
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.stream.lock().unwrap().seek_backwards(5),
            KeyCode::Char('l') => self.stream.lock().unwrap().seek_forwards(5),
            KeyCode::Char('k') => self.stream.lock().unwrap().toggle_play(),
            KeyCode::Char('u') => self.stream.lock().unwrap().set_loop_start(),
            KeyCode::Char('o') => self.stream.lock().unwrap().set_loop_end(),
            KeyCode::Char('i') => self.stream.lock().unwrap().toggle_loop(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
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
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let output_data = self.stream.lock().unwrap().output_data();

        let counter_text = Text::from(vec![
            Line::from(vec!["Value: ".into(), output_data.current_time.yellow()]),
            Line::from(vec!["loop start: ".into(), output_data.loop_start.yellow()]),
            Line::from(vec!["loop end: ".into(), output_data.loop_end.yellow()]),
            Line::from(vec!["looping: ".into(), output_data.is_looping.yellow()]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
