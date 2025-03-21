use hound::WavReader;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

pub struct AudioStream {
    file: BufReader<File>,
    pub channels: usize,
    pub at_end: bool,
    bytes_per_sample: usize,
    pub sample_rate: usize,
    paused: bool,
}

impl AudioStream {
    pub fn from_wave_file(file_path: &str) -> Self {
        let reader = WavReader::open(&file_path).expect("Could not open file");
        let wave_spec = reader.spec();
        let wave_header_size = 44;

        let file = File::open(&file_path).expect("Could not open");
        let mut reader = BufReader::new(file);

        reader
            .seek(SeekFrom::Start(wave_header_size as u64))
            .expect("Could not seek past header");

        AudioStream {
            file: reader,
            channels: wave_spec.channels as usize,
            at_end: false,
            bytes_per_sample: 2,
            sample_rate: wave_spec.sample_rate as usize,
            paused: false,
        }
    }

    pub fn toggle_play(&mut self) {
        self.paused = !self.paused;
    }

    pub fn read_frame(&mut self) -> Vec<i16> {
        let mut frame = vec![0i16; self.channels];
        let mut buffer = vec![0u8; self.channels * self.bytes_per_sample];

        if self.paused {
            println!("Paused");
            return frame;
        }

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

    pub fn get_current_time_seconds(&mut self) -> usize {
        self.get_current_sample_location() / self.sample_rate
    }

    pub fn seek_forwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.sample_rate * seconds * self.channels;
        self.file
            .seek(SeekFrom::Current(bytes_to_seek as i64))
            .expect("Could not seek forwards");
    }

    pub fn seek_backwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.sample_rate * seconds * self.channels;

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
