use hound::WavReader;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use blake3::Hasher;
use std::fs;
use dirs;

pub enum Digits {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
}

pub struct AudioStreamOutputData {
    pub current_time: String,
    pub loop_start: String,
    pub loop_end: String,
    pub is_looping: String,
    pub bookmark_1: String,
    pub bookmark_2: String,
    pub bookmark_3: String,
    pub bookmark_4: String,
    pub bookmark_5: String,
    pub bookmark_6: String,
    pub bookmark_7: String,
    pub bookmark_8: String,
    pub bookmark_9: String,
    pub bookmark_0: String,
}

pub struct Bookmarks {
    bookmark_1: f32,
    bookmark_2: f32,
    bookmark_3: f32,
    bookmark_4: f32,
    bookmark_5: f32,
    bookmark_6: f32,
    bookmark_7: f32,
    bookmark_8: f32,
    bookmark_9: f32,
    bookmark_0: f32,
}

impl Bookmarks {
    pub fn new() -> Self {
        Bookmarks {
            bookmark_1: 0.0,
            bookmark_2: 0.0,
            bookmark_3: 0.0,
            bookmark_4: 0.0,
            bookmark_5: 0.0,
            bookmark_6: 0.0,
            bookmark_7: 0.0,
            bookmark_8: 0.0,
            bookmark_9: 0.0,
            bookmark_0: 0.0,
        }
    }

    pub fn set_bookmark(&mut self, bookmark: Digits, sample: f32) {
        match bookmark {
            Digits::One => self.bookmark_1 = sample,
            Digits::Two => self.bookmark_2 = sample,
            Digits::Three => self.bookmark_3 = sample,
            Digits::Four => self.bookmark_4 = sample,
            Digits::Five => self.bookmark_5 = sample,
            Digits::Six => self.bookmark_6 = sample,
            Digits::Seven => self.bookmark_7 = sample,
            Digits::Eight => self.bookmark_8 = sample,
            Digits::Nine => self.bookmark_9 = sample,
            Digits::Zero => self.bookmark_0 = sample,
        }
    }

    pub fn get_bookmark(&self, bookmark: Digits) -> f32 {
        match bookmark {
            Digits::One => self.bookmark_1,
            Digits::Two => self.bookmark_2,
            Digits::Three => self.bookmark_3,
            Digits::Four => self.bookmark_4,
            Digits::Five => self.bookmark_5,
            Digits::Six => self.bookmark_6,
            Digits::Seven => self.bookmark_7,
            Digits::Eight => self.bookmark_8,
            Digits::Nine => self.bookmark_9,
            Digits::Zero => self.bookmark_0,
        }
    }
}

pub struct AudioStream {
    file: BufReader<File>,
    pub channels: usize,
    bytes_per_sample: usize,
    pub sample_rate: usize,
    paused: bool,
    is_looping: bool,
    loop_sample_start: f32,
    loop_sample_end: f32,
    bookmarks: Bookmarks,
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
            bytes_per_sample: 2,
            sample_rate: wave_spec.sample_rate as usize,
            paused: false,
            is_looping: false,
            loop_sample_start: 0.0,
            loop_sample_end: 0.0,
            bookmarks: Bookmarks::new(),
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
            bookmark_1: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::One))),
            bookmark_2: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Two))),
            bookmark_3: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Three))),
            bookmark_4: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Four))),
            bookmark_5: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Five))),
            bookmark_6: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Six))),
            bookmark_7: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Seven))),
            bookmark_8: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Eight))),
            bookmark_9: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Nine))),
            bookmark_0: format!("{:.2}", self.get_seconds_for_sample(self.bookmarks.get_bookmark(Digits::Zero))),
        }
    }

    pub fn set_bookmark(&mut self, bookmark: Digits) {
        let sample = self.get_current_sample_location();
        self.bookmarks.set_bookmark(bookmark, sample);
    }

    pub fn seek_to_bookmark(&mut self, bookmark: Digits) {
        let bookmark_value = self.bookmarks.get_bookmark(bookmark);
        self.seek_to_sample(bookmark_value);
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
                self.paused = true;
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
