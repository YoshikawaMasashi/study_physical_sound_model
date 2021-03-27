use num_traits::cast::ToPrimitive;
use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::super::filter::Filter;
use super::super::loss::loss;
use super::super::ring_buffer::RingBuffer;
use super::super::thirian::{thirian, thirian_dispersion};

pub struct DelayLine<T> {
    to_right_filters: Vec<Filter<T>>,
    to_left_filters: Vec<Filter<T>>,
    pub v_right_plus: T,
    pub v_left_plus: T,
    pub next_v_left_minus: Option<T>,
    pub next_v_right_minus: Option<T>,
    to_right_delay: usize,
    to_left_delay: usize,
    pub v_right_minus_history: RingBuffer<T>,
    pub v_left_minus_history: RingBuffer<T>,
}

impl<T: Clone + Copy + Float + Zero + FloatConst> DelayLine<T> {
    fn new(to_right_delay: usize, to_left_delay: usize, init: T) -> DelayLine<T> {
        DelayLine {
            to_right_filters: vec![],
            to_left_filters: vec![],
            v_right_plus: init,
            v_left_plus: init,
            next_v_left_minus: None,
            next_v_right_minus: None,
            to_right_delay,
            to_left_delay,
            v_right_minus_history: RingBuffer::new(to_left_delay, init),
            v_left_minus_history: RingBuffer::new(to_right_delay, init),
        }
    }

    fn new_with_filters(
        to_right_delay: usize,
        to_left_delay: usize,
        init: T,
        to_right_filters: Vec<Filter<T>>,
        to_left_filters: Vec<Filter<T>>,
    ) -> DelayLine<T> {
        DelayLine {
            to_right_filters,
            to_left_filters,
            v_right_plus: init,
            v_left_plus: init,
            next_v_left_minus: None,
            next_v_right_minus: None,
            to_right_delay,
            to_left_delay,
            v_right_minus_history: RingBuffer::new(to_left_delay, init),
            v_left_minus_history: RingBuffer::new(to_right_delay, init),
        }
    }

    pub fn do_delay(&mut self) {
        let next_v_right_plus = *self.v_left_minus_history.last();
        self.v_right_plus = next_v_right_plus;

        let next_v_left_plus = *self.v_right_minus_history.last();
        self.v_left_plus = next_v_left_plus;
    }

    pub fn update(&mut self) {
        if let Some(mut v) = self.next_v_left_minus {
            for filter in &mut self.to_right_filters {
                v = filter.filter(v);
            }
            self.v_left_minus_history.push(v);
            self.next_v_left_minus = None;
        } else {
            panic!("no next_v_left_minus")
        }

        if let Some(mut v) = self.next_v_right_minus {
            for filter in &mut self.to_left_filters {
                v = filter.filter(v);
            }
            self.v_right_minus_history.push(v);
            self.next_v_right_minus = None;
        } else {
            panic!("no next_v_right_minus")
        }
    }
}
