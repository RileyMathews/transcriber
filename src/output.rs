use cpal::Stream;
use cpal::{
    Sample, StreamConfig,
    traits::{DeviceTrait, HostTrait},
};
use std::sync::{Arc, Mutex};

use crate::audio_stream::AudioStream;

pub fn output_stream(audio_stream: Arc<Mutex<AudioStream>>) -> Stream {
    println!("Creating output stream");
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")
        .expect("could not obtain output device");
    println!("Output device: {}", device.name().unwrap());

    let config = {
        let audio_stream_lock = audio_stream.lock().unwrap();
        StreamConfig {
            channels: audio_stream_lock.channels as u16,
            sample_rate: cpal::SampleRate(audio_stream_lock.sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        }
    };
    println!("Stream config: {:?}", config);

    let channels_usize = config.channels as usize;

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut reader_guard = audio_stream.lock().unwrap();

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

    return stream;
}
