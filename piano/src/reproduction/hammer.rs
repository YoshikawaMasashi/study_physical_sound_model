pub struct Hammer {
    dt: f32,
    dti: f32,
    x: f32,
    v: f32,
    a: f32,

    v0: f32,
    mi: f32,
    K: f32,
    p: f32,
    Fs: f32,
    F: f32,
    upprev: f32,
    alpha: f32,
    Z2i: f32,
}

impl Hammer {
    pub fn new(f: f32, Fs: f32, m: f32, K: f32, p: f32, Z: f32, alpha: f32, v0: f32) -> Hammer {
        Hammer {
            dt: 1.0 / Fs,
            dti: Fs,
            x: 0.0,
            v: v0,
            a: 0.0,

            v0: v0,
            mi: 1.0 / m,
            K: K,
            p: p,
            Fs: Fs,
            F: 0.0,
            upprev: 0.0,
            alpha: alpha,
            Z2i: 1.0 / (2.0 * Z),
        }
    }

    pub fn load(&mut self, t: f32, vin: f32) -> f32 {
        let mut up: f32 = if (self.x > 0.0) {
            f32::powf(self.x, self.p)
        } else {
            0.0
        };
        let mut dupdt = (up - self.upprev) * self.dti;
        let mut v1: f32 = 0.0;
        let mut x1: f32 = 0.0;
        for k in 0..5 {
            self.F = self.K * (up + self.alpha * dupdt);
            if (self.F < 0.0) {
                self.F = 0.0;
            }
            self.a = -self.F * self.mi;
            v1 = self.v + self.a * self.dt;
            x1 = self.x + (v1 - (vin + self.F * self.Z2i)) * self.dt;
            up = if (x1 > 0.0) {
                f32::powf(x1, self.p)
            } else {
                0.0
            };
            dupdt = (up - self.upprev) * self.dti;
        }
        self.upprev = up;
        self.v = v1;
        self.x = x1;

        self.F
    }
}
