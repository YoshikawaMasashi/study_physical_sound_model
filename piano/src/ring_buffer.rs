pub struct RingBuffer<T> {
    n: usize,
    i: usize,
    v: Vec<T>,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(n: usize, init: T) -> RingBuffer<T> {
        RingBuffer {
            n,
            i: 0,
            v: vec![init; n],
        }
    }

    pub fn iter(&self) -> RingBufferIter<T> {
        RingBufferIter { buf: self, i: 0 }
    }

    pub fn push(&mut self, x: T) {
        self.v[(self.i + self.n - 1) % self.n] = x;
        self.i = (self.i + self.n - 1) % self.n;
    }

    pub fn last(&self) -> &T {
        &self.v[(self.i + self.n - 1) % self.n]
    }

    // pub fn first(&self) -> &T {
    //     &self.v[self.i]
    // }
}

pub struct RingBufferIter<'a, T> {
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

#[test]
fn ring_buffer_work() {
    let mut buf = RingBuffer::new(3, 0);
    let mut iter = buf.iter();

    assert_eq!(Some(&0), iter.next());
    assert_eq!(Some(&0), iter.next());
    assert_eq!(Some(&0), iter.next());
    assert_eq!(None, iter.next());

    assert_eq!(&0, buf.last());

    buf.push(1);
    buf.push(2);
    buf.push(3);
    let mut iter = buf.iter();

    assert_eq!(Some(&3), iter.next());
    assert_eq!(Some(&2), iter.next());
    assert_eq!(Some(&1), iter.next());
    assert_eq!(None, iter.next());

    assert_eq!(&1, buf.last());

    buf.push(4);
    let mut iter = buf.iter();

    assert_eq!(Some(&4), iter.next());
    assert_eq!(Some(&3), iter.next());
    assert_eq!(Some(&2), iter.next());
    assert_eq!(None, iter.next());

    assert_eq!(&2, buf.last());
}
