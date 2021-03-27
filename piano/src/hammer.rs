pub struct Hammer {
    dt: f32,
    dti: f32,
    x: f32,
    v: f32,
    a: f32,

    mi: f32,
    k: f32,
    p: f32,
    f: f32,
    upprev: f32,
    alpha: f32,
    z2i: f32,
}

impl Hammer {
    pub fn new(fs: f32, m: f32, k: f32, p: f32, z: f32, alpha: f32, v0: f32) -> Hammer {
        Hammer {
            dt: 1.0 / fs,
            dti: fs,
            x: 0.0,
            v: v0,
            a: 0.0,

            mi: 1.0 / m,
            k: k,
            p: p,
            f: 0.0,
            upprev: 0.0,
            alpha: alpha,
            z2i: 1.0 / (2.0 * z),
        }
    }

    pub fn load(&mut self, vin: f32) -> f32 {
        let mut up: f32 = if self.x > 0.0 {
            f32::powf(self.x, self.p)
        } else {
            0.0
        };
        let mut dupdt = (up - self.upprev) * self.dti;
        let mut v1: f32 = 0.0;
        let mut x1: f32 = 0.0;
        for _ in 0..5 {
            self.f = self.k * (up + self.alpha * dupdt);
            if self.f < 0.0 {
                self.f = 0.0;
            }
            self.a = -self.f * self.mi;
            v1 = self.v + self.a * self.dt;
            x1 = self.x + (v1 - (vin + self.f * self.z2i)) * self.dt;
            up = if x1 > 0.0 { f32::powf(x1, self.p) } else { 0.0 };
            dupdt = (up - self.upprev) * self.dti;
        }
        self.upprev = up;
        self.v = v1;
        self.x = x1;

        self.f
    }
}