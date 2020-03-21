mod filter;
mod hammer;
mod piano;
mod ring_buffer;
mod soundboard;
mod string;

use hound;

fn main() {
    let mut instrument: piano::Piano<f32> = piano::Piano::new(60.0, 44100.0, 5.0);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
    let amplitude = i16::MAX as f32;
    for i in 0..44100 * 5 {
        writer
            .write_sample((instrument.next() * amplitude) as i16)
            .unwrap();
    }
}
