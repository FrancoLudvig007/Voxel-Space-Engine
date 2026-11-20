//! Matemática vetorial e utilidades de cor compartilhadas por todo o motor.
//!
//! Convenção de coordenadas do mundo (igual à do heightmap):
//! - `x` → colunas do mapa (direita na visão superior)
//! - `y` → linhas do mapa (para baixo na visão superior)
//! - `z` → altitude (para cima)

use eframe::egui::Color32;

class! {
    struct Vec3 {
        x: f32,
        y: f32,
        z: f32,
    }
    impl {
        fn zero() -> Vec3 {
            Vec3 { x: 0.0, y: 0.0, z: 0.0 }
        }
        fn add(self, o: Vec3) -> Vec3 {
            Vec3 { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z }
        }
        fn sub(self, o: Vec3) -> Vec3 {
            Vec3 { x: self.x - o.x, y: self.y - o.y, z: self.z - o.z }
        }
        fn scale(self, s: f32) -> Vec3 {
            Vec3 { x: self.x * s, y: self.y * s, z: self.z * s }
        }
        fn dot(self, o: Vec3) -> f32 {
            self.x * o.x + self.y * o.y + self.z * o.z
        }
        fn cross(self, o: Vec3) -> Vec3 {
            Vec3 {
                x: self.y * o.z - self.z * o.y,
                y: self.z * o.x - self.x * o.z,
                z: self.x * o.y - self.y * o.x,
            }
        }
        fn length(self) -> f32 {
            self.dot(self).sqrt()
        }
        fn normalize(self) -> Vec3 {
            let l = self.length();
            if l > 1e-8 { self.scale(1.0 / l) } else { self }
        }
    }
}
impl Copy for Vec3 {}

impl Default for Vec3 {
    fn default() -> Self {
        Vec3::zero()
    }
}

/// Direção do sol a partir de azimute/elevação (graus).
pub fn sun_direction(azimuth_deg: f32, elevation_deg: f32) -> Vec3 {
    let (az, el) = (azimuth_deg.to_radians(), elevation_deg.to_radians());
    Vec3::new(el.cos() * az.cos(), el.cos() * az.sin(), el.sin()).normalize()
}

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Mistura linear de duas cores em ponto fixo (8 bits de fração) — barata para o laço quente.
#[inline(always)]
pub fn mix_rgb(a: Color32, b: Color32, t: f32) -> Color32 {
    let ti = (t.clamp(0.0, 1.0) * 256.0) as u32;
    let si = 256 - ti;
    Color32::from_rgb(
        ((a.r() as u32 * si + b.r() as u32 * ti) >> 8) as u8,
        ((a.g() as u32 * si + b.g() as u32 * ti) >> 8) as u8,
        ((a.b() as u32 * si + b.b() as u32 * ti) >> 8) as u8,
    )
}

/// Mistura em ponto flutuante com arredondamento (usada pelo pincel de pintura,
/// onde incrementos pequenos não podem ser perdidos por truncamento).
#[inline]
pub fn mix_rgb_precise(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t + 0.5) as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Multiplica o brilho de uma cor (com saturação em 255).
#[inline(always)]
pub fn mul_rgb(c: Color32, f: f32) -> Color32 {
    let fi = (f.max(0.0) * 256.0) as u32;
    Color32::from_rgb(
        ((c.r() as u32 * fi) >> 8).min(255) as u8,
        ((c.g() as u32 * fi) >> 8).min(255) as u8,
        ((c.b() as u32 * fi) >> 8).min(255) as u8,
    )
}

/// Multiplica duas cores componente a componente (textura × tonalidade).
#[inline(always)]
pub fn modulate(a: Color32, b: Color32) -> Color32 {
    Color32::from_rgb(
        ((a.r() as u32 * b.r() as u32) / 255) as u8,
        ((a.g() as u32 * b.g() as u32) / 255) as u8,
        ((a.b() as u32 * b.b() as u32) / 255) as u8,
    )
}

/// Reduz uma diferença de coordenadas ao intervalo `[-size/2, size/2)` (mapa toroidal).
#[inline(always)]
pub fn wrap_delta(d: f32, size: f32) -> f32 {
    let mut r = d.rem_euclid(size);
    if r >= size * 0.5 {
        r -= size;
    }
    r
}

/// Número de threads de trabalho disponíveis no sistema.
pub fn worker_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, 64)
}

/// Hash inteiro rápido e determinístico (ruído de micro-variação).
#[inline(always)]
pub fn hash2(x: i32, y: i32) -> u8 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h >> 24) as u8
}
