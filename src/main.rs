use cpal::{
    Sample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use hound::WavReader;
use k_board::{keyboard::Keyboard, keys::Keys};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).expect("no file given");
    println!("opening {}", filename);

    // Open the file and get its specs, but we'll re-open it for streaming
    let reader = WavReader::open(filename).expect("Could not open file");
    let wave_spec = reader.spec();
    let header_size = 44; // Standard WAV header size, may need adjustment for non-standard WAVs

    // Create a structure to hold the streaming state that can be shared across threads
    let file_reader = Arc::new(Mutex::new(StreamingState::new(
        filename,
        wave_spec.channels as usize,
        header_size,
        wave_spec.sample_rate as usize,
    )));

    let file_reader_clone = file_reader.clone();

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
                let mut reader_guard = file_reader_clone.lock().unwrap();

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
                            "Position: {:.2} seconds: {}",
                            reader_guard.get_current_time_seconds(),
                            reader_guard.get_current_sample_location()
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
                    let mut reader_guard = file_reader.lock().unwrap();
                    reader_guard.seek_backwards(5);
                }
                Keys::Right => {
                    let mut reader_guard = file_reader.lock().unwrap();
                    reader_guard.seek_forwards(5);
                }
                _ => {}
            }
        }

        let reader_guard = file_reader.lock().unwrap();
        if reader_guard.at_end {
            println!("Playback complete");
            break;
        }
    }

    Ok(())
}

// A custom struct to handle streaming directly from the file
struct StreamingState {
    file: BufReader<File>,
    channels: usize,
    at_end: bool,
    bytes_per_sample: usize,
    samples_per_second: usize,
}

impl StreamingState {
    fn new(filename: &str, channels: usize, header_size: usize, samples_per_second: usize) -> Self {
        let file = File::open(filename).expect("Could not open file");
        let mut reader = BufReader::new(file);

        // Skip the WAV header
        reader
            .seek(SeekFrom::Start(header_size as u64))
            .expect("Could not seek past header");

        StreamingState {
            file: reader,
            channels,
            at_end: false,
            bytes_per_sample: 2, // For i16 samples
            samples_per_second,
        }
    }

    fn read_frame(&mut self) -> Vec<i16> {
        let mut frame = vec![0i16; self.channels];
        let mut buffer = vec![0u8; self.channels * self.bytes_per_sample];

        match self.file.read_exact(&mut buffer) {
            Ok(_) => {
                // Convert bytes to i16 samples (little endian)
                for i in 0..self.channels {
                    let idx = i * self.bytes_per_sample;
                    frame[i] = i16::from_le_bytes([buffer[idx], buffer[idx + 1]]);
                }
            }
            Err(_) => {
                // End of file or error
                self.at_end = true;
            }
        }

        frame
    }

    fn get_current_sample_location(&mut self) -> usize {
        (self.file.stream_position().unwrap() / self.bytes_per_sample as u64) as usize
    }

    fn get_current_byte_location(&mut self) -> usize {
        self.file.stream_position().unwrap() as usize
    }

    fn get_current_time_seconds(&mut self) -> usize {
        self.get_current_sample_location() / self.samples_per_second
    }

    fn seek_forwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.samples_per_second * seconds * self.channels;
        self.file
            .seek(SeekFrom::Current(bytes_to_seek as i64))
            .expect("Could not seek forwards");
    }

    fn seek_backwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.samples_per_second * seconds * self.channels;

        if self.get_current_byte_location() < bytes_to_seek {
            println!("Seeking to start of file");
            self.file
                .seek(SeekFrom::Start(44))
                .expect("Could not seek to start");
            return;
        }

        self.file
            .seek(SeekFrom::Current(-(bytes_to_seek as i64)))
            .expect("Could not seek backwards");
    }
}
