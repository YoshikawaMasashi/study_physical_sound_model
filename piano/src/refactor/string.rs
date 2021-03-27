use std::cell::RefCell;
use std::rc::Rc;

use super::super::filter::Filter;
use super::super::loss::loss;
use super::super::ring_buffer::RingBuffer;
use super::super::thirian::{thirian, thirian_dispersion};

struct DelayLineNode {
    z: f32, // インピーダンス
    load: f32, // 力の単位で入っているっぽい
    a: [f32; 2], // たぶんvelocity 0: rightからleftに行く方向 1: leftからrightに行く方向
}

impl DelayLineNode {
    fn new(z: f32) -> DelayLineNode {
        DelayLineNode {
            z,
            load: 0.0,
            a: [0.0, 0.0],
        }
    }
}

struct DelayLine {
    nl: usize,
    nr: usize,
    cl: Vec<Rc<RefCell<DelayLineNode>>>, // cはconnectのl
    cr: Vec<Rc<RefCell<DelayLineNode>>>,
    zl: Vec<f32>,
    zr: Vec<f32>,
    l: Rc<RefCell<DelayLineNode>>,
    r: Rc<RefCell<DelayLineNode>>,
    loadl: f32,
    loadr: f32,
    alphalthis: f32,
    alpharthis: f32,
    alphal: Vec<f32>,
    alphar: Vec<f32>,
    d: [RingBuffer<f32>; 2],
    impedance: f32,
    left_filters: Vec<Filter<f32>>,
    right_filters: Vec<Filter<f32>>,
}

impl DelayLine {
    fn new(
        impedance: f32,
        del1: usize,
        del2: usize,
        left_filters: Vec<Filter<f32>>,
        right_filters: Vec<Filter<f32>>,
    ) -> DelayLine {
        let d = [
            RingBuffer::<f32>::new(del1, 0.0),
            RingBuffer::<f32>::new(del2, 0.0),
        ];
        let l = Rc::new(RefCell::new(DelayLineNode::new(impedance)));
        let r = Rc::new(RefCell::new(DelayLineNode::new(impedance)));

        DelayLine {
            nl: 0,
            nr: 0,
            cl: vec![],
            cr: vec![],
            zl: vec![],
            zr: vec![],
            l,
            r,
            loadl: 0.0,
            loadr: 0.0,
            alphalthis: 0.0,
            alpharthis: 0.0,
            alphal: vec![],
            alphar: vec![],
            d,
            impedance,
            left_filters,
            right_filters
        }
    }

    fn init(&mut self) {
        let mut ztot: f32 = self.l.borrow().z;
        for k in 0..self.nl {
            ztot += self.cl[k].borrow().z;
        }
        self.alphalthis = 2.0 * self.l.borrow().z / ztot;
        for k in 0..self.nl {
            self.alphal.push(2.0 * self.cl[k].borrow().z / ztot);
        }

        ztot = self.r.borrow().z;
        for k in 0..self.nr {
            ztot += self.cr[k].borrow().z;
        }
        self.alpharthis = 2.0 * self.r.borrow().z / ztot;
        for k in 0..self.nr {
            self.alphar.push(2.0 * self.cr[k].borrow().z / ztot);
        }
    }

    fn connect_left(&mut self, l: Rc<RefCell<DelayLineNode>>) {
        self.zl.push(l.borrow().z);
        self.cl.push(l);
        self.nl += 1;
    }

    fn connect_right(&mut self, r: Rc<RefCell<DelayLineNode>>) {
        self.zr.push(r.borrow().z);
        self.cr.push(r);
        self.nr += 1;
    }

    fn do_delay(&mut self) {
        let dar: f32 = *self.d[0].last();
        let dal: f32 = *self.d[1].last();
        self.l.borrow_mut().a[0] = dar;
        self.r.borrow_mut().a[1] = dal;
        self.d[0].push(self.r.borrow().a[0]);
        self.d[1].push(self.l.borrow().a[1]);
    }

    fn do_load(&mut self) {
        if self.nl > 0 {
            self.loadl += self.alphalthis * self.l.borrow().a[0];
            for k in 0..self.nl {
                self.loadl += self.cl[k].borrow().load;
                self.loadl += self.alphal[k] * self.cl[k].borrow().a[1];
            }
        }

        if self.nr > 0 {
            self.loadr += self.alpharthis * self.r.borrow().a[1];
            for k in 0..self.nr {
                self.loadr += self.cr[k].borrow().load;
                self.loadr += self.alphar[k] * self.cr[k].borrow().a[0];
            }
        }
    }

    fn update(&mut self) {
        let mut a = self.loadl - self.l.borrow().a[0];
        let filter_num = self.left_filters.len();
        for i in 0..filter_num {
            a = self.left_filters[i].filter(a);
        }
        self.l.borrow_mut().a[1] = a;

        a = self.loadr - self.r.borrow().a[1];
        let filter_num = self.right_filters.len();
        for i in 0..filter_num {
            a = self.right_filters[i].filter(a);
        }
        self.r.borrow_mut().a[0] = a;

        self.loadl = 0.0;
        self.loadr = 0.0;
    }
}

pub struct String {
    left_string: DelayLine,
    right_string: DelayLine,
    soundboard_impedance: f32
}

impl String {
    pub fn new(
        f: f32,
        fs: f32,
        inpos: f32,
        c1: f32,
        c3: f32,
        b: f32,
        z: f32,
        zb: f32, // board
    ) -> String {
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

        let mut left_string = DelayLine::new(z, del1, del1, vec![], vec![]);
        let mut right_string = DelayLine::new(z, del2, del3, left_filters, right_filters);

        left_string.connect_right(Rc::clone(&right_string.l));
        right_string.connect_left(Rc::clone(&left_string.r));

        left_string.init();
        right_string.init();

        String {
            left_string,right_string,
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

        self.left_string.do_load();
        self.right_string.do_load();

        self.left_string.update();
        self.right_string.update();

        self.right_string.r.borrow().a[1] * 2.0 * self.right_string.impedance / self.soundboard_impedance
    }
}
