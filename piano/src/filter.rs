use num_traits::identities::Zero;
use num_traits::Num;

struct RingBuffer<T> {
    n: usize,
    i: usize,
    v: Vec<T>,
}

impl<T: Clone> RingBuffer<T> {
    fn new(n: usize, init: T) -> RingBuffer<T> {
        RingBuffer {
            n,
            i: 0,
            v: vec![init; n],
        }
    }

    fn iter(&self) -> RingBufferIter<T> {
        RingBufferIter { buf: self, i: 0 }
    }

    fn push(&mut self, x: T) {
        self.v[(self.i + self.n - 1) % self.n] = x;
        self.i = (self.i + self.n - 1) % self.n;
    }
}

struct RingBufferIter<'a, T> {
    buf: &'a RingBuffer<T>,
    i: usize,
}

impl<'a, T: Copy> Iterator for RingBufferIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.i >= self.buf.n {
            None
        } else {
            let ret: &T = &self.buf.v[(self.i + self.buf.i) % self.buf.n];
            self.i += 1;
            Some(ret)
        }
    }
}

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
            k, a, b,
            x: RingBuffer::new(k, init),
            y: RingBuffer::new(k, init),
        }
    }

    fn filter(&mut self, in_value: T) -> T {
        let mut out_value: T = T::zero();
        self.x.push(in_value);
        for (&x_, &b_) in self.x.iter().zip(self.b.iter()) {
            out_value = out_value + x_ * b_;
        }
        for(&y_, &a_) in self.y.iter().take(self.k - 1).zip(self.a.iter().skip(1)) {
            out_value = out_value + y_ * a_;
        }
        out_value = out_value / self.a[0];
        self.y.push(out_value);
        out_value
    }
}

#[test]
fn ring_buffer_work() {
    let mut buf = RingBuffer::new(3, 0);
    let mut iter = buf.iter();

    assert_eq!(Some(&0), iter.next());
    assert_eq!(Some(&0), iter.next());
    assert_eq!(Some(&0), iter.next());
    assert_eq!(None, iter.next());

    buf.push(1);
    buf.push(2);
    buf.push(3);
    let mut iter = buf.iter();

    assert_eq!(Some(&3), iter.next());
    assert_eq!(Some(&2), iter.next());
    assert_eq!(Some(&1), iter.next());
    assert_eq!(None, iter.next());

    buf.push(4);
    let mut iter = buf.iter();

    assert_eq!(Some(&4), iter.next());
    assert_eq!(Some(&3), iter.next());
    assert_eq!(Some(&2), iter.next());
    assert_eq!(None, iter.next());
}