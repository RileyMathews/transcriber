use std::{thread, time::Duration};

use cpal::{
    Sample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use hound::WavReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).expect("no file given");
    println!("opening {}", filename);

    let mut reader = WavReader::open(filename).expect("Could not open file");
    let wave_spec = reader.spec();

    // assume the sample rate and format of my
    // testing wave file.
    // this will likely break on other files
    // will take care of that later
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap_or(0) as f32 / i16::MAX as f32)
        .collect();

    let mut sample_position = 0;

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
                for frame in output.chunks_mut(channels_usize) {
                    for (_ch, sample) in frame.iter_mut().enumerate() {
                        if sample_position < samples.len() {
                            let value = samples[sample_position];
                            *sample = Sample::from_sample(value);
                            sample_position += 1;
                        } else {
                            *sample = Sample::from_sample(0.0)
                        }
                    }
                }
            },
            |err| eprintln!("error occurred on the output stream: {}", err),
            None,
        )
        .expect("Could not open stream");

    stream.play()?;

    thread::sleep(Duration::from_secs(10));

    Ok(())
}
