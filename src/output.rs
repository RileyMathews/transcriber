use cpal::Stream;
use cpal::{
    StreamConfig,
    traits::{DeviceTrait, HostTrait},
};
use std::sync::{Arc, Mutex};

use crate::audio_stream::AudioStream;

pub fn output_stream(audio_stream: Arc<Mutex<AudioStream>>) -> Stream {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device available")
        .expect("could not obtain output device");

    let config = {
        let audio_stream_lock = audio_stream.lock().unwrap();
        StreamConfig {
            channels: audio_stream_lock.channels as u16,
            sample_rate: cpal::SampleRate(audio_stream_lock.sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        }
    };

    let channels_usize = config.channels as usize;

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut reader_guard = audio_stream.lock().unwrap();

                for frame in output.chunks_mut(channels_usize) {
                    let samples = reader_guard.read_frame();

                    for (i, sample) in frame.iter_mut().enumerate() {
                        *sample = samples[i];
                    }
                }
            },
            |err| eprintln!("error occurred on the output stream: {}", err),
            None,
        )
        .expect("Could not open stream");

    return stream;
}
