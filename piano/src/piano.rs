use num_traits::float::Float;
use num_traits::identities::Zero;

use super::hammer::{Hammer, StringHammerConnection};
use super::soundboard::{Soundboard, StringSoundboardConnection};
use super::string::String;

struct Piano<T> {
    strings: Vec<String<T>>,
    hammer: Hammer<T>,
    soundboard: Soundboard<T>,
    dt: T,
}

impl<T: Clone + Copy + Float + Zero> Piano<T> {
    fn next(&mut self) -> T {
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
