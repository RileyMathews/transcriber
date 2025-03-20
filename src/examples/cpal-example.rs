use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use hound::WavReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Taking filename from command-line arguments or using a default
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("example.wav");

    println!("Opening WAV file: {}", filename);

    // Open the WAV file with hound
    let mut reader = WavReader::open(filename)?;
    let wav_spec = reader.spec();

    // Display WAV file information
    println!("WAV Spec: {:?}", wav_spec);

    // Read all samples into memory
    // For stereo files, samples are interleaved (L, R, L, R, ...)
    let samples: Vec<f32> = match wav_spec.sample_format {
        hound::SampleFormat::Int => {
            if wav_spec.bits_per_sample == 16 {
                reader
                    .samples::<i16>()
                    .map(|s| s.unwrap_or(0) as f32 / i16::MAX as f32)
                    .collect()
            } else if wav_spec.bits_per_sample == 24 {
                reader
                    .samples::<i32>()
                    .map(|s| s.unwrap_or(0) as f32 / 0x7FFFFF as f32)
                    .collect()
            } else if wav_spec.bits_per_sample == 32 {
                reader
                    .samples::<i32>()
                    .map(|s| s.unwrap_or(0) as f32 / i32::MAX as f32)
                    .collect()
            } else {
                return Err(format!("Unsupported bit depth: {}", wav_spec.bits_per_sample).into());
            }
        }
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
    };

    // Convert samples to Arc<Mutex<>> for thread-safe sharing
    let samples = Arc::new(Mutex::new(samples));
    let sample_pos = Arc::new(Mutex::new(0));

    // Initialize CPAL audio
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;

    println!("Using audio device: {}", device.name()?);

    // Configure audio stream based on WAV properties
    let config = StreamConfig {
        channels: wav_spec.channels,
        sample_rate: cpal::SampleRate(wav_spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    println!("Audio stream config: {:?}", config);

    // Determine the sample format to use
    let sample_format = device.default_output_config()?.sample_format();
    println!("Using sample format: {:?}", sample_format);

    // The audio callback closure that will be called to fill audio buffers
    let samples_clone = samples.clone();
    let sample_pos_clone = sample_pos.clone();

    // Total number of samples in the file
    let total_samples = {
        let samples = samples.lock().unwrap();
        samples.len()
    };

    // Create and run the audio stream
    let stream = match sample_format {
        SampleFormat::F32 => run_audio::<f32>(
            &device,
            &config,
            samples_clone,
            sample_pos_clone,
            wav_spec.channels as usize,
        )?,
        SampleFormat::I16 => run_audio::<i16>(
            &device,
            &config,
            samples_clone,
            sample_pos_clone,
            wav_spec.channels as usize,
        )?,
        SampleFormat::U16 => run_audio::<u16>(
            &device,
            &config,
            samples_clone,
            sample_pos_clone,
            wav_spec.channels as usize,
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    // Start the stream
    stream.play()?;

    // Calculate playback duration and add a small buffer for safety
    let duration_secs =
        total_samples as f32 / (wav_spec.sample_rate as f32 * wav_spec.channels as f32);
    let sleep_duration = Duration::from_secs_f32(duration_secs + 0.5);

    println!("Playing... (duration: {:.2} seconds)", duration_secs);

    // Sleep while audio plays
    thread::sleep(sleep_duration);

    // Stop the stream
    drop(stream);
    println!("Playback complete!");

    Ok(())
}

// Function to run an audio stream with the appropriate sample type
fn run_audio<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_pos: Arc<Mutex<usize>>,
    channels: usize,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: Sample + Send + 'static + cpal::SizedSample + cpal::FromSample<f32>,
{
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut pos = sample_pos.lock().unwrap();
            let samples = samples.lock().unwrap();

            // Fill the output buffer with our audio data
            for frame in output.chunks_mut(channels) {
                for (_ch, sample) in frame.iter_mut().enumerate() {
                    // Get the correct sample for this channel
                    if *pos < samples.len() {
                        let value = samples[*pos];
                        *sample = Sample::from_sample(value);

                        // Move to the next sample
                        *pos += 1;
                    } else {
                        // End of file, fill with silence
                        *sample = Sample::from_sample(0.0);
                    }
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

