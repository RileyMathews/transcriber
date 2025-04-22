use blake3::Hasher;
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpeedVersion {
    pub speed: f32,
    pub file_path: PathBuf,
}

pub struct SongData {
    pub original_file_path: PathBuf,
    pub hash: String,
    pub song_dir: PathBuf,
    pub speed_versions: Vec<SpeedVersion>,
}

impl SongData {
    pub fn from_wave_file(file_path: &str) -> Self {
        // First, read and hash the entire file
        let mut hasher = Hasher::new();
        let mut file = std::fs::File::open(file_path).expect("Could not open file");
        std::io::copy(&mut file, &mut hasher).expect("Could not read file for hashing");
        let hash = hasher.finalize().to_hex().to_string();

        // Create the songs directory if it doesn't exist
        let mut song_dir = dirs::data_dir().expect("Could not find data directory");
        song_dir.push("transcriber");
        song_dir.push("songs");
        song_dir.push(&hash);

        if !song_dir.exists() {
            fs::create_dir_all(&song_dir).expect("Could not create song directory");
        }

        let speed_versions_path = song_dir.join("speed_versions.json");

        let mut versions = vec![SpeedVersion {
            speed: 1.0,
            file_path: PathBuf::from(file_path),
        }];

        if speed_versions_path.exists() {
            if let Ok(speed_versions_str) = fs::read_to_string(&speed_versions_path) {
                if let Ok(loaded_versions) =
                    serde_json::from_str::<Vec<SpeedVersion>>(&speed_versions_str)
                {
                    // Only add versions that still exist on disk
                    versions.extend(loaded_versions.into_iter().filter(|v| v.file_path.exists()));
                }
            }
        }

        SongData {
            original_file_path: PathBuf::from(file_path),
            hash,
            song_dir,
            speed_versions: versions,
        }
    }

    pub fn save_new_speed_version(&self, file_path: PathBuf, speed: f32) {
        let speed_versions_path = self.song_dir.join("speed_versions.json");

        let mut versions = self.speed_versions.clone();

        versions.extend([SpeedVersion { speed, file_path }]);

        let stringified = serde_json::to_string_pretty(&versions).unwrap();
        fs::write(speed_versions_path, stringified).expect("could not update versions");
    }
}

