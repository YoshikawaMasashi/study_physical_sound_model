use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::filter::Filter;

fn combinations(n: u64, k: u64) -> u64 {
    let mut divisor: u64 = 1;
    let mut multiplier: u64 = n;
    let mut answer: u64 = 1;
    let k = std::cmp::min(k, n - k);
    while (divisor <= k) {
        answer = (answer * multiplier) / divisor;
        multiplier -= 1;
        divisor += 1;
    }
    answer
}

fn Db<T: Float>(B: T, f: T, M: usize) -> T {
    let C1: T;
    let C2: T;
    let k1: T;
    let k2: T;
    let k3: T;
    if (M == 4) {
        C1 = T::from(0.069618).unwrap();
        C2 = T::from(2.0427).unwrap();
        k1 = T::from(-0.00050469).unwrap();
        k2 = T::from(-0.0064264).unwrap();
        k3 = T::from(-2.8743).unwrap();
    } else {
        C1 = T::from(0.071089).unwrap();
        C2 = T::from(2.1074).unwrap();
        k1 = T::from(-0.0026580).unwrap();
        k2 = T::from(-0.014811).unwrap();
        k3 = T::from(-2.9018).unwrap();
    }

    let logB: T = T::ln(B);
    let kd: T = T::exp(k1 * logB * logB + k2 * logB + k3);
    let Cd: T = T::exp(C1 * logB + C2);
    let halfstep: T = T::from(f64::powf(2.0, 1.0 / 12.0)).unwrap();
    let Ikey: T = T::ln(f * halfstep / T::from(27.5).unwrap()) / T::ln(halfstep);
    let D: T = T::exp(Cd - Ikey * kd);
    D
}

pub fn thirian<T: Float + Zero + FloatConst>(D: T, N: usize) -> Filter<T> {
    let mut a = vec![T::zero(); N + 1];
    let mut b = vec![T::zero(); N + 1];

    for k in 0..N + 1 {
        let mut ak: T = T::from(combinations(N as u64, k as u64)).unwrap();
        if k % 2 == 1 {
            ak = -ak;
        }
        for n in 0..N + 1 {
            ak = ak * (D - T::from(N - n).unwrap());
            ak = ak / (D - T::from(N - k).unwrap() + T::from(n).unwrap());
        }
        a[k] = ak;
        b[N - k] = ak;
    }

    Filter::new(N, a, b)
}

pub fn thirian_dispersion<T: Float + Zero + FloatConst>(B: T, f: T, M: usize) -> Filter<T> {
    let N: usize = 2;
    let D: T = Db(B, f, M);

    if (D <= T::from(1.0).unwrap()) {
        let mut a = vec![T::zero(); N + 1];
        let mut b = vec![T::zero(); N + 1];
        a[0] = T::from(1).unwrap();
        b[0] = T::from(1).unwrap();
        Filter::new(N, a, b)
    } else {
        thirian(D, N)
    }
}

#[test]
fn thirian_dispersion_work() {
    let B: f64 = 0.000175;
    let f: f64 = 261.520935;
    let M: usize = 4;

    let mut filter = thirian_dispersion(B, f, M);

    assert_eq!(filter.a[0], 1.000000);
    assert_eq!(filter.a[1], -1.2356172436816146);
    assert_eq!(filter.a[2], 0.4083694427233187);
    assert_eq!(filter.b[0], 0.4083694427233187);
    assert_eq!(filter.b[1], -1.2356172436816146);
    assert_eq!(filter.b[2], 1.000000);

    assert_eq!(filter.groupdelay(261.520935, 44100.0), 6.849290787197244);
}
