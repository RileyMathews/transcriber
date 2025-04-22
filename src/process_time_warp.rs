use crate::save_data::SongData;

pub fn process(song_data: SongData, speed: f32) -> Result<(), String> {
    let output_path = song_data.song_dir.join(format!("speed_{:.2}.wav", speed));

    if song_data
        .speed_versions
        .iter()
        .any(|v| v.speed == speed && v.file_path == output_path)
    {
        println!("Speed version already exists: {}", output_path.display());
        return Ok(());
    }

    let status = std::process::Command::new("rubberband-r3")
        .arg("-t")
        .arg(format!("{:.2}", speed))
        .arg(&song_data.original_file_path)
        .arg(&output_path)
        .status()
        .map_err(|e| format!("Failed to run rubberband: {}", e))?;

    if !status.success() {
        return Err("rubberband failed to process the file".to_string());
    }

    song_data.save_new_speed_version(output_path, speed);

    return Ok(());
}
