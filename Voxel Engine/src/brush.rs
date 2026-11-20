//! Ferramenta de escultura e pintura de terreno em tempo real.

use crate::math::{hash2, mix_rgb_precise, modulate, smoothstep};
use crate::pyrsutils::prelude::{basename, isfile};
use crate::terrain::{Perlin, Terrain, MAX_HEIGHT};
use eframe::egui::Color32;
use image::imageops::FilterType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrushMode {
    Raise,
    Lower,
    Smooth,
    Flatten,
    Paint,
}

impl BrushMode {
    pub const ALL: [BrushMode; 5] = [Self::Raise, Self::Lower, Self::Smooth, Self::Flatten, Self::Paint];

    pub fn label(self) -> &'static str {
        match self {
            Self::Raise => "⬆ Elevar",
            Self::Lower => "⬇ Rebaixar",
            Self::Smooth => "≈ Suavizar",
            Self::Flatten => "▬ Aplainar",
            Self::Paint => "🖌 Pintar cor/tex",
        }
    }

    pub fn overlay_color(self) -> Color32 {
        match self {
            Self::Raise => Color32::from_rgb(120, 255, 120),
            Self::Lower => Color32::from_rgb(255, 120, 100),
            Self::Smooth => Color32::from_rgb(120, 200, 255),
            Self::Flatten => Color32::from_rgb(255, 230, 120),
            Self::Paint => Color32::from_rgb(255, 255, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintSource {
    SolidColor,
    GrassTex,
    RockTex,
    SandTex,
    SnowTex,
    RoadTex,
    CustomTex,
}

impl PaintSource {
    pub const ALL: [PaintSource; 7] = [
        Self::SolidColor,
        Self::GrassTex,
        Self::RockTex,
        Self::SandTex,
        Self::SnowTex,
        Self::RoadTex,
        Self::CustomTex,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::SolidColor => "Cor sólida",
            Self::GrassTex => "Grama (textura)",
            Self::RockTex => "Rocha (textura)",
            Self::SandTex => "Areia (textura)",
            Self::SnowTex => "Neve (textura)",
            Self::RoadTex => "Estrada (textura)",
            Self::CustomTex => "Imagem externa",
        }
    }
}

pub struct Brush {
    pub mode: BrushMode,
    pub radius: f32,
    /// Força por segundo (multiplicada pelo `dt` do frame → independente do FPS).
    pub strength: f32,
    /// 0 = borda totalmente suave, 1 = borda dura.
    pub hardness: f32,
    pub flatten_height: f32,
    pub flatten_pick_on_click: bool,
    pub paint_color: Color32,
    pub paint_opacity: f32,
    /// Também pinta enquanto esculpe (útil para trilhas/estradas).
    pub paint_while_sculpting: bool,
    pub paint_source: PaintSource,
    pub paint_tex_scale: f32,
    pub tint_texture: bool,
    pub paint_tex_name: String,
    paint_tex_w: usize,
    paint_tex_h: usize,
    paint_tex_pixels: Vec<Color32>,
    scratch: Vec<f32>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            mode: BrushMode::Raise,
            radius: 24.0,
            strength: 1.0,
            hardness: 0.25,
            flatten_height: 80.0,
            flatten_pick_on_click: true,
            paint_color: Color32::from_rgb(150, 120, 80),
            paint_opacity: 0.8,
            paint_while_sculpting: false,
            paint_source: PaintSource::SolidColor,
            paint_tex_scale: 0.5,
            tint_texture: false,
            paint_tex_name: "Cor sólida".to_string(),
            paint_tex_w: 0,
            paint_tex_h: 0,
            paint_tex_pixels: Vec::new(),
            scratch: Vec::with_capacity(128 * 128),
        }
    }
}

fn generate_preset_texture(src: PaintSource) -> (usize, usize, Vec<Color32>) {
    let size = 64usize;
    let perlin = Perlin::new(match src {
        PaintSource::GrassTex => 101,
        PaintSource::RockTex => 202,
        PaintSource::SandTex => 303,
        PaintSource::SnowTex => 404,
        _ => 505,
    });
    let inv = 1.0 / size as f32;
    let mut pixels = vec![Color32::GRAY; size * size];
    for y in 0..size {
        for x in 0..size {
            let n = perlin.fbm(x as f32 * inv, y as f32 * inv, 4, 4, 0.55, src == PaintSource::RockTex);
            let t = (n * 0.5 + 0.5).clamp(0.0, 1.0);
            let grain = (hash2(x as i32, y as i32) as f32 / 255.0 - 0.5) * 0.14;
            let f = (0.75 + t * 0.45 + grain).clamp(0.4, 1.3);
            let base = match src {
                PaintSource::GrassTex => Color32::from_rgb(82, 134, 48),
                PaintSource::RockTex => Color32::from_rgb(118, 110, 102),
                PaintSource::SandTex => Color32::from_rgb(206, 188, 134),
                PaintSource::SnowTex => Color32::from_rgb(236, 240, 248),
                PaintSource::RoadTex => {
                    let stripe = if (x % 32) > 13 && (x % 32) < 18 && (y % 16) < 10 {
                        0.12
                    } else {
                        0.0
                    };
                    let v = (72.0 * f + stripe * 140.0).min(240.0) as u8;
                    let b = (76.0 * f + stripe * 80.0).min(240.0) as u8;
                    pixels[y * size + x] = Color32::from_rgb(v, v, b);
                    continue;
                }
                _ => Color32::WHITE,
            };
            pixels[y * size + x] = Color32::from_rgb(
                (base.r() as f32 * f).min(255.0) as u8,
                (base.g() as f32 * f).min(255.0) as u8,
                (base.b() as f32 * f).min(255.0) as u8,
            );
        }
    }
    (size, size, pixels)
}

impl Brush {
    /// Seleciona a fonte de pintura (cor sólida, textura procedural ou imagem carregada).
    pub fn set_paint_source(&mut self, src: PaintSource) {
        self.paint_source = src;
        match src {
            PaintSource::SolidColor => {
                self.paint_tex_name = "Cor sólida".to_string();
            }
            PaintSource::CustomTex => {
                if self.paint_tex_pixels.is_empty() {
                    self.paint_tex_name = "Nenhuma imagem".to_string();
                }
            }
            preset => {
                let (w, h, px) = generate_preset_texture(preset);
                self.paint_tex_w = w;
                self.paint_tex_h = h;
                self.paint_tex_pixels = px;
                self.paint_tex_name = preset.label().to_string();
            }
        }
    }

    /// Carrega uma textura externa (PNG/JPG) para pintura contínua no Colormap.
    pub fn load_paint_texture(&mut self, path: &str) -> Result<String, String> {
        if !isfile(path) {
            return Err(format!("Textura não encontrada: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let mut rgb = img.to_rgb8();
        if rgb.width() > 256 || rgb.height() > 256 {
            rgb = image::imageops::resize(&rgb, 256, 256, FilterType::Triangle);
        }
        let (w, h) = (rgb.width() as usize, rgb.height() as usize);
        if w == 0 || h == 0 {
            return Err("Textura vazia.".into());
        }
        self.paint_tex_w = w;
        self.paint_tex_h = h;
        self.paint_tex_pixels = rgb.pixels().map(|p| Color32::from_rgb(p[0], p[1], p[2])).collect();
        self.paint_source = PaintSource::CustomTex;
        self.paint_tex_name = basename(path);
        Ok(format!("Textura do pincel '{}' ({w}x{h}) carregada.", self.paint_tex_name))
    }

    #[inline]
    fn sample_paint(&self, wx: i32, wy: i32) -> Color32 {
        if self.paint_source == PaintSource::SolidColor || self.paint_tex_pixels.is_empty() {
            return self.paint_color;
        }
        let sc = self.paint_tex_scale.max(0.05);
        let u = (wx as f32 * sc).rem_euclid(self.paint_tex_w as f32) as usize;
        let v = (wy as f32 * sc).rem_euclid(self.paint_tex_h as f32) as usize;
        let c = self.paint_tex_pixels[v.min(self.paint_tex_h - 1) * self.paint_tex_w + u.min(self.paint_tex_w - 1)];
        if self.tint_texture { modulate(c, self.paint_color) } else { c }
    }

    #[inline]
    fn falloff(&self, t: f32) -> f32 {
        if t >= 1.0 {
            0.0
        } else if t <= self.hardness {
            1.0
        } else {
            smoothstep(0.0, 1.0, (1.0 - t) / (1.0 - self.hardness).max(1e-4))
        }
    }

    /// Aplica o pincel centrado em `(wx, wy)` (coordenadas de mundo, com wrap-around).
    /// `invert` troca elevar ↔ rebaixar (Ctrl na viewport).
    pub fn apply(&mut self, terrain: &mut Terrain, wx: f32, wy: f32, dt: f32, invert: bool) {
        let r = self.radius.max(0.5);
        let ri = r.ceil() as i32;
        let (cx, cy) = (wx.floor() as i32, wy.floor() as i32);
        let rate = self.strength * dt;
        let mode = match (self.mode, invert) {
            (BrushMode::Raise, true) => BrushMode::Lower,
            (BrushMode::Lower, true) => BrushMode::Raise,
            (m, _) => m,
        };
        let side = (2 * ri + 1) as usize;

        if mode == BrushMode::Smooth {
            // Média 3×3 calculada antes da escrita (buffer reutilizado, sem alocação).
            self.scratch.clear();
            self.scratch.resize(side * side, 0.0);
            for j in 0..side as i32 {
                for i in 0..side as i32 {
                    let (x, y) = (cx - ri + i, cy - ri + j);
                    let mut acc = 0.0;
                    for oy in -1..=1 {
                        for ox in -1..=1 {
                            acc += terrain.height_raw(x + ox, y + oy);
                        }
                    }
                    self.scratch[j as usize * side + i as usize] = acc / 9.0;
                }
            }
        }

        let paint = mode == BrushMode::Paint || self.paint_while_sculpting;
        for j in -ri..=ri {
            for i in -ri..=ri {
                let d = ((i * i + j * j) as f32).sqrt() / r;
                let w = self.falloff(d);
                if w <= 0.0 {
                    continue;
                }
                let idx = terrain.idx(cx + i, cy + j);
                let h = &mut terrain.heights[idx];
                match mode {
                    BrushMode::Raise => *h += rate * w * 40.0,
                    BrushMode::Lower => *h -= rate * w * 40.0,
                    BrushMode::Smooth => {
                        let avg = self.scratch[(j + ri) as usize * side + (i + ri) as usize];
                        *h += (avg - *h) * (rate * w * 8.0).min(1.0);
                    }
                    BrushMode::Flatten => *h += (self.flatten_height - *h) * (rate * w * 4.0).min(1.0),
                    BrushMode::Paint => {}
                }
                *h = h.clamp(0.0, MAX_HEIGHT);
                if paint {
                    let a = (rate * w * self.paint_opacity * 4.0).min(1.0);
                    let target = self.sample_paint(cx + i, cy + j);
                    terrain.albedo[idx] = mix_rgb_precise(terrain.albedo[idx], target, a);
                }
            }
        }
        if paint {
            terrain.colormap_auto = false;
        }
        terrain.update_region(cx, cy, ri + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::FlatParams;

    #[test]
    fn sculpt_modes_and_texture_paint_wrap() {
        let mut t = Terrain::new_flat(&FlatParams {
            size_exp: 6,
            base_height: 50.0,
            color: Color32::from_rgb(100, 100, 100),
        });
        let mut b = Brush::default();
        b.radius = 6.0;
        b.strength = 2.0;
        b.mode = BrushMode::Raise;
        // Borda toroidal em (0, 0) deve afetar (63, 63) também
        b.apply(&mut t, 0.0, 0.0, 0.5, false);
        assert!(t.height_raw(0, 0) > 50.0);
        assert!(t.height_raw(63, 63) > 50.0);

        b.mode = BrushMode::Paint;
        b.set_paint_source(PaintSource::GrassTex);
        b.apply(&mut t, 10.0, 10.0, 1.0, false);
        assert_ne!(t.albedo[t.idx(10, 10)], Color32::from_rgb(100, 100, 100));
    }
}
