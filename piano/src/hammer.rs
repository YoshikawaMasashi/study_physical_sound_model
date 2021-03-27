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
}

impl Hammer {
    pub fn new(fs: f32, m: f32, k: f32, p: f32, alpha: f32, v0: f32) -> Hammer {
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
        }
    }

    pub fn calculate_force(
        &mut self,
        dual_force_of_input_without_hammer_force: f32,
        sum_of_impedance_at_junction: f32,
    ) -> f32 {
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
            x1 = self.x
                + (v1
                    - (dual_force_of_input_without_hammer_force + self.f)
                        / (sum_of_impedance_at_junction))
                    * self.dt;
            up = if x1 > 0.0 { f32::powf(x1, self.p) } else { 0.0 };
            dupdt = (up - self.upprev) * self.dti;
        }
        self.upprev = up;
        self.v = v1;
        self.x = x1;

        self.f
    }
}
