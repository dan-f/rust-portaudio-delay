extern crate portaudio;

use portaudio as p;

/**
 * A basic one-shot delay line implementation.
 */

const CHANNELS: usize = 2;
const FRAME_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: usize = 128;
const SAMPLES_PER_BUFFER: usize = FRAMES_PER_BUFFER * CHANNELS;
const DELAY_TIME_SECONDS: f32 = 0.5;
const DELAY_LINE_BUFLEN: usize = (FRAME_RATE as f32 * CHANNELS as f32 * DELAY_TIME_SECONDS) as usize;
const BUFFERS_PER_DELAY_LINE: usize = DELAY_LINE_BUFLEN / SAMPLES_PER_BUFFER;
const DRY_AMP: f32 = 0.5;
const WET_AMP: f32 = 1.0 - DRY_AMP;

fn main() {
    println!("Delay. Sample rate: {}, Buf size: {}", FRAME_RATE, FRAMES_PER_BUFFER);
    match run() {
        Ok(_) => {},
        e => {
            eprintln!("Delay failed with: {:?}", e);
        },
    }
}

fn stereo_buffer_idxs(frame_idx: usize) -> (usize, usize) {
    (frame_idx * 2, frame_idx * 2 + 1)
}

fn compute_frame(frame_idx: usize, in_buffer: &[f32], delay_line: &[f32]) -> (f32, f32) {
    let (sample_left_idx, sample_right_idx) = stereo_buffer_idxs(frame_idx);
    let delay_left_idx = (BUFFERS_PER_DELAY_LINE - 1) * SAMPLES_PER_BUFFER + sample_left_idx;
    let delay_right_idx = (BUFFERS_PER_DELAY_LINE - 1) * SAMPLES_PER_BUFFER + sample_right_idx;
    (
        in_buffer[sample_left_idx] * DRY_AMP
            + delay_line[delay_left_idx] * WET_AMP,
        in_buffer[sample_right_idx] * DRY_AMP
            + delay_line[delay_right_idx] * WET_AMP
    )
}

fn run() -> Result<(), p::Error> {
    let pa = try!(p::PortAudio::new());
    let settings = try!(pa.default_duplex_stream_settings(CHANNELS as i32, CHANNELS as i32, FRAME_RATE, FRAMES_PER_BUFFER as u32));

    let mut delay_line: [f32; DELAY_LINE_BUFLEN] = [0.0; DELAY_LINE_BUFLEN];

    let audio_callback = move |stream: p::DuplexStreamCallbackArgs<f32, f32>| {
        for frame_idx in 0..stream.frames {
            let (sample_left_idx, sample_right_idx) = stereo_buffer_idxs(frame_idx);
            let (sample_left, sample_right) = compute_frame(frame_idx, &stream.in_buffer, &delay_line);
            stream.out_buffer[sample_left_idx] = sample_left;
            stream.out_buffer[sample_right_idx] = sample_right;
        }
        // move values down the delay line
        for sample_idx in (SAMPLES_PER_BUFFER..DELAY_LINE_BUFLEN).rev() {
            delay_line[sample_idx] = delay_line[sample_idx - SAMPLES_PER_BUFFER];
        }
        // queue the current buffer up into the delay line
        for sample_idx in 0..SAMPLES_PER_BUFFER {
            delay_line[sample_idx] = stream.in_buffer[sample_idx];
        }
        p::Continue
    };

    let mut stream = try!(pa.open_non_blocking_stream(settings, audio_callback));
    try!(stream.start());

    println!("Playing.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    };
}
