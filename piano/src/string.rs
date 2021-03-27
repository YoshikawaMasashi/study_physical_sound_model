use num_traits::cast::ToPrimitive;
use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::filter::{empty_filter, Filter};
use super::loss::loss;
use super::ring_buffer::RingBuffer;
use super::thirian::{thirian, thirian_dispersion};

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
        let mut next_v_right_plus = *self.v_left_minus_history.last();
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

pub struct String<T> {
    pub delay_line_left: DelayLine<T>,
    pub delay_line_right: DelayLine<T>,
    pub impedance: T,
}

impl<T: Clone + Copy + Float + Zero + ToPrimitive + FloatConst + std::fmt::Display> String<T> {
    pub fn new(
        note_frequency: T,
        sample_frequency: T,
        inpos: T,
        c1: T,
        c3: T,
        B: T,
        impedance: T,
    ) -> String<T> {
        let total_delay: T = sample_frequency / note_frequency;
        let left_line_delay: T = inpos * total_delay * T::from(0.5).unwrap();
        let right_line_delay = total_delay * T::from(0.5).unwrap() - left_line_delay;
        let left_line_delay: usize = left_line_delay.to_usize().unwrap();
        let right_line_delay: usize = right_line_delay.to_usize().unwrap();

        let mut dispersion: Vec<Filter<T>> = vec![];
        let mut dispersion_delay = T::from(0).unwrap();
        let M: usize = if (note_frequency > T::from(400).unwrap()) {
            1
        } else {
            4
        };
        println!("B: {}, f: {}, M: {}", B, note_frequency, M);
        for m in 0..M {
            let dispersion_ = thirian_dispersion(B, note_frequency, M);
            dispersion_delay =
                dispersion_delay + dispersion_.groupdelay(note_frequency, sample_frequency);
            dispersion.push(dispersion_);
        }

        let lowpass = loss(note_frequency, sample_frequency, c1, c3);
        let lowpass_delay = lowpass.groupdelay(note_frequency, sample_frequency);

        let right_line_delay_to_right = total_delay / T::from(2).unwrap()
            - T::from(left_line_delay).unwrap()
            - dispersion_delay;
        let right_line_delay_to_right = right_line_delay_to_right.to_usize().unwrap();
        let right_line_delay_to_left = total_delay / T::from(2).unwrap()
            - T::from(left_line_delay).unwrap()
            - lowpass_delay
            - T::from(5).unwrap();
        let right_line_delay_to_left = right_line_delay_to_left.to_usize().unwrap();

        let tuning: T = (total_delay
            - (T::from(left_line_delay).unwrap()
                + T::from(left_line_delay).unwrap()
                + T::from(right_line_delay_to_right).unwrap()
                + T::from(right_line_delay_to_left).unwrap()
                + dispersion_delay
                + lowpass_delay));
        let fractional = thirian(tuning, tuning.to_usize().unwrap());
        let tuningdelay = fractional.groupdelay(note_frequency, sample_frequency);
        println!("D: {}", tuning.to_f64().unwrap());

        println!(
            "total delay = {}/{}, left delay = {} / {}, right delay = {} / {}, dispersion_delay = {}, lowpass delay = {}, fractional delay = {}/{}",
            (T::from(left_line_delay).unwrap()
                + T::from(left_line_delay).unwrap()
                + T::from(right_line_delay_to_right).unwrap()
                + T::from(right_line_delay_to_left).unwrap()
                + dispersion_delay
                + lowpass_delay + tuningdelay), total_delay,
            left_line_delay, left_line_delay,
            right_line_delay_to_right, right_line_delay_to_left,
            dispersion_delay, lowpass_delay, tuningdelay, tuning
        );

        String {
            delay_line_left: DelayLine::new(left_line_delay, left_line_delay, T::from(0).unwrap()),
            delay_line_right: DelayLine::new_with_filters(
                right_line_delay_to_right,
                right_line_delay_to_left,
                T::from(0).unwrap(),
                dispersion,
                vec![lowpass, fractional],
            ),
            impedance,
        }
    }

    pub fn pin_update(&mut self) {
        self.delay_line_left.next_v_left_minus = Some(T::zero() - self.delay_line_left.v_left_plus);
    }

    pub fn do_delay(&mut self) {
        self.delay_line_left.do_delay();
        self.delay_line_right.do_delay();
    }

    pub fn update(&mut self) {
        self.delay_line_left.update();
        self.delay_line_right.update();
    }
}
