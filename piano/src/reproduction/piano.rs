use std::cell::RefCell;
use std::rc::Rc;

use super::dwgs::DWGs;
use super::hammer::Hammer;

pub struct Piano {
    v0: f32,
    samples: usize,
    sample: usize,
    Fs: f32,
    t: f32,
    dt: f32,
    Z: f32,
    Zb: f32,
    Zh: f32,
    f: f32,

    nstrings: usize,
    string: Vec<DWGs>,
    hammer: Hammer,
}

impl Piano {
    pub fn new(note: usize, Fs: f32, v0: f32, samples: usize) -> Piano {
        let f: f32 = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);

        let f0 = 27.5;
        let rho = 7850.0;
        let p = 2.0 + 1.0 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        let m = 0.06 - 0.058 * f32::powf(f32::ln(f / f0) / f32::ln(4192.0 / f0), 0.1);
        let K = 40.0 / f32::powf(0.7e-3, p);
        let mut L = 1.4 - 1.32 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        L = 0.04 + 1.4 / (1.0 + f32::exp(-3.4 + 1.4 * f32::ln(f / f0)));
        let r = 0.002 * f32::powf(1.0 + 0.6 * f32::ln(f / f0), -1.4);
        let rhoL = std::f32::consts::PI * r * r * rho;
        let T = (2.0 * L * f) * (2.0 * L * f) * rhoL;
        let E = 200.0e9;
        let flong = f32::sqrt(E / rho) / (2.0 * L);

        let rcore = if (r < 0.0006) { r } else { 0.0006 };
        let B = (std::f32::consts::PI * std::f32::consts::PI * std::f32::consts::PI)
            * E
            * (rcore * rcore * rcore * rcore)
            / (4.0 * L * L * T);
        let hp = 1.0 / 7.0;
        let Z = f32::sqrt(T * rhoL);
        let Zb = 4000.0;
        let Zh = 0.0;

        println!(
            "f = {}, r = {} mm, L = {}, T = {}, hammer = {}, Z = {}, K = {}, B = {}",
            f,
            1000.0 * r,
            L,
            T,
            hp,
            Z,
            K,
            B,
        );

        let c1 = 0.25;
        let c3 = 5.85;
        let nstrings: usize = if (note < 31) {
            1
        } else if (note < 41) {
            2
        } else {
            3
        };
        let c1b = 20.0;
        let c3b = 20.0;
        let mut string = vec![];
        let TUNE = [1.0, 1.0003, 0.9996];
        for k in 0..nstrings {
            string.push(DWGs::new(
                f * TUNE[k],
                Fs,
                hp,
                c1,
                c3,
                B,
                Z,
                Zb + (nstrings - 1) as f32 * Z,
                Zh,
            ));
        }

        let a = -1.0 / 4.0;
        let alpha = 0.1e-4 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        let hammer = Hammer::new(f, Fs, m, K, p, Z, alpha, v0);
        Piano {
            v0,
            samples,
            sample: 0,
            Fs,
            t: 0.0,
            dt: 1.0 / Fs,
            Z,
            Zb,
            Zh,
            f,

            nstrings,
            string,
            hammer,
        }
    }

    pub fn go(&mut self) -> f32 {
        let mut vstring: f32 = 0.0;
        for k in 0..self.nstrings {
            vstring += self.string[k].input_velocity();
        }
        let hload = self.hammer.load(self.t, vstring / self.nstrings as f32);

        let mut load = 0.0;
        for k in 0..self.nstrings {
            load += (2.0 * self.Z * self.string[k].go_hammer(hload / (2.0 * self.Z)))
                / (self.Z * self.nstrings as f32 + self.Zb);
        }

        let mut output = 0.0;
        for k in 0..self.nstrings {
            output += self.string[k].go_soundboard(load);
        }

        output
    }
}
