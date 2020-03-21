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
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.i >= self.buf.n {
            None
        } else {
            let ret: T = self.buf.v[(self.i + self.buf.i) % self.buf.n];
            self.i += 1;
            Some(ret)
        }
    }
}

pub struct Filter {}

#[test]
fn ring_buffer_work() {
    let mut buf = RingBuffer::new(3, 0);
    let mut iter = buf.iter();

    assert_eq!(Some(0), iter.next());
    assert_eq!(Some(0), iter.next());
    assert_eq!(Some(0), iter.next());
    assert_eq!(None, iter.next());

    buf.push(1);
    buf.push(2);
    buf.push(3);
    let mut iter = buf.iter();

    assert_eq!(Some(3), iter.next());
    assert_eq!(Some(2), iter.next());
    assert_eq!(Some(1), iter.next());
    assert_eq!(None, iter.next());

    buf.push(4);
    let mut iter = buf.iter();

    assert_eq!(Some(4), iter.next());
    assert_eq!(Some(3), iter.next());
    assert_eq!(Some(2), iter.next());
    assert_eq!(None, iter.next());
}