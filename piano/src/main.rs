mod filter;
mod hammer;
mod loss;
mod piano;
mod ring_buffer;
mod string;
mod thirian;

use hound;

fn main() {
    let mut instrument: piano::Piano = piano::Piano::new(60, 44100.0, 5.0);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
    let amplitude = i16::MAX as f32;
    for _ in 0..44100 * 3 {
        writer
            .write_sample((10.0 * instrument.go() * amplitude) as i16)
            .unwrap();
    }
}
