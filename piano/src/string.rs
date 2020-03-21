use num_traits::float::Float;
use num_traits::identities::Zero;

use super::filter::Filter;
use super::ring_buffer::RingBuffer;

pub struct DelayLine<T> {
    to_right_filters: Vec<Filter<T>>,
    to_left_filters: Vec<Filter<T>>,
    pub v_right_plus: T,
    pub v_right_minus: T,
    pub v_left_plus: T,
    pub v_left_minus: T,
    delay: usize,
    v_right_minus_history: RingBuffer<T>,
    v_left_minus_history: RingBuffer<T>,
}

impl<T: Clone + Copy + Float + Zero> DelayLine<T> {
    fn new(delay: usize, init: T) -> DelayLine<T> {
        DelayLine {
            to_right_filters: vec![],
            to_left_filters: vec![],
            v_right_plus: init,
            v_right_minus: init,
            v_left_plus: init,
            v_left_minus: init,
            delay,
            v_right_minus_history: RingBuffer::new(delay, init),
            v_left_minus_history: RingBuffer::new(delay, init),
        }
    }

    fn new_with_filters(
        delay: usize,
        init: T,
        to_right_filters: Vec<Filter<T>>,
        to_left_filters: Vec<Filter<T>>,
    ) -> DelayLine<T> {
        DelayLine {
            to_right_filters,
            to_left_filters,
            v_right_plus: init,
            v_right_minus: init,
            v_left_plus: init,
            v_left_minus: init,
            delay,
            v_right_minus_history: RingBuffer::new(delay, init),
            v_left_minus_history: RingBuffer::new(delay, init),
        }
    }

    fn do_delay(&mut self) {
        let mut next_v_right_plus = *self.v_left_minus_history.last();
        for filter in &mut self.to_right_filters {
            next_v_right_plus = filter.filter(next_v_right_plus);
        }
        self.v_right_plus = next_v_right_plus;

        let mut next_v_left_plus = *self.v_right_minus_history.last();
        for filter in &mut self.to_left_filters {
            next_v_left_plus = filter.filter(next_v_left_plus);
        }
        self.v_left_plus = next_v_left_plus;
    }

    pub fn left_update(&mut self, v_left_minus: T) {
        self.v_left_minus = v_left_minus;
        self.v_left_minus_history.push(v_left_minus);
    }

    pub fn right_update(&mut self, v_right_minus: T) {
        self.v_right_minus = v_right_minus;
        self.v_right_minus_history.push(v_right_minus);
    }
}

pub struct String<T> {
    pub delay_line_left: DelayLine<T>,
    pub delay_line_right: DelayLine<T>,
    pub impedance: T,
}

impl<T: Clone + Copy + Float + Zero> String<T> {
    fn pin_update(&mut self) {
        self.delay_line_left
            .left_update(T::zero() - self.delay_line_left.v_left_plus);
    }
}
