use num_traits::float::{Float, FloatConst};
use num_traits::identities::Zero;

use super::filter::Filter;

fn combinations(n: u64, k: u64) -> u64 {
    let mut divisor: u64 = 1;
    let mut multiplier: u64 = n;
    let mut answer: u64 = 1;
    let k = std::cmp::min(k, n - k);
    while divisor <= k {
        answer = (answer * multiplier) / divisor;
        multiplier -= 1;
        divisor += 1;
    }
    answer
}

fn db<T: Float>(b: T, f: T, m: usize) -> T {
    let c1: T;
    let c2: T;
    let k1: T;
    let k2: T;
    let k3: T;
    if m == 4 {
        c1 = T::from(0.069618).unwrap();
        c2 = T::from(2.0427).unwrap();
        k1 = T::from(-0.00050469).unwrap();
        k2 = T::from(-0.0064264).unwrap();
        k3 = T::from(-2.8743).unwrap();
    } else {
        c1 = T::from(0.071089).unwrap();
        c2 = T::from(2.1074).unwrap();
        k1 = T::from(-0.0026580).unwrap();
        k2 = T::from(-0.014811).unwrap();
        k3 = T::from(-2.9018).unwrap();
    }

    let logb: T = T::ln(b);
    let kd: T = T::exp(k1 * logb * logb + k2 * logb + k3);
    let cd: T = T::exp(c1 * logb + c2);
    let halfstep: T = T::from(f64::powf(2.0, 1.0 / 12.0)).unwrap();
    let ikey: T = T::ln(f * halfstep / T::from(27.5).unwrap()) / T::ln(halfstep);
    let d: T = T::exp(cd - ikey * kd);
    d
}

pub fn thirian<T: Float + Zero + FloatConst>(d: T, n: usize) -> Filter<T> {
    let mut a = vec![T::zero(); n + 1];
    let mut b = vec![T::zero(); n + 1];

    for k in 0..n + 1 {
        let mut ak: T = T::from(combinations(n as u64, k as u64)).unwrap();
        if k % 2 == 1 {
            ak = -ak;
        }
        for i in 0..n + 1 {
            ak = ak * (d - T::from(n - i).unwrap());
            ak = ak / (d - T::from(n - k).unwrap() + T::from(i).unwrap());
        }
        a[k] = ak;
        b[n - k] = ak;
    }

    Filter::new(n, a, b, String::from("thirian"))
}

pub fn thirian_dispersion<T: Float + Zero + FloatConst>(b: T, f: T, m: usize) -> Filter<T> {
    let n: usize = 2;
    let d: T = db(b, f, m);

    if d <= T::from(1.0).unwrap() {
        let mut a1 = vec![T::zero(); n + 1];
        let mut a2 = vec![T::zero(); n + 1];
        a1[0] = T::from(1).unwrap();
        a2[0] = T::from(1).unwrap();
        Filter::new(n, a1, a2, String::from("thirian_dispersion"))
    } else {
        thirian(d, n)
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

#[test]
fn thirian_work() {
    let D: f64 = 6.1712799072265625;

    let mut filter = thirian(D, D as usize);
}
