use super::hammer::Hammer;
use super::loss::loss;
use super::string::String;
use super::thirian::{thirian, thirian_dispersion};

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

pub struct Piano {
    z: f32,
    zb: f32,

    nstrings: usize,
    strings: Vec<StringHammerSoundboard>,
    hammers: Vec<Hammer>,
}

impl Piano {
    pub fn new(note: usize, fs: f32, v0: f32) -> Piano {
        let f: f32 = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);

        let f0 = 27.5;
        let rho = 7850.0;
        let p = 2.0 + 1.0 * f32::ln(f / f0) / f32::ln(4192.0 / f0);
        let m = 0.06 - 0.058 * f32::powf(f32::ln(f / f0) / f32::ln(4192.0 / f0), 0.1);
        let k = 40.0 / f32::powf(0.7e-3, p);
        let l = 0.04 + 1.4 / (1.0 + f32::exp(-3.4 + 1.4 * f32::ln(f / f0)));
        let r = 0.002 * f32::powf(1.0 + 0.6 * f32::ln(f / f0), -1.4);
        let rho_l = std::f32::consts::PI * r * r * rho;
        let t = (2.0 * l * f) * (2.0 * l * f) * rho_l;
        let e = 200.0e9;

        let rcore = if r < 0.0006 { r } else { 0.0006 };
        let b = (std::f32::consts::PI * std::f32::consts::PI * std::f32::consts::PI)
            * e
            * (rcore * rcore * rcore * rcore)
            / (4.0 * l * l * t);
        let hp = 1.0 / 7.0;
        let z = f32::sqrt(t * rho_l);
        let zb = 4000.0;

        println!(
            "f = {}, r = {} mm, L = {}, T = {}, hammer = {}, Z = {}, k = {}, B = {}",
            f,
            1000.0 * r,
            l,
            t,
            hp,
            z,
            k,
            b,
        );

        let c1 = 0.25;
        let c3 = 5.85;
        let nstrings: usize = if note < 31 {
            1
        } else if note < 41 {
            2
        } else {
            3
        };
        let mut strings = vec![];
        const TUNE: [f32; 3] = [1.0, 1.0003, 0.9996];
        for i in 0..nstrings {
            strings.push(StringHammerSoundboard::new(
                f * TUNE[i],
                fs,
                hp,
                c1,
                c3,
                b,
                z,
                zb + (nstrings - 1) as f32 * z,
            ));
        }

        let alpha = 0.1e-4 * f32::ln(f / f0) / f32::ln(4192.0 / f0);

        let mut hammers: Vec<Hammer> = vec![];
        for _ in 0..nstrings {
            hammers.push(Hammer::new(fs, m, k, p, z, alpha, v0));
        }
        Piano {
            z,
            zb,
            nstrings,
            strings,
            hammers,
        }
    }

    pub fn go(&mut self) -> f32 {
        let mut load = 0.0;
        for i in 0..self.nstrings {
            let hload = self.hammers[i].load(self.strings[i].input_velocity() as f32);
            load += (2.0 * self.z * self.strings[i].go_hammer(hload / (2.0 * self.z)))
                / (self.z * self.nstrings as f32 + self.zb);
        }

        let mut output = 0.0;
        for i in 0..self.nstrings {
            output += self.strings[i].go_soundboard(load);
        }

        output
    }
}
