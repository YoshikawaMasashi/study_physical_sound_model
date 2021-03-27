use std::cell::RefCell;
use std::rc::Rc;

use super::dwgs::DWGs;
use super::hammer::Hammer;

pub struct Piano {
    v0: f32,
    samples: usize,
    sample: usize,
    fs: f32,
    t: f32,
    dt: f32,
    z: f32,
    zb: f32,
    zh: f32,
    f: f32,

    nstrings: usize,
    string: Vec<DWGs>,
    hammers: Vec<Hammer>,
}

impl Piano {
    pub fn new(note: usize, fs: f32, v0: f32, samples: usize) -> Piano {
        let f: f32 = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);

        let f0 = 27.5;
        let rho = 7850.0;
        let p = 2.0 + 1.0 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        let m = 0.06 - 0.058 * f32::powf(f32::ln(f / f0) / f32::ln(4192.0 / f0), 0.1);
        let K = 40.0 / f32::powf(0.7e-3, p);
        let mut l = 1.4 - 1.32 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        l = 0.04 + 1.4 / (1.0 + f32::exp(-3.4 + 1.4 * f32::ln(f / f0)));
        let r = 0.002 * f32::powf(1.0 + 0.6 * f32::ln(f / f0), -1.4);
        let rho_l = std::f32::consts::PI * r * r * rho;
        let t = (2.0 * l * f) * (2.0 * l * f) * rho_l;
        let e = 200.0e9;
        let flong = f32::sqrt(e / rho) / (2.0 * l);

        let rcore = if (r < 0.0006) { r } else { 0.0006 };
        let b = (std::f32::consts::PI * std::f32::consts::PI * std::f32::consts::PI)
            * e
            * (rcore * rcore * rcore * rcore)
            / (4.0 * l * l * t);
        let hp = 1.0 / 7.0;
        let z = f32::sqrt(t * rho_l);
        let zb = 4000.0;
        let zh = 0.0;

        println!(
            "f = {}, r = {} mm, L = {}, T = {}, hammer = {}, Z = {}, K = {}, B = {}",
            f,
            1000.0 * r,
            l,
            t,
            hp,
            z,
            K,
            b,
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
        const TUNE: [f32; 3] = [1.0, 1.0003, 0.9996];
        for k in 0..nstrings {
            string.push(DWGs::new(
                f * TUNE[k],
                fs,
                hp,
                c1,
                c3,
                b,
                z,
                zb + (nstrings - 1) as f32 * z,
                zh,
            ));
        }

        let a = -1.0 / 4.0;
        let alpha = 0.1e-4 * f32::ln(f / f0) / f32::ln(4192.0 / f0);

        let mut hammers: Vec<Hammer> = vec![];
        for k in 0..nstrings {
            hammers.push(Hammer::new(f, fs, m, K, p, z, alpha, v0));
        }
        Piano {
            v0,
            samples,
            sample: 0,
            fs,
            t: 0.0,
            dt: 1.0 / fs,
            z,
            zb,
            zh,
            f,

            nstrings,
            string,
            hammers,
        }
    }

    pub fn go(&mut self) -> f32 {
        let mut hloads: Vec<f32> = vec![];
        for k in 0..self.nstrings {
            hloads.push(self.hammers[k].load(self.string[k].input_velocity() as f32));
        }

        let mut load = 0.0;
        for k in 0..self.nstrings {
            load += (2.0 * self.z * self.string[k].go_hammer(hloads[k] / (2.0 * self.z)))
                / (self.z * self.nstrings as f32 + self.zb);
        }

        let mut output = 0.0;
        for k in 0..self.nstrings {
            output += self.string[k].go_soundboard(load);
        }

        output
    }
}
