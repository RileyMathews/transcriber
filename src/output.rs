use pipewire as pw;
use pw::{properties::properties, spa};
use spa::pod::Pod;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::audio_stream::AudioStream;

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
                *pw::keys::NODE_LATENCY => format!("128/{}", sample_rate),
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
                            let mut reader_guard = user_data.lock().unwrap();
                            let stride = channels_usize * 2;
                            let n_frames = slice.len() / stride;

                            for i in 0..n_frames {
                                let samples = reader_guard.read_frame();
                                for (j, &sample) in samples.iter().enumerate().take(channels_usize) {
                                    let start = i * stride + (j * 2);
                                    let end = start + 2;
                                    if end <= slice.len() {
                                        slice[start..end].copy_from_slice(&sample.to_le_bytes());
                                    }
                                }
                            }

                            let chunk = data.chunk_mut();
                            *chunk.offset_mut() = 0;
                            *chunk.stride_mut() = stride as _;
                            *chunk.size_mut() = (stride * n_frames) as _;
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
