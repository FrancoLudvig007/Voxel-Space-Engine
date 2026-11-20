#![allow(dead_code)]

class! {
    struct Vec3 {
        x: f32,
        y: f32,
        z: f32,
    }
    impl {
        fn add(self, o: Vec3) -> Vec3 {
            Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
        }
        fn sub(self, o: Vec3) -> Vec3 {
            Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
        }
        fn mul(self, s: f32) -> Vec3 {
            Vec3::new(self.x * s, self.y * s, self.z * s)
        }
        fn dot(self, o: Vec3) -> f32 {
            self.x * o.x + self.y * o.y + self.z * o.z
        }
        fn cross(self, o: Vec3) -> Vec3 {
            Vec3::new(
                self.y * o.z - self.z * o.y,
                self.z * o.x - self.x * o.z,
                self.x * o.y - self.y * o.x,
            )
        }
        fn length(self) -> f32 {
            self.dot(self).sqrt()
        }
        fn normalize(self) -> Vec3 {
            let l = self.length();
            if l < 1e-6 { Vec3::new(0.0, 0.0, 0.0) } else { self.mul(1.0 / l) }
        }
        fn lerp(self, b: Vec3, t: f32) -> Vec3 {
            self.add(b.sub(self).mul(t))
        }
        fn dist(self, b: Vec3) -> f32 {
            self.sub(b).length()
        }
        fn rotate_axis(self, k: Vec3, theta: f32) -> Vec3 {
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let t1 = self.mul(cos_t);
            let t2 = k.cross(self).mul(sin_t);
            let t3 = k.mul(k.dot(self) * (1.0 - cos_t));
            t1.add(t2).add(t3)
        }
    }
}
impl Copy for Vec3 {}

class! {
    struct TexCoord {
        u: f32,
        v: f32,
    }
    impl {}
}
impl Copy for TexCoord {}

class! {
    struct Transform3D {
        pos: Vec3,
        forward: Vec3,
        up: Vec3,
        right: Vec3,
    }
    impl {
        fn orthonormalize(&mut self) {
            self.forward = self.forward.normalize();
            self.up = self.up.normalize();
            self.right = self.up.cross(self.forward).normalize();
        }
        fn pitch(&mut self, angle: f32) {
            self.forward = self.forward.rotate_axis(self.right, angle);
            self.up = self.up.rotate_axis(self.right, angle);
            self.orthonormalize();
        }
        fn yaw(&mut self, angle: f32) {
            self.forward = self.forward.rotate_axis(self.up, angle);
            self.right = self.right.rotate_axis(self.up, angle);
            self.orthonormalize();
        }
        fn roll(&mut self, angle: f32) {
            self.up = self.up.rotate_axis(self.forward, angle);
            self.right = self.right.rotate_axis(self.forward, angle);
            self.orthonormalize();
        }
    }
}

class! {
    struct Camera6DOF : Transform3D {
        spawn_pos: Vec3,
        recoil_timer: f32,
    }
    impl {
        fn default_ship() -> Self {
            let base = Transform3D::new(
                Vec3::new(4.0, 0.0, 4.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            );
            Self::new(base, Vec3::new(4.0, 0.0, 4.0), 999.0)
        }
        fn reset_to_spawn(&mut self) {
            self.pos = self.spawn_pos;
        }
        fn recoil_offset_rad(&self) -> f32 {
            if self.recoil_timer < 0.15 {
                (1.0 - self.recoil_timer / 0.15) * 0.09
            } else {
                0.0
            }
        }
    }
}
