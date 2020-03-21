use num_traits::float::Float;
use num_traits::identities::Zero;

use super::string::String;

pub struct Soundboard<T> {
    impedance: T,
    pub velocity: T,
}

impl<T: Zero> Soundboard<T> {
    pub fn new(impedance: T) -> Soundboard<T> {
        Soundboard {
            impedance,
            velocity: T::zero(),
        }
    }
}

pub struct StringSoundboardConnection<'a, T> {
    strings: &'a mut Vec<String<T>>,
    soundboard: &'a mut Soundboard<T>,
}

impl<'a, T: Clone + Copy + Float + Zero> StringSoundboardConnection<'a, T> {
    pub fn new(
        strings: &'a mut Vec<String<T>>,
        soundboard: &'a mut Soundboard<T>,
    ) -> StringSoundboardConnection<'a, T> {
        StringSoundboardConnection {
            strings,
            soundboard,
        }
    }
    pub fn update_velocity(&mut self) {
        let connection_velocity = self.calculate_connection_velocity();

        for string in self.strings.iter_mut() {
            string.delay_line_right.next_v_right_minus =
                Some(connection_velocity - string.delay_line_right.v_right_plus);
        }
        self.soundboard.velocity = connection_velocity;
    }

    fn calculate_connection_velocity(&self) -> T {
        let connection_velocity = T::zero();
        let sum_of_impedance = T::zero();
        for string in self.strings.iter() {
            let connection_velocity = connection_velocity
                + T::from(2).unwrap() * string.impedance * string.delay_line_right.v_right_plus;
            let sum_of_impedance = sum_of_impedance + string.impedance;
        }
        let connection_velocity =
            connection_velocity / (sum_of_impedance + self.soundboard.impedance);
        connection_velocity
    }
}
