use pipewire as pw;
use pw::{properties::properties, spa};
use spa::pod::Pod;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::audio_stream::AudioStream;

// Quantum size requested from PipeWire - controls how often the callback fires
// and how many frames we process per callback. Set to 128 for low latency.
// Note: PipeWire may allocate larger buffers, but we only fill this many frames
// to keep position updates smooth in the TUI.
const QUANTUM_SIZE: usize = 128;

pub struct PipewireStream {
    _thread: thread::JoinHandle<()>,
}

pub fn output_stream(audio_stream: Arc<Mutex<AudioStream>>) -> PipewireStream {
    let thread = thread::spawn(move || {
        pw::init();

        let mainloop = pw::main_loop::MainLoopRc::new(None).expect("Failed to create mainloop");
        let context = pw::context::ContextRc::new(&mainloop, None).expect("Failed to create context");
        let core = context.connect_rc(None).expect("Failed to connect to PipeWire");

        let (sample_rate, channels) = {
            let audio_stream_lock = audio_stream.lock().unwrap();
            (audio_stream_lock.sample_rate as u32, audio_stream_lock.channels as u32)
        };

        let stream = pw::stream::StreamBox::new(
            &core,
            "audio-playback",
            properties! {
                *pw::keys::MEDIA_TYPE => "Audio",
                *pw::keys::MEDIA_CATEGORY => "Playback",
                *pw::keys::MEDIA_ROLE => "Music",
                *pw::keys::NODE_LATENCY => format!("{}/{}", QUANTUM_SIZE, sample_rate),
            },
        )
        .expect("Failed to create stream");

        let channels_usize = channels as usize;

        let _listener = stream
            .add_local_listener_with_user_data(audio_stream)
            .process(move |stream, user_data| {
                if let Some(mut buffer) = stream.dequeue_buffer() {
                    let datas = buffer.datas_mut();
                    if let Some(data) = datas.first_mut() {
                        if let Some(slice) = data.data() {
                            const BYTES_PER_SAMPLE: usize = 2; // i16
                            let stride = channels_usize * BYTES_PER_SAMPLE;

                            // PipeWire provides a large buffer (can be 24k+ frames)
                            let pipewire_buffer_frames = slice.len() / stride;

                            // We intentionally only fill QUANTUM_SIZE frames per callback
                            // (not the full buffer) to keep the file position advancing in
                            // small increments. This makes the TUI position display update
                            // smoothly instead of jumping in large chunks.
                            let frames_to_process = QUANTUM_SIZE.min(pipewire_buffer_frames);

                            let mut reader_guard = user_data.lock().unwrap();
                            for i in 0..frames_to_process {
                                let samples = reader_guard.read_frame();
                                for (j, &sample) in samples.iter().enumerate().take(channels_usize) {
                                    let byte_offset = i * stride + j * BYTES_PER_SAMPLE;
                                    slice[byte_offset..byte_offset + BYTES_PER_SAMPLE]
                                        .copy_from_slice(&sample.to_le_bytes());
                                }
                            }

                            // Tell PipeWire how much data we actually wrote (not the full buffer)
                            let chunk = data.chunk_mut();
                            *chunk.offset_mut() = 0;
                            *chunk.stride_mut() = stride as _;
                            *chunk.size_mut() = (stride * frames_to_process) as _;
                        }
                    }
                }
            })
            .register()
            .expect("Failed to register listener");

        let mut audio_info = spa::param::audio::AudioInfoRaw::new();
        audio_info.set_format(spa::param::audio::AudioFormat::S16LE);
        audio_info.set_rate(sample_rate);
        audio_info.set_channels(channels);

        let obj = pw::spa::pod::Object {
            type_: pw::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: pw::spa::param::ParamType::EnumFormat.as_raw(),
            properties: audio_info.into(),
        };
        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
        .unwrap()
        .0
        .into_inner();

        let mut params = [Pod::from_bytes(&values).unwrap()];

        stream
            .connect(
                spa::utils::Direction::Output,
                None,
                pw::stream::StreamFlags::AUTOCONNECT
                    | pw::stream::StreamFlags::MAP_BUFFERS
                    | pw::stream::StreamFlags::RT_PROCESS,
                &mut params,
            )
            .expect("Failed to connect stream");

        mainloop.run();
    });

    PipewireStream { _thread: thread }
}
