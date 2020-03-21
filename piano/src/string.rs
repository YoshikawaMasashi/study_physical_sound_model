use num_traits::float::Float;
use num_traits::identities::Zero;

use super::filter::Filter;
use super::ring_buffer::RingBuffer;

pub struct DelayLine<T> {
    to_right_filters: Vec<Filter<T>>,
    to_left_filters: Vec<Filter<T>>,
    pub v_right_plus: T,
    pub v_left_plus: T,
    pub next_v_left_minus: Option<T>,
    pub next_v_right_minus: Option<T>,
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
            v_left_plus: init,
            next_v_left_minus: None,
            next_v_right_minus: None,
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
            v_left_plus: init,
            next_v_left_minus: None,
            next_v_right_minus: None,
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

    fn update(&mut self) {
        if let Some(v) = self.next_v_left_minus {
            self.v_left_minus_history.push(v);
            self.next_v_left_minus = None;
        } else {
            panic!("no next_v_left_minus")
        }

        if let Some(v) = self.next_v_right_minus {
            self.v_right_minus_history.push(v);
            self.next_v_right_minus = None;
        } else {
            panic!("no next_v_right_minus")
        }
    }
}

pub struct String<T> {
    pub delay_line_left: DelayLine<T>,
    pub delay_line_right: DelayLine<T>,
    pub impedance: T,
}

impl<T: Clone + Copy + Float + Zero> String<T> {
    fn pin_update(&mut self) {
        self.delay_line_left.next_v_left_minus = Some(T::zero() - self.delay_line_left.v_left_plus);
    }
}
