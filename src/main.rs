use cpal::{
    Sample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use hound::WavReader;
use k_board::{keyboard::Keyboard, keys::Keys};
use std::sync::{Arc, Mutex};
mod audio_stream;
use audio_stream::AudioStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).expect("no file given");
    println!("opening {}", filename);

    // Open the file and get its specs, but we'll re-open it for streaming
    let reader = WavReader::open(filename).expect("Could not open file");
    let wave_spec = reader.spec();

    // Create a structure to hold the streaming state that can be shared across threads
    let audio_stream = Arc::new(Mutex::new(AudioStream::from_wave_file(filename)));

    let audio_stream_clone = audio_stream.clone();

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")
        .expect("could not obtain output device");

    let config = StreamConfig {
        channels: wave_spec.channels,
        sample_rate: cpal::SampleRate(wave_spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let channels_usize = config.channels as usize;

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut reader_guard = audio_stream_clone.lock().unwrap();

                for frame in output.chunks_mut(channels_usize) {
                    if reader_guard.at_end {
                        // Fill with silence if we've reached the end
                        for sample in frame.iter_mut() {
                            *sample = Sample::from_sample(0.0);
                        }
                    } else {
                        // Read a frame of samples
                        let samples = reader_guard.read_frame();

                        for (i, sample) in frame.iter_mut().enumerate() {
                            if i < samples.len() {
                                *sample = samples[i];
                            } else {
                                *sample = Sample::from_sample(0.0);
                            }
                        }

                        println!(
                            "Position: {:.2} seconds",
                            reader_guard.get_current_time_seconds(),
                        );
                    }
                }
            },
            |err| eprintln!("error occurred on the output stream: {}", err),
            None,
        )
        .expect("Could not open stream");

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
                    let mut reader_guard = audio_stream.lock().unwrap();
                    reader_guard.seek_backwards(5);
                }
                Keys::Right => {
                    let mut reader_guard = audio_stream.lock().unwrap();
                    reader_guard.seek_forwards(5);
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
