mod filter;
mod hammer;
mod piano;
mod ring_buffer;
mod soundboard;
mod string;

fn main() {
    println!("Hello, world!");
    let mut instrument: piano::Piano<f32> = piano::Piano::new(60.0, 44100.0, 5.0);

    for i in 0..100 {
        println!("{}", instrument.next());
    }
}
