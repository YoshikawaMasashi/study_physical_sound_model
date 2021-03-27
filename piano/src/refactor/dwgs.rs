use std::cell::RefCell;
use std::rc::Rc;

use super::super::filter::Filter;
use super::super::loss::loss;
use super::super::ring_buffer::RingBuffer;
use super::super::thirian::{thirian, thirian_dispersion};

struct DWGNode {
    z: f32,
    load: f32,
    a: [f32; 2],
}

impl DWGNode {
    fn new(z: f32) -> DWGNode {
        DWGNode {
            z,
            load: 0.0,
            a: [0.0, 0.0],
        }
    }
}

struct DWG {
    del1: usize,
    del2: usize,
    nl: usize,
    nr: usize,
    pl: Vec<usize>,
    pr: Vec<usize>,
    cl: Vec<Rc<RefCell<DWGNode>>>,
    cr: Vec<Rc<RefCell<DWGNode>>>,
    l: Rc<RefCell<DWGNode>>,
    r: Rc<RefCell<DWGNode>>,
    loadl: f32,
    loadr: f32,
    al: Vec<f32>,
    ar: Vec<f32>,
    alphalthis: f32,
    alpharthis: f32,
    alphal: Vec<f32>,
    alphar: Vec<f32>,
    d: [RingBuffer<f32>; 2],
    filters: Rc<RefCell<DWGFilters>>,
    commute: bool,
}

impl DWG {
    fn new(
        z: f32,
        del1: usize,
        del2: usize,
        commute: bool,
        filters: Rc<RefCell<DWGFilters>>,
    ) -> DWG {
        let d = [
            RingBuffer::<f32>::new(del1, 0.0),
            RingBuffer::<f32>::new(del2, 0.0),
        ];
        let l = Rc::new(RefCell::new(DWGNode::new(z)));
        let r = Rc::new(RefCell::new(DWGNode::new(z)));

        DWG {
            del1,
            del2,
            nl: 0,
            nr: 0,
            pl: vec![],
            pr: vec![],
            cl: vec![],
            cr: vec![],
            l,
            r,
            loadl: 0.0,
            loadr: 0.0,
            al: vec![],
            ar: vec![],
            alphalthis: 0.0,
            alpharthis: 0.0,
            alphal: vec![],
            alphar: vec![],
            d,
            filters,
            commute,
        }
    }

    fn init(&mut self) {
        let mut ztot: f32 = 0.0;

        ztot = self.l.borrow().z;
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

    fn connect_left_with_pol(&mut self, l: Rc<RefCell<DWGNode>>, polarity: usize) {
        self.cl.push(l);
        self.pl.push(polarity);
        self.nl += 1;
    }

    fn connect_right_with_pol(&mut self, r: Rc<RefCell<DWGNode>>, polarity: usize) {
        self.cr.push(r);
        self.pr.push(polarity);
        self.nr += 1;
    }

    fn connect_left(&mut self, l: Rc<RefCell<DWGNode>>) {
        self.connect_left_with_pol(l, 0);
    }

    fn connect_right(&mut self, r: Rc<RefCell<DWGNode>>) {
        self.connect_right_with_pol(r, 0);
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
        if (self.nl == 0) {
            self.loadl = 0.0;
        } else {
            self.loadl = self.alphalthis * self.l.borrow().a[0];
            for k in 0..self.nl {
                let polarity = if self.pl[k] != 0 { 0 } else { 1 };
                self.loadl += self.cl[k].borrow().load;
                self.loadl += self.alphal[k] * self.cl[k].borrow().a[polarity];
            }
        }

        if (self.nr == 0) {
            self.loadr = 0.0;
        } else {
            self.loadr = self.alpharthis * self.r.borrow().a[1];
            for k in 0..self.nr {
                let polarity = if self.pr[k] != 0 { 1 } else { 0 };
                self.loadr += self.cr[k].borrow().load;
                self.loadr += self.alphar[k] * self.cr[k].borrow().a[polarity];
            }
        }
    }

    fn update(&mut self) {
        let mut a = self.loadl - self.l.borrow().a[0];
        if (self.commute) {
            let M = self.filters.borrow().dispersion.len();
            for m in 0..M {
                a = self.filters.borrow_mut().dispersion[m].filter(a);
            }
        }
        self.l.borrow_mut().a[1] = a;

        a = self.loadr - self.r.borrow().a[1];
        if (self.commute) {
            a = self.filters.borrow_mut().lowpass.filter(a);
            a = self.filters.borrow_mut().fracdelay.filter(a);
        }
        self.r.borrow_mut().a[0] = a;
    }
}

pub struct DWGFilters {
    dispersion: Vec<Filter<f32>>,
    lowpass: Filter<f32>,
    fracdelay: Filter<f32>,
}

pub struct DWGs {
    filters: Rc<RefCell<DWGFilters>>,
    M: usize,
    d: [DWG; 4],
}

impl DWGs {
    pub fn new(
        f: f32,
        Fs: f32,
        inpos: f32,
        c1: f32,
        c3: f32,
        B: f32,
        Z: f32,
        Zb: f32,
        Zh: f32,
    ) -> DWGs {
        let deltot = Fs / f;
        let mut del1 = (inpos * 0.5 * deltot) as usize;
        if (del1 < 2) {
            del1 = 1;
        }

        let M = if (f > 400.0) { 1 } else { 4 };
        let mut dispersion = vec![];
        for m in 0..M {
            dispersion.push(thirian_dispersion(B, f, M));
        }
        let dispersion_delay = M as f32 * dispersion[0].groupdelay(f, Fs);
        let lowpass = loss(f, Fs, c1, c3);
        let lowpass_delay = lowpass.groupdelay(f, Fs);

        let mut del2 = (0.5 * (deltot - 2.0 * (del1 as f32)) - dispersion_delay) as usize;
        let mut del3 = (0.5 * (deltot - 2.0 * (del1 as f32)) - lowpass_delay - 5.0) as usize;
        if (del2 < 2) {
            del2 = 1;
        }
        if (del3 < 2) {
            del3 = 1;
        }

        let D = (deltot
            - (del1 as f32
                + del1 as f32
                + del2 as f32
                + del3 as f32
                + dispersion_delay
                + lowpass_delay));
        println!("D: {}", D);
        let fracdelay = thirian(D, D as usize);
        let tuning_delay = fracdelay.groupdelay(f, Fs);

        println!("total delay = {}/{}, leftdel = {}/{}, rightdel = {}/{}, dispersion delay = {}, lowpass delay = {}, fractional delay = {}/{}",
            del1 as f32+del1 as f32+del2 as f32+del3 as f32+dispersion_delay+lowpass_delay+tuning_delay,deltot, del1, del1, del2, del3, dispersion_delay, lowpass_delay, tuning_delay,D
        );

        let filters = Rc::new(RefCell::new(DWGFilters {
            dispersion,
            lowpass,
            fracdelay,
        }));

        let mut d0 = DWG::new(Z, del1, del1, false, Rc::clone(&filters));
        let mut d1 = DWG::new(Z, del2, del3, true, Rc::clone(&filters));
        let mut d2 = DWG::new(Zb, 0, 0, false, Rc::clone(&filters));
        let mut d3 = DWG::new(Zh, 0, 0, false, Rc::clone(&filters));

        d0.connect_right(Rc::clone(&d1.l));
        d1.connect_left(Rc::clone(&d0.r));
        d1.connect_right(Rc::clone(&d2.l));
        d2.connect_left(Rc::clone(&d1.r));

        d0.connect_right(Rc::clone(&d3.l));
        d1.connect_left(Rc::clone(&d3.l));
        d3.connect_left(Rc::clone(&d0.r));
        d3.connect_left(Rc::clone(&d1.l));

        d0.init();
        d1.init();
        d2.init();
        d3.init();

        DWGs {
            filters: Rc::clone(&filters),
            M,
            d: [d0, d1, d2, d3],
        }
    }

    pub fn input_velocity(&self) -> f32 {
        self.d[1].l.borrow().a[0] + self.d[0].r.borrow().a[1]
    }

    pub fn go_hammer(&mut self, load: f32) -> f32 {
        self.d[3].l.borrow_mut().load = load;
        for k in 0..2 {
            self.d[k].do_delay();
        }
        self.d[1].r.borrow().a[1]
    }

    pub fn go_soundboard(&mut self, load: f32) -> f32 {
        self.d[2].l.borrow_mut().load = load;
        for k in 0..3 {
            self.d[k].do_load();
        }

        for k in 0..3 {
            self.d[k].update();
        }

        self.d[2].l.borrow().a[1]
    }
}
