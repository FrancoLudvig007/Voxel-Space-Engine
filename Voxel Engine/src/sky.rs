//! Céu panorâmico/cilíndrico projetado acima do horizonte.
//!
//! A imagem é armazenada em ordem de colunas (column-major) porque o
//! renderizador Voxel Space desenha a tela coluna a coluna: cada coluna da tela
//! lê uma única coluna da textura do céu de forma sequencial na memória.

use crate::math::{mix_rgb, smoothstep};
use crate::pyrsutils::prelude::*;
use crate::terrain::Perlin;
use eframe::egui::Color32;
use image::imageops::FilterType;

pub struct Sky {
    pub width: usize,
    pub height: usize,
    pub pixels_cm: Vec<Color32>,
    /// Linha (fração da altura) onde fica o horizonte na imagem.
    /// 0.5 = panorama equiretangular; 1.0 = apenas hemisfério superior.
    pub horizon_frac: f32,
    /// Cor média junto ao horizonte — usada como cor da névoa (fusão perfeita).
    pub fog_color: Color32,
    pub name: String,
}

impl Sky {
    fn from_rows(w: usize, h: usize, rows: &[Color32], horizon_frac: f32, name: String) -> Self {
        let mut pixels_cm = vec![Color32::BLACK; w * h];
        for y in 0..h {
            let src = &rows[y * w..(y + 1) * w];
            for (x, &c) in src.iter().enumerate() {
                pixels_cm[x * h + y] = c;
            }
        }
        let mut sky = Self { width: w, height: h, pixels_cm, horizon_frac, fog_color: Color32::GRAY, name };
        sky.recompute_fog();
        sky
    }

    pub fn set_horizon_frac(&mut self, f: f32) {
        self.horizon_frac = f.clamp(0.05, 1.0);
        self.recompute_fog();
    }

    fn recompute_fog(&mut self) {
        let row = ((self.horizon_frac * (self.height - 1) as f32) as isize - 2)
            .clamp(0, self.height as isize - 1) as usize;
        let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
        for x in 0..self.width {
            let c = self.pixels_cm[x * self.height + row];
            r += c.r() as u64;
            g += c.g() as u64;
            b += c.b() as u64;
        }
        let n = self.width.max(1) as u64;
        self.fog_color = Color32::from_rgb((r / n) as u8, (g / n) as u8, (b / n) as u8);
    }

    /// Céu procedural (gradiente atmosférico + nuvens fBm periódicas no eixo horizontal).
    pub fn procedural(semente: u64) -> Self {
        let (w, h) = (1024usize, 512usize);
        let perlin = Perlin::new(semente);
        let horizon = h as f32 * 0.5;
        let zenith = Color32::from_rgb(36, 88, 176);
        let horizon_c = Color32::from_rgb(184, 208, 232);
        let haze = Color32::from_rgb(140, 156, 172);
        let cloud = Color32::from_rgb(246, 246, 252);
        let mut rows = vec![Color32::BLACK; w * h];
        for y in range(h as i64) {
            let y = y as usize;
            let e = (horizon - y as f32) / horizon; // 1 = zênite, 0 = horizonte, <0 abaixo
            for x in 0..w {
                rows[y * w + x] = if e >= 0.0 {
                    let base = mix_rgb(horizon_c, zenith, e.powf(0.55));
                    let n = perlin.fbm(x as f32 / w as f32, y as f32 / h as f32, 6, 5, 0.55, false);
                    let dens = smoothstep(0.02, 0.40, n) * smoothstep(0.0, 0.12, e) * (1.0 - e * 0.6);
                    mix_rgb(base, cloud, dens * 0.9)
                } else {
                    mix_rgb(horizon_c, haze, (-e * 2.0).min(1.0))
                };
            }
        }
        Self::from_rows(w, h, &rows, 0.5, "Céu procedural".to_string())
    }

    /// Carrega PNG/JPG externo como panorama cilíndrico.
    pub fn load(path: &str, horizon_frac: f32) -> Result<Self, String> {
        if !isfile(path) {
            return Err(format!("Arquivo não encontrado: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let mut rgb = img.to_rgb8();
        if rgb.width() > 4096 {
            let nh = (rgb.height() as f32 * 4096.0 / rgb.width() as f32) as u32;
            rgb = image::imageops::resize(&rgb, 4096, nh.max(2), FilterType::Triangle);
        }
        let (w, h) = (rgb.width() as usize, rgb.height() as usize);
        if w < 2 || h < 2 {
            return Err("Imagem de céu muito pequena.".into());
        }
        let rows: Vec<Color32> = rgb.pixels().map(|p| Color32::from_rgb(p[0], p[1], p[2])).collect();
        Ok(Self::from_rows(w, h, &rows, horizon_frac, basename(path)))
    }
}
