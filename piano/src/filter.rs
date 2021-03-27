use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::ring_buffer::RingBuffer;

pub struct Filter<T> {
    pub n: usize,
    pub a: Vec<T>,
    pub b: Vec<T>,
    pub name: String,
    pub x: RingBuffer<T>,
    pub y: RingBuffer<T>,
}

impl<T: Clone + Copy + Float + Zero + FloatConst> Filter<T> {
    pub fn new(n: usize, a: Vec<T>, b: Vec<T>, name: String) -> Filter<T> {
        assert_eq!(n + 1, a.len());
        assert_eq!(n + 1, b.len());
        Filter {
            n,
            a,
            b,
            name,
            x: RingBuffer::new(n + 1, T::zero()),
            y: RingBuffer::new(n + 1, T::zero()),
        }
    }

    pub fn filter(&mut self, in_value: T) -> T {
        let mut out_value: T = T::zero();
        self.x.push(in_value);
        for (&x_, &b_) in self.x.iter().zip(self.b.iter()) {
            out_value = out_value + x_ * b_;
            // println!("b: {}, x: {}", b_.to_f64().unwrap(), x_.to_f64().unwrap());
        }
        for (&y_, &a_) in self.y.iter().take(self.n).zip(self.a.iter().skip(1)) {
            out_value = out_value - y_ * a_;
            // println!("a: {}, y: {}", a_.to_f64().unwrap(), y_.to_f64().unwrap());
        }
        out_value = out_value / self.a[0];
        self.y.push(out_value);
        out_value
    }

    pub fn groupdelay(&self, note_frequency: T, sample_frequency: T) -> T {
        let df: T = T::from(5).unwrap();
        let f2: T = note_frequency + df;
        let f1: T = note_frequency - df;
        let omega2: T = T::from(2).unwrap() * T::PI() * f2 / sample_frequency;
        let omega1: T = T::from(2).unwrap() * T::PI() * f1 / sample_frequency;
        (omega2 * self.phasedelay(f2, sample_frequency)
            - omega1 * self.phasedelay(f1, sample_frequency))
            / (omega2 - omega1)
    }

    fn phasedelay(&self, note_frequency: T, sample_frequency: T) -> T {
        let mut hn: [T; 2] = [T::zero(), T::zero()];
        let mut hd: [T; 2] = [T::zero(), T::zero()];
        let mut h: [T; 2] = [T::zero(), T::zero()];

        let omega: T = T::from(2).unwrap() * T::PI() * note_frequency / sample_frequency;
        for k in 0..(self.n + 1) {
            hn[0] = hn[0] + T::cos(T::from(k).unwrap() * omega) * self.b[k];
            hn[1] = hn[1] + T::sin(T::from(k).unwrap() * omega) * self.b[k];
        }
        for k in 0..(self.n + 1) {
            hd[0] = hd[0] + T::cos(T::from(k).unwrap() * omega) * self.a[k];
            hd[1] = hd[1] + T::sin(T::from(k).unwrap() * omega) * self.a[k];
        }
        self.complex_divide(&hn, &hd, &mut h);
        let arg: T = h[1].atan2(h[0]);
        let arg: T = if arg < T::zero() {
            arg + T::from(2).unwrap() * T::PI()
        } else {
            arg
        };

        return arg / omega;
    }

    fn complex_divide(&self, hn: &[T; 2], hd: &[T; 2], h: &mut [T; 2]) {
        let magn: T = T::sqrt(hn[0] * hn[0] + hn[1] * hn[1]);
        let argn: T = hn[1].atan2(hn[0]);
        let magd: T = T::sqrt(hd[0] * hd[0] + hd[1] * hd[1]);
        let argd: T = hd[1].atan2(hd[0]);
        let mag: T = magn / magd;
        let arg: T = argn - argd;
        h[0] = mag * T::cos(arg);
        h[1] = mag * T::sin(arg);
    }
}
