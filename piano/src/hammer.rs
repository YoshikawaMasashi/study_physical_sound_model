use num_traits::float::Float;
use num_traits::identities::Zero;

use super::string::String;

pub struct Hammer<T> {
    compression_felt: T,
    velocity: T,
    weight: T,

    // 力の方程式のパラメーター
    p: T,
    K: T,
    alpha: T,
}

impl<T: Zero> Hammer<T> {
    pub fn new(initial_velocity: T, weight: T, p: T, K: T, alpha: T) -> Hammer<T> {
        Hammer {
            compression_felt: T::zero(),
            velocity: initial_velocity,
            weight,
            p,
            K,
            alpha,
        }
    }
}

pub struct StringHammerConnection<'a, T> {
    strings: &'a mut Vec<String<T>>,
    hammer: &'a mut Hammer<T>,
}

impl<'a, T: Clone + Copy + Float + Zero> StringHammerConnection<'a, T> {
    pub fn new(
        strings: &'a mut Vec<String<T>>,
        hammer: &'a mut Hammer<T>,
    ) -> StringHammerConnection<'a, T> {
        StringHammerConnection { strings, hammer }
    }

    pub fn update_string_velocity(&mut self, dt: T) {
        let force = self.calculate_force(dt);
        let string_velocity = self.calculate_average_string_velocity()
            + self.calculate_average_additional_string_velocity(force);

        for string in self.strings.iter_mut() {
            string.delay_line_left.next_v_right_minus =
                Some(string_velocity - string.delay_line_left.v_right_plus);
            string.delay_line_right.next_v_left_minus =
                Some(string_velocity - string.delay_line_right.v_left_plus);
        }
    }

    fn calculate_force(&mut self, dt: T) -> T {
        //
        // 弦と力の関係を自己無撞着方程式的に解く
        //
        let average_string_velocity = self.calculate_average_string_velocity();

        let mut up = self.calculate_up(self.hammer.compression_felt, self.hammer.p);
        let mut prev_up = up;

        let mut dupdt = (up - prev_up) / dt;
        let mut force = T::zero();
        let mut next_compression_felt = T::zero();
        let mut next_hammer_velocity = T::zero();

        for k in 0..5 {
            force = self.relu(self.hammer.K * (up + self.hammer.alpha * dupdt));

            let average_additional_string_velocity =
                self.calculate_average_additional_string_velocity(force);

            let acceleration = -force / self.hammer.weight;
            next_hammer_velocity = self.hammer.velocity + acceleration * dt;

            next_compression_felt = self.hammer.compression_felt
                + (next_hammer_velocity
                    - (average_string_velocity + average_additional_string_velocity))
                    * dt;

            up = self.calculate_up(next_compression_felt, self.hammer.p);
            dupdt = (up - prev_up) / dt;
        }

        self.hammer.velocity = next_hammer_velocity;
        self.hammer.compression_felt = next_compression_felt;
        force
    }

    fn calculate_average_string_velocity(&self) -> T {
        let average_string_velocity = T::zero();
        for string in self.strings.iter() {
            let average_string_velocity =
                average_string_velocity + string.delay_line_left.v_right_plus;
            let average_string_velocity =
                average_string_velocity + string.delay_line_right.v_left_plus;
        }
        let average_string_velocity =
            average_string_velocity / T::from(self.strings.len()).unwrap();
        average_string_velocity
    }

    fn calculate_average_additional_string_velocity(&self, force: T) -> T {
        let average_additional_string_velocity = T::zero();
        for string in self.strings.iter() {
            let average_additional_string_velocity =
                average_additional_string_velocity + force / string.impedance;
        }
        let average_additional_string_velocity =
            average_additional_string_velocity / T::from(self.strings.len()).unwrap();
        average_additional_string_velocity
    }

    fn calculate_up(&self, u: T, p: T) -> T {
        if u > T::zero() {
            T::powf(u, p)
        } else {
            T::zero()
        }
    }

    fn relu(&self, x: T) -> T {
        if x > T::zero() {
            x
        } else {
            T::zero()
        }
    }
}
