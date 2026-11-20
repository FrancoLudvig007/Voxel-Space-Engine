//! Terreno contínuo baseado em Heightmap + Colormap, com repetição infinita
//! (wrap-around toroidal via máscara de bits — dimensões sempre potência de 2).
//!
//! O renderizador lê apenas `cells` (altura + cor já iluminada intercaladas em
//! 8 bytes, amigável ao cache). `heights` e `albedo` são os dados "fonte"
//! editados pelo pincel; após cada edição apenas a região tocada é recalculada.

use crate::math::{hash2, mix_rgb, mul_rgb, smoothstep, sun_direction, Vec3};
use crate::pyrsutils::prelude::*;
use eframe::egui::Color32;
use image::imageops::FilterType;

/// Faixa de altura de um heightmap 8/16 bits importado (0..255 unidades, estilo Comanche).
pub const HEIGHTMAP_RANGE: f32 = 255.0;
/// Limite de altura que o pincel pode alcançar.
pub const MAX_HEIGHT: f32 = 1024.0;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Cell {
    pub h: f32,
    pub color: Color32,
}

// ============================================================================
// Ruído de Perlin periódico (tileable) — essencial para o mapa repetir sem costuras
// ============================================================================
pub struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    pub fn new(semente: u64) -> Self {
        seed(semente);
        let mut p: Vec<u8> = (0..=255u8).collect();
        shuffle(&mut p);
        let mut perm = [0u8; 512];
        for (i, slot) in enumerate(perm.iter_mut()) {
            *slot = p[i & 255];
        }
        Self { perm }
    }

    #[inline(always)]
    fn hash(&self, x: i32, y: i32) -> u8 {
        let a = self.perm[(x & 255) as usize] as usize;
        self.perm[a + (y & 255) as usize]
    }

    #[inline(always)]
    fn grad(h: u8, x: f32, y: f32) -> f32 {
        match h & 7 {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            3 => -x - y,
            4 => x,
            5 => -x,
            6 => y,
            _ => -y,
        }
    }

    #[inline(always)]
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    /// Ruído 2D com período `period` em ambos os eixos (coordenadas de rede).
    pub fn noise_periodic(&self, x: f32, y: f32, period: i32) -> f32 {
        let (xf, yf) = (x.floor(), y.floor());
        let xi = (xf as i32).rem_euclid(period);
        let yi = (yf as i32).rem_euclid(period);
        let xi1 = (xi + 1) % period;
        let yi1 = (yi + 1) % period;
        let (fx, fy) = (x - xf, y - yf);
        let (u, v) = (Self::fade(fx), Self::fade(fy));
        let n00 = Self::grad(self.hash(xi, yi), fx, fy);
        let n10 = Self::grad(self.hash(xi1, yi), fx - 1.0, fy);
        let n01 = Self::grad(self.hash(xi, yi1), fx, fy - 1.0);
        let n11 = Self::grad(self.hash(xi1, yi1), fx - 1.0, fy - 1.0);
        let a = n00 + (n10 - n00) * u;
        let b = n01 + (n11 - n01) * u;
        a + (b - a) * v
    }

    /// Fractal Brownian Motion periódico em `[0,1)²` (lacunaridade 2 → períodos inteiros).
    pub fn fbm(&self, u: f32, v: f32, base_freq: u32, octaves: u32, persistence: f32, ridged: bool) -> f32 {
        let (mut sum, mut amp, mut norm) = (0.0f32, 1.0f32, 0.0f32);
        let mut freq = base_freq.max(1);
        for _ in 0..octaves.max(1) {
            let mut n = self.noise_periodic(u * freq as f32, v * freq as f32, freq as i32);
            if ridged {
                n = 1.0 - n.abs() * 1.6;
                n = n * n - 0.5;
            }
            sum += n * amp;
            norm += amp;
            amp *= persistence;
            freq = freq.saturating_mul(2);
        }
        sum / norm
    }
}

// ============================================================================
// Parâmetros de geração
// ============================================================================
#[derive(Clone, Debug)]
pub struct NoiseParams {
    pub seed: u64,
    pub size_exp: u32,
    pub base_freq: u32,
    pub octaves: u32,
    pub persistence: f32,
    pub amplitude: f32,
    pub exponent: f32,
    pub ridged: bool,
    pub auto_color: bool,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            seed: 1337,
            size_exp: 10,
            base_freq: 4,
            octaves: 8,
            persistence: 0.5,
            amplitude: 255.0,
            exponent: 1.7,
            ridged: false,
            auto_color: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlatParams {
    pub size_exp: u32,
    pub base_height: f32,
    pub color: Color32,
}

impl Default for FlatParams {
    fn default() -> Self {
        Self { size_exp: 10, base_height: 40.0, color: Color32::from_rgb(92, 130, 64) }
    }
}

/// Arredonda para a potência de 2 mais próxima dentro de `[64, 4096]`.
fn pow2_round(v: u32) -> u32 {
    let v = v.max(1);
    let lo = 1u32 << (31 - v.leading_zeros());
    let hi = lo.saturating_mul(2);
    let r = if v - lo <= hi - v { lo } else { hi };
    r.clamp(64, 4096)
}

// ============================================================================
// Terreno
// ============================================================================
pub struct Terrain {
    pub width: usize,
    pub height: usize,
    pub mask_x: usize,
    pub mask_y: usize,
    pub shift_y: u32,
    pub heights: Vec<f32>,
    pub albedo: Vec<Color32>,
    pub cells: Vec<Cell>,
    pub lighting: bool,
    pub sun_azimuth: f32,
    pub sun_elevation: f32,
    pub sun_dir: Vec3,
    pub ambient: f32,
    pub relief: f32,
    /// `true` enquanto o colormap for gerado automaticamente a partir da altura.
    pub colormap_auto: bool,
    /// Incrementado a cada modificação (usado para invalidar o minimapa).
    pub revision: u64,
    pub name: String,
}

impl Terrain {
    fn alloc(w: usize, h: usize) -> Self {
        assert!(w.is_power_of_two() && h.is_power_of_two(), "dimensões devem ser potência de 2");
        let (az, el) = (225.0, 38.0);
        Self {
            width: w,
            height: h,
            mask_x: w - 1,
            mask_y: h - 1,
            shift_y: w.trailing_zeros(),
            heights: vec![0.0; w * h],
            albedo: vec![Color32::GRAY; w * h],
            cells: vec![Cell::default(); w * h],
            lighting: true,
            sun_azimuth: az,
            sun_elevation: el,
            sun_dir: sun_direction(az, el),
            ambient: 0.38,
            relief: 1.0,
            colormap_auto: true,
            revision: 0,
            name: String::new(),
        }
    }

    /// Realoca o armazenamento se as dimensões mudarem. Retorna `true` se mudou.
    fn resize_storage(&mut self, w: usize, h: usize) -> bool {
        if w == self.width && h == self.height {
            return false;
        }
        assert!(w.is_power_of_two() && h.is_power_of_two());
        self.width = w;
        self.height = h;
        self.mask_x = w - 1;
        self.mask_y = h - 1;
        self.shift_y = w.trailing_zeros();
        self.heights = vec![0.0; w * h];
        self.albedo = vec![Color32::GRAY; w * h];
        self.cells = vec![Cell::default(); w * h];
        true
    }

    pub fn new_flat(p: &FlatParams) -> Self {
        let size = 1usize << p.size_exp.clamp(6, 12);
        let mut t = Self::alloc(size, size);
        t.apply_flat(p);
        t
    }

    pub fn apply_flat(&mut self, p: &FlatParams) {
        let size = 1usize << p.size_exp.clamp(6, 12);
        self.resize_storage(size, size);
        self.heights.fill(p.base_height);
        self.albedo.fill(p.color);
        self.colormap_auto = false;
        self.name = format!("Plano {size}x{size}");
        self.rebuild_all();
    }

    pub fn new_perlin(p: &NoiseParams, water_raw: f32) -> Self {
        let size = 1usize << p.size_exp.clamp(6, 12);
        let mut t = Self::alloc(size, size);
        t.generate_perlin(p, water_raw);
        t
    }

    #[inline(always)]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (((y as usize) & self.mask_y) << self.shift_y) | ((x as usize) & self.mask_x)
    }

    #[inline(always)]
    pub fn height_raw(&self, x: i32, y: i32) -> f32 {
        self.heights[self.idx(x, y)]
    }

    /// Altura bilinear (unidades brutas, sem escala vertical) — usada em colisão.
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let (xf, yf) = (x.floor(), y.floor());
        let (ix, iy) = (xf as i32, yf as i32);
        let (fx, fy) = (x - xf, y - yf);
        let h00 = self.height_raw(ix, iy);
        let h10 = self.height_raw(ix + 1, iy);
        let h01 = self.height_raw(ix, iy + 1);
        let h11 = self.height_raw(ix + 1, iy + 1);
        let a = h00 + (h10 - h00) * fx;
        let b = h01 + (h11 - h01) * fx;
        a + (b - a) * fy
    }

    pub fn set_sun(&mut self, azimuth_deg: f32, elevation_deg: f32) {
        self.sun_azimuth = azimuth_deg;
        self.sun_elevation = elevation_deg;
        self.sun_dir = sun_direction(azimuth_deg, elevation_deg);
    }

    #[inline]
    fn update_cell(&mut self, x: i32, y: i32, flat_norm: f32) {
        let i = self.idx(x, y);
        let h = self.heights[i];
        let color = if self.lighting {
            let k = self.relief;
            let dx = (self.height_raw(x - 1, y) - self.height_raw(x + 1, y)) * k;
            let dy = (self.height_raw(x, y - 1) - self.height_raw(x, y + 1)) * k;
            let n = Vec3::new(dx, dy, 2.0).normalize();
            let diff = n.dot(self.sun_dir).max(0.0);
            mul_rgb(self.albedo[i], (self.ambient + (1.0 - self.ambient) * diff) * flat_norm)
        } else {
            self.albedo[i]
        };
        self.cells[i] = Cell { h, color };
    }

    /// Recalcula a cache de renderização (`cells`) numa região quadrada (com wrap).
    pub fn update_region(&mut self, cx: i32, cy: i32, r: i32) {
        let (w, h) = (self.width as i32, self.height as i32);
        let (x0, x1) = if 2 * r + 1 >= w { (0, w - 1) } else { (cx - r, cx + r) };
        let (y0, y1) = if 2 * r + 1 >= h { (0, h - 1) } else { (cy - r, cy + r) };
        // Normaliza para que o terreno plano mantenha a cor original do colormap.
        let flat = self.ambient + (1.0 - self.ambient) * self.sun_dir.z.max(0.05);
        let flat_norm = 1.0 / flat;
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.update_cell(x, y, flat_norm);
            }
        }
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn rebuild_all(&mut self) {
        let (w, h) = (self.width as i32, self.height as i32);
        self.update_region(w / 2, h / 2, w.max(h));
    }

    // ------------------------------------------------------------------------
    // Geração procedural
    // ------------------------------------------------------------------------
    pub fn generate_perlin(&mut self, p: &NoiseParams, water_raw: f32) {
        let size = 1usize << p.size_exp.clamp(6, 12);
        let mudou = self.resize_storage(size, size);
        let perlin = Perlin::new(p.seed);
        let threads = crate::math::worker_threads();
        let rows_per = size.div_ceil(threads);
        let inv = 1.0 / size as f32;
        std::thread::scope(|s| {
            for (ci, chunk) in self.heights.chunks_mut(rows_per * size).enumerate() {
                let perlin = &perlin;
                s.spawn(move || {
                    for (j, row) in chunk.chunks_mut(size).enumerate() {
                        let v = (ci * rows_per + j) as f32 * inv;
                        for (x, out) in row.iter_mut().enumerate() {
                            *out = perlin.fbm(x as f32 * inv, v, p.base_freq, p.octaves, p.persistence, p.ridged);
                        }
                    }
                });
            }
        });
        let (mn, mx) = self
            .heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &h| (a.min(h), b.max(h)));
        let range = (mx - mn).max(1e-6);
        for h in self.heights.iter_mut() {
            *h = ((*h - mn) / range).powf(p.exponent) * p.amplitude;
        }
        // Mantém um colormap importado (mesmo tamanho) se o usuário desligar a cor automática.
        if p.auto_color || mudou || self.colormap_auto {
            self.auto_colorize(water_raw);
        }
        self.name = format!("Perlin seed={} {size}x{size}", p.seed);
        self.rebuild_all();
    }

    /// Gera um colormap a partir de altura/inclinação (areia, grama, floresta, rocha, neve).
    pub fn auto_colorize(&mut self, water_raw: f32) {
        let max_h = self.heights.iter().cloned().fold(1.0f32, f32::max);
        let sand = Color32::from_rgb(198, 184, 134);
        let wet_sand = Color32::from_rgb(150, 136, 96);
        let seabed = Color32::from_rgb(96, 92, 66);
        let grass = Color32::from_rgb(90, 136, 54);
        let forest = Color32::from_rgb(46, 88, 38);
        let rock = Color32::from_rgb(120, 110, 98);
        let snow = Color32::from_rgb(240, 242, 248);
        let (w, h) = (self.width as i32, self.height as i32);
        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let hh = self.heights[i];
                let slope = ((self.height_raw(x - 1, y) - self.height_raw(x + 1, y)).abs()
                    + (self.height_raw(x, y - 1) - self.height_raw(x, y + 1)).abs())
                    * 0.5;
                let jitter = (hash2(x, y) as f32 / 255.0 - 0.5) * 0.08;
                let t = (hh / max_h + jitter).clamp(0.0, 1.0);
                let mut c = if hh < water_raw - 1.0 {
                    mix_rgb(wet_sand, seabed, ((water_raw - hh) / 40.0).clamp(0.0, 1.0))
                } else if hh < water_raw + 3.0 {
                    sand
                } else if t < 0.55 {
                    mix_rgb(grass, forest, smoothstep(0.25, 0.55, t))
                } else if t < 0.78 {
                    mix_rgb(forest, rock, smoothstep(0.55, 0.78, t))
                } else {
                    mix_rgb(rock, snow, smoothstep(0.80, 0.90, t))
                };
                if slope > 1.2 && hh >= water_raw + 3.0 {
                    c = mix_rgb(c, rock, smoothstep(1.2, 3.5, slope));
                }
                self.albedo[i] = mul_rgb(c, 1.0 + jitter * 1.5);
            }
        }
        self.colormap_auto = true;
    }

    // ------------------------------------------------------------------------
    // Importação / exportação
    // ------------------------------------------------------------------------
    pub fn load_heightmap(&mut self, path: &str, water_raw: f32) -> Result<String, String> {
        if !isfile(path) {
            return Err(format!("Arquivo não encontrado: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let (w0, h0) = (img.width(), img.height());
        let (w, h) = (pow2_round(w0), pow2_round(h0));
        let mut gray = img.to_luma16();
        if (w, h) != (w0, h0) {
            gray = image::imageops::resize(&gray, w, h, FilterType::Triangle);
        }
        let mudou = self.resize_storage(w as usize, h as usize);
        for (dst, p) in self.heights.iter_mut().zip(gray.pixels()) {
            *dst = p[0] as f32 / 65535.0 * HEIGHTMAP_RANGE;
        }
        if mudou || self.colormap_auto {
            self.auto_colorize(water_raw);
        }
        self.name = basename(path);
        self.rebuild_all();
        Ok(format!("Heightmap '{}' carregado ({w0}x{h0} → {w}x{h}).", self.name))
    }

    pub fn load_colormap(&mut self, path: &str) -> Result<String, String> {
        if !isfile(path) {
            return Err(format!("Arquivo não encontrado: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let (w0, h0) = (img.width(), img.height());
        let (w, h) = (self.width as u32, self.height as u32);
        let mut rgb = img.to_rgb8();
        if (w, h) != (w0, h0) {
            rgb = image::imageops::resize(&rgb, w, h, FilterType::Triangle);
        }
        for (dst, p) in self.albedo.iter_mut().zip(rgb.pixels()) {
            *dst = Color32::from_rgb(p[0], p[1], p[2]);
        }
        self.colormap_auto = false;
        self.rebuild_all();
        Ok(format!("Colormap '{}' aplicado ({w0}x{h0} → {w}x{h}).", basename(path)))
    }

    pub fn export_heightmap(&self, path: &str) -> Result<String, String> {
        let (w, h) = (self.width as u32, self.height as u32);
        let img = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_fn(w, h, |x, y| {
            let v = self.heights[(y as usize) * self.width + x as usize] / HEIGHTMAP_RANGE;
            image::Luma([(v.clamp(0.0, 1.0) * 65535.0) as u16])
        });
        img.save(path).map_err(|e| format!("Falha ao salvar '{path}': {e}"))?;
        Ok(format!("Heightmap exportado (PNG 16 bits) → {path}"))
    }

    pub fn export_colormap(&self, path: &str) -> Result<String, String> {
        let (w, h) = (self.width as u32, self.height as u32);
        let img = image::RgbImage::from_fn(w, h, |x, y| {
            let c = self.albedo[(y as usize) * self.width + x as usize];
            image::Rgb([c.r(), c.g(), c.b()])
        });
        img.save(path).map_err(|e| format!("Falha ao salvar '{path}': {e}"))?;
        Ok(format!("Colormap exportado → {path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_periodic_wraps_seamlessly() {
        let p = Perlin::new(42);
        let a = p.fbm(0.0, 0.25, 4, 5, 0.5, false);
        let b = p.fbm(1.0, 0.25, 4, 5, 0.5, false);
        assert!((a - b).abs() < 1e-5);
    }

    #[test]
    fn flat_terrain_and_bilinear_height() {
        let mut t = Terrain::new_flat(&FlatParams {
            size_exp: 6,
            base_height: 45.0,
            color: Color32::from_rgb(80, 120, 60),
        });
        assert_eq!(t.width, 64);
        assert!((t.height_at(10.5, 20.5) - 45.0).abs() < 1e-4);
        let i = t.idx(0, 0);
        t.heights[i] = 100.0;
        assert!((t.height_at(64.0, 64.0) - 100.0).abs() < 1e-4);
    }
}
