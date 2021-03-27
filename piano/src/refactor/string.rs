use std::cell::RefCell;
use std::rc::Rc;

use super::super::filter::Filter;
use super::super::loss::loss;
use super::super::ring_buffer::RingBuffer;
use super::super::thirian::{thirian, thirian_dispersion};

struct StringNode {
    a: [f32; 2], // たぶんvelocity 0: rightからleftに行く方向 1: leftからrightに行く方向
}

impl StringNode {
    fn new() -> StringNode {
        StringNode { a: [0.0, 0.0] }
    }
}

struct String {
    l: Rc<RefCell<StringNode>>,
    r: Rc<RefCell<StringNode>>,
    loadl: f32,
    loadr: f32,
    d: [RingBuffer<f32>; 2],
    impedance: f32,
    left_filters: Vec<Filter<f32>>,
    right_filters: Vec<Filter<f32>>,
}

impl String {
    fn new(
        impedance: f32,
        del1: usize,
        del2: usize,
        left_filters: Vec<Filter<f32>>,
        right_filters: Vec<Filter<f32>>,
    ) -> String {
        let d = [
            RingBuffer::<f32>::new(del1, 0.0),
            RingBuffer::<f32>::new(del2, 0.0),
        ];
        let l = Rc::new(RefCell::new(StringNode::new()));
        let r = Rc::new(RefCell::new(StringNode::new()));

        String {
            l,
            r,
            loadl: 0.0,
            loadr: 0.0,
            d,
            impedance,
            left_filters,
            right_filters,
        }
    }

    fn do_delay(&mut self) {
        let mut dar: f32 = *self.d[0].last();
        let mut dal: f32 = *self.d[1].last();

        let filter_num = self.left_filters.len();
        for i in 0..filter_num {
            dar = self.left_filters[i].filter(dar);
        }
        let filter_num = self.right_filters.len();
        for i in 0..filter_num {
            dal = self.right_filters[i].filter(dal);
        }

        self.l.borrow_mut().a[0] = dar;
        self.r.borrow_mut().a[1] = dal;
        self.d[0].push(self.r.borrow().a[0]);
        self.d[1].push(self.l.borrow().a[1]);
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

        let total_delay = deltot
            - (del1 as f32
                + del1 as f32
                + del2 as f32
                + del3 as f32
                + dispersion_delay
                + lowpass_delay);
        println!("D: {}", total_delay);
        let fracdelay = thirian(total_delay, total_delay as usize);
        right_filters.push(thirian(total_delay, total_delay as usize));
        let tuning_delay = fracdelay.groupdelay(f, fs);

        println!("total delay = {}/{}, leftdel = {}/{}, rightdel = {}/{}, dispersion delay = {}, lowpass delay = {}, fractional delay = {}/{}",
            del1 as f32+del1 as f32+del2 as f32+del3 as f32+dispersion_delay+lowpass_delay+tuning_delay,deltot, del1, del1, del2, del3, dispersion_delay, lowpass_delay, tuning_delay, total_delay
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
        self.right_string.l.borrow().a[0] + self.left_string.r.borrow().a[1]
    }

    pub fn go_hammer(&mut self, load: f32) -> f32 {
        self.left_string.loadr += load;
        self.right_string.loadl += load;

        self.left_string.do_delay();
        self.right_string.do_delay();
        self.right_string.r.borrow().a[1]
    }

    pub fn go_soundboard(&mut self, load: f32) -> f32 {
        self.right_string.loadr += load;

        self.left_string.loadr += self.left_string.r.borrow().a[1];
        self.left_string.loadr += self.right_string.l.borrow().a[0];

        self.right_string.loadl += self.right_string.l.borrow().a[0];
        self.right_string.loadl += self.left_string.r.borrow().a[1];

        let a = self.left_string.loadl - self.left_string.l.borrow().a[0];
        self.left_string.l.borrow_mut().a[1] = a;
        let a = self.left_string.loadr - self.left_string.r.borrow().a[1];
        self.left_string.r.borrow_mut().a[0] = a;

        let a = self.right_string.loadl - self.right_string.l.borrow().a[0];
        self.right_string.l.borrow_mut().a[1] = a;
        let a = self.right_string.loadr - self.right_string.r.borrow().a[1];
        self.right_string.r.borrow_mut().a[0] = a;

        self.right_string.loadl = 0.0;
        self.right_string.loadr = 0.0;
        self.left_string.loadr = 0.0;

        self.right_string.r.borrow().a[1] * 2.0 * self.right_string.impedance
            / self.soundboard_impedance
    }
}
