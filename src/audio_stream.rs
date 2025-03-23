use hound::WavReader;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

pub struct AudioStreamOutputData {
    pub current_time: String,
    pub loop_start: String,
    pub loop_end: String,
    pub is_looping: String,
}

pub struct AudioStream {
    file: BufReader<File>,
    pub channels: usize,
    pub at_end: bool,
    bytes_per_sample: usize,
    pub sample_rate: usize,
    paused: bool,
    is_looping: bool,
    loop_sample_start: f32,
    loop_sample_end: f32,
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
            is_looping: false,
            loop_sample_start: 0.0,
            loop_sample_end: 0.0,
        }
    }

    pub fn output_data(&mut self) -> AudioStreamOutputData {
        let current_time = self.get_current_time_seconds();
        let loop_start = self.get_loop_start_seconds();
        let loop_end = self.get_seconds_for_sample(self.loop_sample_end);
        let is_looping = self.is_looping;

        AudioStreamOutputData {
            current_time: format!("{:.2}", current_time),
            loop_start: format!("{:.2}", loop_start),
            loop_end: format!("{:.2}", loop_end),
            is_looping: format!("{}", is_looping),
        }
    }

    pub fn toggle_play(&mut self) {
        self.paused = !self.paused;
    }

    pub fn read_frame(&mut self) -> Vec<i16> {
        let mut frame = vec![0i16; self.channels];
        let mut buffer = vec![0u8; self.channels * self.bytes_per_sample];

        if self.is_looping && self.get_current_sample_location() > self.loop_sample_end {
            self.seek_to_sample(self.loop_sample_start);
            return frame;
        }

        if self.paused {
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

    fn get_current_sample_location(&mut self) -> f32 {
        (self.file.stream_position().unwrap() / self.bytes_per_sample as u64) as f32
    }

    fn get_current_byte_location(&mut self) -> usize {
        self.file.stream_position().unwrap() as usize
    }

    fn get_seconds_for_sample(&mut self, sample: f32) -> f32 {
        (sample as f32 / self.sample_rate as f32) as f32
    }

    pub fn set_loop_start(&mut self) {
        self.loop_sample_start = self.get_current_sample_location()
    }

    pub fn set_loop_end(&mut self) {
        self.loop_sample_end = self.get_current_sample_location()
    }

    pub fn get_loop_start_seconds(&mut self) -> f32 {
        self.get_seconds_for_sample(self.loop_sample_start)
    }

    pub fn toggle_loop(&mut self) {
        self.is_looping = !self.is_looping;
    }

    pub fn get_current_time_seconds(&mut self) -> f32 {
        let current = self.get_current_sample_location();
        self.get_seconds_for_sample(current)
    }

    pub fn seek_forwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.sample_rate * seconds * self.channels;
        self.file
            .seek(SeekFrom::Current(bytes_to_seek as i64))
            .expect("Could not seek forwards");
    }

    pub fn seek_to_sample(&mut self, sample: f32) {
        let byte_location = (sample * self.bytes_per_sample as f32) as u64;
        self.file
            .seek(SeekFrom::Start(byte_location))
            .expect("Could not seek to sample");
    }

    pub fn seek_backwards(&mut self, seconds: usize) {
        let bytes_to_seek = self.sample_rate * seconds * self.channels;

        if self.get_current_byte_location() < bytes_to_seek {
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
