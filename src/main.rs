use cpal::traits::StreamTrait;
use k_board::{keyboard::Keyboard, keys::Keys};
use output::output_stream;
use std::sync::{Arc, Mutex};
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

    // Keep the main thread alive until playback completes
    println!("Playing... Press Ctrl+C to stop");
    loop {
        //std::thread::sleep(Duration::from_millis(500));
        // if j is pressed then seek backwards
        let keyboard = Keyboard::new();
        for key in keyboard {
            match key {
                Keys::Left => {
                    let mut stream = audio_stream.lock().unwrap();
                    stream.seek_backwards(5);
                }
                Keys::Right => {
                    let mut stream = audio_stream.lock().unwrap();
                    stream.seek_forwards(5);
                }
                Keys::Up => {
                    let mut stream = audio_stream.lock().unwrap();
                    stream.toggle_play();
                }
                _ => {}
            }
        }

        let reader_guard = audio_stream.lock().unwrap();
        if reader_guard.at_end {
            println!("Playback complete");
            break;
        }
    }

    Ok(())
}
