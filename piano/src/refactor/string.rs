use std::cell::RefCell;
use std::rc::Rc;

use super::super::filter::Filter;
use super::super::loss::loss;
use super::super::ring_buffer::RingBuffer;
use super::super::thirian::{thirian, thirian_dispersion};

struct DelayLine {
    history_buffer: RingBuffer<f32>,
    filters: Vec<Filter<f32>>,
}

impl DelayLine {
    fn new(size: usize, filters: Vec<Filter<f32>>) -> Self {
        DelayLine{
            history_buffer: RingBuffer::<f32>::new(size, 0.0),
            filters
        }
    }

    fn do_delay(&mut self, input: f32) -> f32 {
        let mut x: f32 = *self.history_buffer.last();

        let filter_num = self.filters.len();
        for i in 0..filter_num {
            x = self.filters[i].filter(x);
        }

        self.history_buffer.push(input);

        x
    }
}

struct String {
    v_at_left_to_left: f32, // l[0]
    v_at_right_to_left: f32, // r[0]
    v_at_left_to_right: f32, // l[1]
    v_at_right_to_right: f32, // r[1]
    loadl: f32,
    loadr: f32,
    impedance: f32,
    to_left_delay_line: DelayLine,
    to_right_delay_line: DelayLine,
}

impl String {
    fn new(
        impedance: f32,
        del1: usize,
        del2: usize,
        left_filters: Vec<Filter<f32>>,
        right_filters: Vec<Filter<f32>>,
    ) -> String {
        String {
            v_at_left_to_left: 0.0,
            v_at_right_to_left: 0.0,
            v_at_left_to_right: 0.0,
            v_at_right_to_right: 0.0,
            loadl: 0.0,
            loadr: 0.0,
            impedance,
            to_left_delay_line: DelayLine::new(del1, left_filters),
            to_right_delay_line: DelayLine::new(del2, right_filters),
        }
    }

    fn do_delay(&mut self) {
        self.v_at_left_to_left = self.to_left_delay_line.do_delay(self.v_at_right_to_left);
        self.v_at_right_to_right = self.to_right_delay_line.do_delay(self.v_at_left_to_right);
    }
}

pub struct StringHammerSoundboard {
    left_string: String,
    right_string: String,
    // string_impedance: f32,
    soundboard_impedance: f32,
}

impl StringHammerSoundboard {
    pub fn new(
        f: f32,
        fs: f32,
        inpos: f32,
        c1: f32,
        c3: f32,
        b: f32,
        z: f32,
        zb: f32, // board
    ) -> StringHammerSoundboard {
        let deltot = fs / f;
        let mut del1 = (inpos * 0.5 * deltot) as usize;
        if del1 < 2 {
            del1 = 1;
        }

        let mut left_filters = vec![];
        let mut right_filters = vec![];

        let m = if f > 400.0 { 1 } else { 4 };
        let mut dispersion = vec![];
        for _ in 0..m {
            dispersion.push(thirian_dispersion(b, f, m));
            left_filters.push(thirian_dispersion(b, f, m));
        }
        let dispersion_delay = m as f32 * dispersion[0].groupdelay(f, fs);
        let lowpass = loss(f, c1, c3);
        right_filters.push(loss(f, c1, c3));
        let lowpass_delay = lowpass.groupdelay(f, fs);

        let mut del2 = (0.5 * (deltot - 2.0 * (del1 as f32)) - dispersion_delay) as usize;
        let mut del3 = (0.5 * (deltot - 2.0 * (del1 as f32)) - lowpass_delay - 5.0) as usize;
        if del2 < 2 {
            del2 = 1;
        }
        if del3 < 2 {
            del3 = 1;
        }

        let fracdelay_delay = deltot
            - (del1 as f32
                + del1 as f32
                + del2 as f32
                + del3 as f32
                + dispersion_delay
                + lowpass_delay);
        let fracdelay = thirian(fracdelay_delay, fracdelay_delay as usize);
        right_filters.push(thirian(fracdelay_delay, fracdelay_delay as usize));
        let tuning_delay = fracdelay.groupdelay(f, fs);

        println!("total delay = {}/{}, leftdel = {}/{}, rightdel = {}/{}, dispersion delay = {}, lowpass delay = {}, fractional delay = {}/{}",
            del1 as f32+del1 as f32+del2 as f32+del3 as f32+dispersion_delay+lowpass_delay+tuning_delay,deltot, del1, del1, del2, del3, dispersion_delay, lowpass_delay, tuning_delay, fracdelay_delay
        );

        let left_string = String::new(z, del1, del1, vec![], vec![]);
        let right_string = String::new(z, del2, del3, left_filters, right_filters);

        StringHammerSoundboard {
            left_string,
            right_string,
            // string_impedance: z,
            soundboard_impedance: zb,
        }
    }

    pub fn input_velocity(&self) -> f32 {
        self.right_string.v_at_left_to_left + self.left_string.v_at_right_to_right
    }

    pub fn go_hammer(&mut self, load: f32) -> f32 {
        self.left_string.loadr += load;
        self.right_string.loadl += load;

        self.left_string.do_delay();
        self.right_string.do_delay();
        self.right_string.v_at_right_to_right
    }

    pub fn go_soundboard(&mut self, load: f32) -> f32 {
        self.right_string.loadr += load;

        self.left_string.loadr += self.left_string.v_at_right_to_right;
        self.left_string.loadr += self.right_string.v_at_left_to_left;

        self.right_string.loadl += self.right_string.v_at_left_to_left;
        self.right_string.loadl += self.left_string.v_at_right_to_right;

        let a = self.left_string.loadl - self.left_string.v_at_left_to_left;
        self.left_string.v_at_left_to_right = a;
        let a = self.left_string.loadr - self.left_string.v_at_right_to_right;
        self.left_string.v_at_right_to_left = a;

        let a = self.right_string.loadl - self.right_string.v_at_left_to_left;
        self.right_string.v_at_left_to_right = a;
        let a = self.right_string.loadr - self.right_string.v_at_right_to_right;
        self.right_string.v_at_right_to_left = a;

        self.right_string.loadl = 0.0;
        self.right_string.loadr = 0.0;
        self.left_string.loadr = 0.0;

        self.right_string.v_at_right_to_right * 2.0 * self.right_string.impedance
            / self.soundboard_impedance
    }
}
