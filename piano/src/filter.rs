use num_traits::identities::Zero;
use num_traits::Num;

use super::ring_buffer::RingBuffer;

pub struct Filter<T> {
    k: usize,
    a: Vec<T>,
    b: Vec<T>,
    x: RingBuffer<T>,
    y: RingBuffer<T>,
}

impl<T: Clone + Copy + Num + Zero> Filter<T> {
    fn new(k: usize, a: Vec<T>, b: Vec<T>, init: T) -> Filter<T> {
        assert_eq!(k, a.len());
        assert_eq!(k, b.len());
        Filter {
            k,
            a,
            b,
            x: RingBuffer::new(k, init),
            y: RingBuffer::new(k, init),
        }
    }

    pub fn filter(&mut self, in_value: T) -> T {
        let mut out_value: T = T::zero();
        self.x.push(in_value);
        for (&x_, &b_) in self.x.iter().zip(self.b.iter()) {
            out_value = out_value + x_ * b_;
        }
        for (&y_, &a_) in self.y.iter().take(self.k - 1).zip(self.a.iter().skip(1)) {
            out_value = out_value + y_ * a_;
        }
        out_value = out_value / self.a[0];
        self.y.push(out_value);
        out_value
    }
}
