use super::hammer::Hammer;
use super::loss::loss;
use super::string::String;
use super::thirian::{thirian, thirian_dispersion};

/*
F+ \Sigma{Z_i(v_i^- - v_i^+}) = 0
v_i^- + v_i^+ = v

Therefore,
F+ \Sigma{2 Z_i v_i^-} = v\Sigma{Z_i}
we name this dual_force_of_input

velocity at junction `v` is dual_force_of_input devided by sum of impedance.
and output is determined by this `v`.
*/

pub struct Piano {
    string_impedance: f32,
    soundboard_impedance: f32,

    nstrings: usize,
    left_strings: Vec<String>,
    right_strings: Vec<String>,
    hammers: Vec<Hammer>,
}

impl Piano {
    pub fn new(note: usize, sample_rate: f32, v0: f32) -> Piano {
        let note_frequency: f32 = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);

        let f0 = 27.5;
        let rho = 7850.0;
        let p = 2.0 + 1.0 * f32::ln(note_frequency / f0) / f32::ln(4192.0 / f0);
        let m = 0.06 - 0.058 * f32::powf(f32::ln(note_frequency / f0) / f32::ln(4192.0 / f0), 0.1);
        let k = 40.0 / f32::powf(0.7e-3, p);
        let l = 0.04 + 1.4 / (1.0 + f32::exp(-3.4 + 1.4 * f32::ln(note_frequency / f0)));
        let r = 0.002 * f32::powf(1.0 + 0.6 * f32::ln(note_frequency / f0), -1.4);
        let rho_l = std::f32::consts::PI * r * r * rho;
        let t = (2.0 * l * note_frequency) * (2.0 * l * note_frequency) * rho_l;
        let e = 200.0e9;

        let rcore = if r < 0.0006 { r } else { 0.0006 };
        let thirian_b = (std::f32::consts::PI * std::f32::consts::PI * std::f32::consts::PI)
            * e
            * (rcore * rcore * rcore * rcore)
            / (4.0 * l * l * t);
        let hammer_position = 1.0 / 7.0;
        let string_impedance = f32::sqrt(t * rho_l);
        let soundboard_impedance = 4000.0;

        println!(
            "note_frequency = {}, r = {} mm, L = {}, T = {}, hammer_position = {}, string_impedance = {}, k = {}, thirian_b = {}",
            note_frequency,
            1000.0 * r,
            l,
            t,
            hammer_position,
            string_impedance,
            k,
            thirian_b,
        );

        let lowpass_c1 = 0.25;
        let lowpass_c3 = 5.85;
        let nstrings: usize = if note < 31 {
            1
        } else if note < 41 {
            2
        } else {
            3
        };
        const TUNE: [f32; 3] = [1.0, 1.0003, 0.9996];
        let mut left_strings = vec![];
        let mut right_strings = vec![];
        for i in 0..nstrings {
            let (ls, rs) = Self::new_string(
                note_frequency * TUNE[i],
                sample_rate,
                hammer_position,
                lowpass_c1,
                lowpass_c3,
                thirian_b,
            );
            left_strings.push(ls);
            right_strings.push(rs);
        }

        let alpha = 0.1e-4 * f32::ln(note_frequency / f0) / f32::ln(4192.0 / f0);

        let mut hammers: Vec<Hammer> = vec![];
        for _ in 0..nstrings {
            hammers.push(Hammer::new(
                sample_rate,
                m,
                k,
                p,
                alpha,
                v0,
            ));
        }
        Piano {
            string_impedance,
            soundboard_impedance,
            nstrings,
            left_strings,
            right_strings,
            hammers,
        }
    }

    fn new_string(
        note_frequency: f32,
        sample_rate: f32,
        hammer_position: f32,
        lowpass_c1: f32,
        lowpass_c3: f32,
        thirian_b: f32,
    ) -> (String, String) {
        let deltot = sample_rate / note_frequency;
        let mut del1 = (hammer_position * 0.5 * deltot) as usize;
        if del1 < 2 {
            del1 = 1;
        }

        let mut left_filters = vec![];
        let mut right_filters = vec![];

        let m = if note_frequency > 400.0 { 1 } else { 4 };
        let mut dispersion = vec![];
        for _ in 0..m {
            dispersion.push(thirian_dispersion(thirian_b, note_frequency, m));
            left_filters.push(thirian_dispersion(thirian_b, note_frequency, m));
        }
        let dispersion_delay = m as f32 * dispersion[0].groupdelay(note_frequency, sample_rate);
        let lowpass = loss(note_frequency, lowpass_c1, lowpass_c3);
        right_filters.push(loss(note_frequency, lowpass_c1, lowpass_c3));
        let lowpass_delay = lowpass.groupdelay(note_frequency, sample_rate);

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
        let tuning_delay = fracdelay.groupdelay(note_frequency, sample_rate);

        println!("total delay = {}/{}, leftdel = {}/{}, rightdel = {}/{}, dispersion delay = {}, lowpass delay = {}, fractional delay = {}/{}",
            del1 as f32+del1 as f32+del2 as f32+del3 as f32+dispersion_delay+lowpass_delay+tuning_delay,deltot, del1, del1, del2, del3, dispersion_delay, lowpass_delay, tuning_delay, fracdelay_delay
        );

        let left_string = String::new(del1, del1, vec![], vec![]);
        let right_string = String::new(del2, del3, left_filters, right_filters);

        (left_string, right_string)
    }

    fn input_velocity(&self, string_idx: usize) -> f32 {
        self.right_strings[string_idx].v_at_left_to_left
            + self.left_strings[string_idx].v_at_right_to_right
    }

    fn do_delay(&mut self, string_idx: usize) {
        self.left_strings[string_idx].do_delay();
        self.right_strings[string_idx].do_delay();
    }

    pub fn go(&mut self) -> f32 {
        // delay line update
        for i in 0..self.nstrings {
            self.do_delay(i);
        }

        let mut dual_force_of_input_at_string_soundboard: f32 = 0.0;
        let mut dual_force_of_input_at_string_hammer: Vec<f32> = vec![];
        // calculate dual_force_of_input
        for i in 0..self.nstrings {
            let vin = self.input_velocity(i);
            let hammer_force = self.hammers[i].calculate_force(vin, self.string_impedance);
            dual_force_of_input_at_string_hammer.push(2.0 * self.string_impedance * vin + hammer_force);
        }
        for i in 0..self.nstrings {
            dual_force_of_input_at_string_soundboard +=
                2.0 * self.string_impedance * self.right_strings[i].v_at_right_to_right;
        }

        // calculate velocity
        for i in 0..self.nstrings {
            let velocity_at_string_hammer =
                dual_force_of_input_at_string_hammer[i] / (2.0 * self.string_impedance);
            self.left_strings[i].v_at_right_to_left =
                velocity_at_string_hammer - self.left_strings[i].v_at_right_to_right;
            self.right_strings[i].v_at_left_to_right =
                velocity_at_string_hammer - self.right_strings[i].v_at_left_to_left;
        }
        let velocity_at_string_soundboard = dual_force_of_input_at_string_soundboard
            / (self.nstrings as f32 * self.string_impedance + self.soundboard_impedance);
        for i in 0..self.nstrings {
            self.left_strings[i].v_at_left_to_right = -self.left_strings[i].v_at_left_to_left;
            self.right_strings[i].v_at_right_to_left =
                velocity_at_string_soundboard - self.right_strings[i].v_at_right_to_right;
        }

        let output = velocity_at_string_soundboard;

        output
    }
}
