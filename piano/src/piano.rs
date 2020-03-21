use num_traits::cast::ToPrimitive;
use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::hammer::{Hammer, StringHammerConnection};
use super::soundboard::{Soundboard, StringSoundboardConnection};
use super::string::String;

pub struct Piano<T> {
    strings: Vec<String<T>>,
    hammer: Hammer<T>,
    soundboard: Soundboard<T>,
    dt: T,
}

impl<T: Clone + Copy + Float + Zero + ToPrimitive + FloatConst> Piano<T> {
    pub fn new(note: T, sample_frequency: T, initial_hammer_velocity: T) -> Piano<T> {
        let dt: f64 = 1.0 / sample_frequency.to_f64().unwrap();
        let note_frequency: f64 = 440.0 * f64::powf(2.0, (note.to_f64().unwrap() - 69.0) / 12.0);

        // from http://large.stanford.edu/courses/2007/ph210/otey2/c/
        let f0: f64 = 27.5;
        let rho: f64 = 7850.0;
        let p: f64 = 2.0 + 1.0 * f64::ln(note_frequency / f0) / f64::ln(4192.0 / f0);
        let m: f64 =
            0.06 - 0.058 * f64::powf(f64::ln(note_frequency / f0) / f64::ln(4192.0 / f0), 0.1);
        let K: f64 = 40.0 / f64::powf(0.7e-3, p);
        let L: f64 = 0.04 + 1.4 / (1.0 + f64::exp(-3.4 + 1.4 * f64::ln(note_frequency / f0)));
        let r: f64 = 0.002 * f64::powf(1.0 + 0.6 * f64::ln(note_frequency / f0), -1.4);
        let rhoL: f64 = std::f64::consts::PI * r * r * rho;
        let T: f64 = (2.0 * L * note_frequency) * (2.0 * L * note_frequency) * rhoL;
        let Z: f64 = f64::sqrt(T * rhoL);
        let Zb: f64 = 4000.0;
        let E: f64 = 200.0e9;
        let flong: f64 = f64::sqrt(E / rho) / (2.0 * L);

        let rcore: f64 = if (r < 0.0006) { r } else { 0.0006 };
        let B: f64 = (std::f64::consts::PI * std::f64::consts::PI * std::f64::consts::PI)
            * E
            * (rcore * rcore * rcore * rcore)
            / (4.0 * L * L * T);
        let hp: f64 = 1.0 / 7.0;

        print!(
            "f = {}, r = {} mm, L = {}, T = {}, hammer = {}, Z = {}, K = {}, B = {}\n",
            note_frequency,
            1000.0 * r,
            L,
            T,
            hp,
            Z,
            K,
            B
        );

        let c1: f64 = 0.25;
        let c3: f64 = 5.85;
        let nstrings: usize = if note < T::from(31).unwrap() {
            1
        } else if note < T::from(41).unwrap() {
            2
        } else {
            3
        };

        let c1b: f64 = 20.0;
        let c3b: f64 = 20.0;

        let mut strings: Vec<String<T>> = Vec::new();
        let TUNE: [T; 3] = [
            T::from(1).unwrap(),
            T::from(1.0003).unwrap(),
            T::from(0.9996).unwrap(),
        ];
        for k in 0..nstrings {
            strings.push(String::new(
                T::from(note_frequency).unwrap() * TUNE[k],
                T::from(sample_frequency).unwrap(),
                T::from(hp).unwrap(),
                T::from(c1).unwrap(),
                T::from(c3).unwrap(),
                T::from(B).unwrap(),
                T::from(Z).unwrap(),
            ));
        }

        let a: f64 = -1.0 / 4.0;
        let mix: f64 = 1.0;
        let alpha: f64 = 0.1e-4 * f64::ln(note_frequency / f0) / f64::ln(4192.0 / f0);
        let mut hammer = Hammer::new(
            initial_hammer_velocity,
            T::from(m).unwrap(),
            T::from(p).unwrap(),
            T::from(K).unwrap(),
            T::from(alpha).unwrap(),
        );
        let mut soundboard = Soundboard::new(T::from(Zb).unwrap());

        Piano {
            strings,
            hammer,
            soundboard,
            dt: T::from(dt).unwrap(),
        }
    }

    pub fn next(&mut self) -> T {
        for string in self.strings.iter_mut() {
            string.pin_update();
        }

        {
            StringHammerConnection::new(&mut self.strings, &mut self.hammer)
                .update_string_velocity(self.dt);
        }

        {
            StringSoundboardConnection::new(&mut self.strings, &mut self.soundboard)
                .update_velocity();
        }

        for string in self.strings.iter_mut() {
            string.update();
        }

        self.soundboard.velocity
    }
}
