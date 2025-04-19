use blake3::Hasher;
use std::fs;
use std::path::PathBuf;
use dirs;

pub struct SongData {
    pub hash: String,
    pub song_dir: PathBuf,
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

        SongData {
            hash,
            song_dir,
        }
    }
} 