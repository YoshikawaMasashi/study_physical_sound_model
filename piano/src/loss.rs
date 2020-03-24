use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::filter::Filter;

pub fn loss<T: Float + Zero + FloatConst>(f0: T, fs: T, c1: T, c3: T) -> Filter<T> {
    let mut a = vec![T::zero(); 2];
    let mut b = vec![T::zero(); 2];

    let g: T = T::from(1).unwrap() - c1 / f0;
    let b_: T = T::from(4).unwrap() * c3 + f0;
    let a1: T =
        (-b_ + T::sqrt(b_ * b_ - T::from(16).unwrap() * c3 * c3)) / (T::from(4).unwrap() * c3);

    b[0] = g * (T::from(1).unwrap() + a1);
    b[1] = T::from(0).unwrap();
    a[0] = T::from(1).unwrap();
    a[1] = a1;

    Filter::new(1, a, b)
}
