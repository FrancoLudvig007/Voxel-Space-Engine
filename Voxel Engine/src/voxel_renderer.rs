//! Núcleo do motor Voxel Space (estilo Comanche / Delta Force — NovaLogic).
//!
//! Renderização por coluna, front-to-back, com `y_buffer` por coluna (cada
//! pixel é escrito exatamente uma vez → zero overdraw), pitch por deslocamento
//! do horizonte (y-shearing), LOD por passo de raio crescente, névoa, plano de
//! água translúcido e céu cilíndrico. Os buffers de cor e profundidade são
//! **column-major** (`idx = x * altura + y`) para que o laço quente escreva de
//! forma sequencial na memória, e as colunas são divididas entre threads
//! (`std::thread::scope`) sem nenhuma alocação por frame.

use crate::math::{mix_rgb, modulate, mul_rgb, worker_threads};
use crate::pyrsutils::prelude::{basename, isfile};
use crate::sky::Sky;
use crate::terrain::{Cell, Perlin, Terrain};
use eframe::egui::{Color32, ColorImage};
use image::imageops::FilterType;
use std::f32::consts::{FRAC_PI_2, TAU};
use std::sync::Arc;

pub const WATER_TEX_SIZE: usize = 128;
const WATER_TEX_MASK: usize = WATER_TEX_SIZE - 1;
const WATER_TEX_SHIFT: u32 = 7;

/// Textura tileable (128×128) aplicada sobre o plano de água.
pub struct WaterTexture {
    pub pixels: Vec<Color32>,
    pub name: String,
}

impl WaterTexture {
    /// Gera uma textura de superfície aquática contínua (ondulações + cáusticas).
    pub fn procedural() -> Self {
        let perlin = Perlin::new(4242);
        let size = WATER_TEX_SIZE;
        let inv = 1.0 / size as f32;
        let mut pixels = vec![Color32::WHITE; size * size];
        for y in 0..size {
            for x in 0..size {
                let n = perlin.fbm(x as f32 * inv, y as f32 * inv, 6, 4, 0.55, true);
                let crest = (n * 0.5 + 0.5).clamp(0.0, 1.0);
                let l = 175.0 + crest * 80.0;
                pixels[y * size + x] = Color32::from_rgb(
                    (l * 0.88) as u8,
                    (l * 0.96) as u8,
                    l.min(255.0) as u8,
                );
            }
        }
        Self { pixels, name: "Água procedural".to_string() }
    }

    /// Carrega uma textura de água externa (redimensionada para 128×128 tileable).
    pub fn load(path: &str) -> Result<Self, String> {
        if !isfile(path) {
            return Err(format!("Arquivo não encontrado: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let rgb = image::imageops::resize(
            &img.to_rgb8(),
            WATER_TEX_SIZE as u32,
            WATER_TEX_SIZE as u32,
            FilterType::Triangle,
        );
        let pixels = rgb.pixels().map(|p| Color32::from_rgb(p[0], p[1], p[2])).collect();
        Ok(Self { pixels, name: basename(path) })
    }
}

// ============================================================================
// Câmera e configurações
// ============================================================================
#[derive(Clone, Debug)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    /// Rotação horizontal (radianos). 0 = olhando para +x.
    pub yaw: f32,
    /// Deslocamento do horizonte em frações da altura da tela (+ = olhar para cima).
    pub pitch: f32,
    pub fov_deg: f32,
}

impl Camera {
    #[inline]
    pub fn forward(&self) -> (f32, f32) {
        (self.yaw.cos(), self.yaw.sin())
    }
    #[inline]
    pub fn right(&self) -> (f32, f32) {
        (-self.yaw.sin(), self.yaw.cos())
    }
}

#[derive(Clone, Debug)]
pub struct RenderSettings {
    pub draw_distance: f32,
    pub vertical_scale: f32,
    /// Incremento do passo do raio a cada amostra (LOD: amostragem mais esparsa ao longe).
    pub lod_factor: f32,
    pub near_step: f32,
    pub fog_enabled: bool,
    /// Início da névoa como fração da distância de visão.
    pub fog_start: f32,
    pub water_enabled: bool,
    /// Altitude do plano de água (unidades de mundo, já com escala vertical).
    pub water_level: f32,
    pub water_color: Color32,
    /// Profundidade a partir da qual a água fica totalmente opaca.
    pub water_opaque_depth: f32,
    pub water_reflectivity: f32,
    pub water_waves: bool,
    pub render_scale: f32,
    /// 0 = automático.
    pub threads: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            draw_distance: 1800.0,
            vertical_scale: 1.0,
            lod_factor: 0.012,
            near_step: 0.5,
            fog_enabled: true,
            fog_start: 0.55,
            water_enabled: true,
            water_level: 62.0,
            water_color: Color32::from_rgb(24, 78, 120),
            water_opaque_depth: 22.0,
            water_reflectivity: 0.35,
            water_waves: true,
            render_scale: 0.75,
            threads: 0,
        }
    }
}

/// Destaque do pincel desenhado diretamente sobre o terreno.
#[derive(Clone, Copy, Debug)]
pub struct BrushOverlay {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: Color32,
}

/// Projeção compartilhada entre o Voxel Space e o rasterizador de malhas:
/// `sx = W/2 + lateral·f/z`, `sy = horizonte + (h_cam − h)·f/z`.
/// Como ambos usam a mesma fórmula e a mesma métrica de profundidade (`z` ao
/// longo do vetor frontal), o Z-buffer fica perfeitamente sincronizado.
#[derive(Clone, Copy, Debug)]
pub struct Projection {
    pub width: usize,
    pub height: usize,
    pub cam_x: f32,
    pub cam_y: f32,
    pub cam_h: f32,
    pub yaw: f32,
    pub fwd: (f32, f32),
    pub right: (f32, f32),
    pub tan_half: f32,
    pub focal: f32,
    pub horizon: f32,
}

impl Projection {
    pub fn new(cam: &Camera, w: usize, h: usize) -> Self {
        let tan_half = (cam.fov_deg.clamp(20.0, 150.0).to_radians() * 0.5).tan();
        Self {
            width: w,
            height: h,
            cam_x: cam.x,
            cam_y: cam.y,
            cam_h: cam.height,
            yaw: cam.yaw,
            fwd: cam.forward(),
            right: cam.right(),
            tan_half,
            focal: (w as f32 * 0.5) / tan_half,
            horizon: h as f32 * 0.5 + cam.pitch * h as f32,
        }
    }

    /// Fator lateral da coluna (`tan` do ângulo da coluna relativo ao eixo frontal).
    #[inline(always)]
    pub fn column_k(&self, col: usize) -> f32 {
        ((col as f32 + 0.5) / self.width as f32 * 2.0 - 1.0) * self.tan_half
    }

    /// Direção do raio da coluna, escalada para que `z` seja profundidade (sem fisheye).
    #[inline(always)]
    pub fn ray_dir(&self, col: usize) -> (f32, f32) {
        let k = self.column_k(col);
        (self.fwd.0 + self.right.0 * k, self.fwd.1 + self.right.1 * k)
    }
}

// ============================================================================
// Contexto imutável de um frame (compartilhado entre threads)
// ============================================================================
#[derive(Clone, Copy)]
struct WaterCtx {
    level: f32,
    surface: Color32,
    inv_opaque: f32,
    waves: bool,
}

#[derive(Clone, Copy)]
struct BrushCtx {
    bx: i32,
    by: i32,
    r2_in: i32,
    r2_out: i32,
    color: Color32,
}

struct FrameCtx<'a> {
    cells: &'a [Cell],
    mask_x: usize,
    mask_y: usize,
    shift_y: u32,
    size_x: i32,
    size_y: i32,
    proj: Projection,
    vscale: f32,
    draw_dist: f32,
    lod: f32,
    near: f32,
    fog: Option<(f32, f32)>, // (início, 1/faixa)
    fog_color: Color32,
    water: Option<WaterCtx>,
    water_tex: &'a [Color32],
    brush: Option<BrushCtx>,
    sky: &'a Sky,
    sky_rows: &'a [u32],
    wave: &'a [f32; 256],
    t_off: usize,
}

impl FrameCtx<'_> {
    #[inline(always)]
    fn cell(&self, ix: i32, iy: i32) -> Cell {
        let i = (((iy as usize) & self.mask_y) << self.shift_y) | ((ix as usize) & self.mask_x);
        // SAFETY-free: índice sempre < len graças às máscaras de potência de 2.
        self.cells[i]
    }

    #[inline(always)]
    fn wrap_i(d: i32, size: i32) -> i32 {
        let r = d.rem_euclid(size);
        if r > size / 2 {
            r - size
        } else {
            r
        }
    }

    /// Altura e cor da superfície visível (terreno ou água translúcida com textura).
    #[inline(always)]
    fn surface(&self, ix: i32, iy: i32) -> (f32, Color32) {
        let cell = self.cell(ix, iy);
        let th = cell.h * self.vscale;
        match self.water {
            Some(w) if th < w.level => {
                let depth = (w.level - th) * w.inv_opaque;
                let d = depth.min(1.0);
                // Ease-out: margens rasas transparentes, áreas profundas opacas.
                let alpha = d * (2.0 - d);
                let (ox, oy) = if w.waves {
                    ((self.t_off as i32) / 2, (self.t_off as i32) / 3)
                } else {
                    (0, 0)
                };
                let wx_i = (ix.wrapping_add(ox) as usize) & WATER_TEX_MASK;
                let wy_i = (iy.wrapping_add(oy) as usize) & WATER_TEX_MASK;
                let tex_c = self.water_tex[(wy_i << WATER_TEX_SHIFT) | wx_i];
                let mut s = modulate(tex_c, w.surface);
                if w.waves {
                    let a = self.wave[((ix as usize).wrapping_mul(5).wrapping_add(self.t_off)) & 255];
                    let b = self.wave[((iy as usize).wrapping_mul(3).wrapping_add(self.t_off * 2 / 3)) & 255];
                    s = mul_rgb(s, 1.0 + 0.06 * (a + b));
                }
                (w.level, mix_rgb(cell.color, s, alpha))
            }
            _ => (th, cell.color),
        }
    }
}

/// Renderiza uma coluna completa da tela (terreno front-to-back + céu).
fn render_column(ctx: &FrameCtx, col: usize, out_c: &mut [Color32], out_d: &mut [f32]) {
    let p = &ctx.proj;
    let (dx, dy) = p.ray_dir(col);
    let mut ybuf = out_c.len() as i32; // limite inferior ainda não desenhado (exclusivo)
    let mut z = 1.0f32;
    let mut dz = ctx.near;

    while z < ctx.draw_dist {
        let wx = p.cam_x + dx * z;
        let wy = p.cam_y + dy * z;
        let (ix, iy) = (wx.floor() as i32, wy.floor() as i32);
        let (surf, mut color) = ctx.surface(ix, iy);
        let sy = (p.horizon + (p.cam_h - surf) * p.focal / z) as i32;

        if sy < ybuf {
            let top = sy.max(0);
            if let Some(b) = ctx.brush {
                let ddx = FrameCtx::wrap_i(ix - b.bx, ctx.size_x);
                let ddy = FrameCtx::wrap_i(iy - b.by, ctx.size_y);
                let d2 = ddx * ddx + ddy * ddy;
                if d2 <= b.r2_out {
                    color = if d2 >= b.r2_in { mix_rgb(color, b.color, 0.75) } else { mix_rgb(color, b.color, 0.12) };
                }
            }
            if let Some((start, inv)) = ctx.fog {
                let f = (z - start) * inv;
                if f > 0.0 {
                    color = mix_rgb(color, ctx.fog_color, f);
                }
            }
            let (a, b) = (top as usize, ybuf as usize);
            out_c[a..b].fill(color);
            out_d[a..b].fill(z);
            ybuf = top;
            if ybuf <= 0 {
                return; // coluna completa: descarte imediato do restante do raio
            }
        }
        z += dz;
        dz += ctx.lod;
    }

    // Céu cilíndrico: u acompanha o yaw + ângulo da coluna; v vem da tabela por linha.
    let sky = ctx.sky;
    let ang = p.yaw + p.column_k(col).atan();
    let u = (((ang / TAU).rem_euclid(1.0)) * sky.width as f32) as usize;
    let column = &sky.pixels_cm[u.min(sky.width - 1) * sky.height..][..sky.height];
    for y in 0..ybuf as usize {
        out_c[y] = column[ctx.sky_rows[y] as usize];
        out_d[y] = f32::INFINITY;
    }
}

// ============================================================================
// Renderer
// ============================================================================
pub struct VoxelRenderer {
    pub width: usize,
    pub height: usize,
    /// Cor column-major (`x * height + y`).
    pub color: Vec<Color32>,
    /// Profundidade column-major (mesma métrica `z` usada pelo rasterizador de malhas).
    pub depth: Vec<f32>,
    pub water_tex: WaterTexture,
    sky_rows: Vec<u32>,
    wave: [f32; 256],
    /// Imagem de saída reaproveitada entre frames (sem realocar quando o egui já a liberou).
    output: Arc<ColorImage>,
}

impl VoxelRenderer {
    pub fn new(w: usize, h: usize) -> Self {
        let mut wave = [0.0f32; 256];
        for (i, v) in wave.iter_mut().enumerate() {
            *v = (i as f32 / 256.0 * TAU).sin();
        }
        let mut r = Self {
            width: 0,
            height: 0,
            color: Vec::new(),
            depth: Vec::new(),
            water_tex: WaterTexture::procedural(),
            sky_rows: Vec::new(),
            wave,
            output: Arc::new(ColorImage::new([1, 1], Color32::BLACK)),
        };
        r.resize(w, h);
        r
    }

    pub fn load_water_texture(&mut self, path: &str) -> Result<String, String> {
        let tex = WaterTexture::load(path)?;
        let nome = tex.name.clone();
        self.water_tex = tex;
        Ok(format!("Textura de água '{nome}' carregada."))
    }

    pub fn reset_water_texture(&mut self) {
        self.water_tex = WaterTexture::procedural();
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let (w, h) = (w.max(16), h.max(16));
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            self.color.resize(w * h, Color32::BLACK);
            self.depth.resize(w * h, f32::INFINITY);
            self.sky_rows.resize(h, 0);
        }
    }

    pub fn render(
        &mut self,
        terrain: &Terrain,
        sky: &Sky,
        cam: &Camera,
        s: &RenderSettings,
        brush: Option<BrushOverlay>,
        time: f32,
    ) -> Projection {
        let (w, h) = (self.width, self.height);
        let proj = Projection::new(cam, w, h);

        // Tabela de linhas do céu (mapeamento angular, segue o pitch) — sem alocação.
        let hr = sky.horizon_frac * (sky.height - 1) as f32;
        let below = (sky.height - 1) as f32 - hr;
        for (y, row) in self.sky_rows.iter_mut().enumerate() {
            let ang = ((proj.horizon - (y as f32 + 0.5)) / proj.focal).atan();
            let r = if ang >= 0.0 { hr - ang / FRAC_PI_2 * hr } else { hr + (-ang) / FRAC_PI_2 * below };
            *row = r.round().clamp(0.0, (sky.height - 1) as f32) as u32;
        }

        let fog = if s.fog_enabled {
            let start = s.draw_distance * s.fog_start.clamp(0.0, 0.99);
            Some((start, 1.0 / (s.draw_distance - start).max(1.0)))
        } else {
            None
        };
        let water = s.water_enabled.then(|| WaterCtx {
            level: s.water_level,
            surface: mix_rgb(s.water_color, sky.fog_color, s.water_reflectivity),
            inv_opaque: 1.0 / s.water_opaque_depth.max(0.5),
            waves: s.water_waves,
        });
        let brush = brush.map(|b| {
            let r_out = b.radius.max(1.0);
            let r_in = (r_out - (r_out * 0.12).max(1.5)).max(0.0);
            BrushCtx {
                bx: b.x.floor() as i32,
                by: b.y.floor() as i32,
                r2_in: (r_in * r_in) as i32,
                r2_out: (r_out * r_out) as i32,
                color: b.color,
            }
        });

        let ctx = FrameCtx {
            cells: &terrain.cells,
            mask_x: terrain.mask_x,
            mask_y: terrain.mask_y,
            shift_y: terrain.shift_y,
            size_x: terrain.width as i32,
            size_y: terrain.height as i32,
            proj,
            vscale: s.vertical_scale,
            draw_dist: s.draw_distance,
            lod: s.lod_factor.max(0.0),
            near: s.near_step.max(0.05),
            fog,
            fog_color: sky.fog_color,
            water,
            water_tex: &self.water_tex.pixels,
            brush,
            sky,
            sky_rows: &self.sky_rows,
            wave: &self.wave,
            t_off: (time * 24.0) as usize,
        };

        let threads = if s.threads == 0 { worker_threads() } else { s.threads }.clamp(1, w);
        let cols_per = w.div_ceil(threads);
        let chunk = cols_per * h;
        if threads == 1 {
            for (col, (c, d)) in self.color.chunks_mut(h).zip(self.depth.chunks_mut(h)).enumerate() {
                render_column(&ctx, col, c, d);
            }
        } else {
            let ctx = &ctx;
            std::thread::scope(|sc| {
                for (ti, (cc, dd)) in self.color.chunks_mut(chunk).zip(self.depth.chunks_mut(chunk)).enumerate() {
                    sc.spawn(move || {
                        for (j, (c, d)) in cc.chunks_mut(h).zip(dd.chunks_mut(h)).enumerate() {
                            render_column(ctx, ti * cols_per + j, c, d);
                        }
                    });
                }
            });
        }
        proj
    }

    /// Converte coordenada de tela em coordenada de mundo refazendo o raymarch
    /// exatamente como o renderizador (retorna o ponto da superfície visível).
    pub fn pick(&self, terrain: &Terrain, s: &RenderSettings, proj: &Projection, sx: f32, sy: f32) -> Option<(f32, f32)> {
        if sx < 0.0 || sy < 0.0 || sx >= proj.width as f32 || sy >= proj.height as f32 {
            return None;
        }
        let (dx, dy) = proj.ray_dir(sx as usize);
        let mut z = 1.0f32;
        let mut dz = s.near_step.max(0.05);
        while z < s.draw_distance {
            let wx = proj.cam_x + dx * z;
            let wy = proj.cam_y + dy * z;
            let i = terrain.idx(wx.floor() as i32, wy.floor() as i32);
            let mut surf = terrain.cells[i].h * s.vertical_scale;
            if s.water_enabled && surf < s.water_level {
                surf = s.water_level;
            }
            if proj.horizon + (proj.cam_h - surf) * proj.focal / z <= sy {
                return Some((wx, wy));
            }
            z += dz;
            dz += s.lod_factor.max(0.0);
        }
        None
    }

    /// Transpõe (em blocos, amigável ao cache) o buffer column-major para a
    /// imagem row-major do egui. Reaproveita a alocação anterior sempre que o
    /// egui já tiver liberado sua referência ao `Arc`.
    pub fn present(&mut self) -> Arc<ColorImage> {
        let (w, h) = (self.width, self.height);
        if Arc::get_mut(&mut self.output).is_none() {
            self.output = Arc::new(ColorImage::new([w, h], Color32::BLACK));
        }
        let img = Arc::get_mut(&mut self.output).expect("Arc exclusivo");
        if img.size != [w, h] {
            img.size = [w, h];
            img.pixels.resize(w * h, Color32::BLACK);
        }
        const B: usize = 32;
        let src = &self.color;
        let dst = &mut img.pixels;
        for bx in (0..w).step_by(B) {
            let bx1 = (bx + B).min(w);
            for by in (0..h).step_by(B) {
                let by1 = (by + B).min(h);
                for x in bx..bx1 {
                    let col = &src[x * h..x * h + h];
                    for y in by..by1 {
                        dst[y * w + x] = col[y];
                    }
                }
            }
        }
        self.output.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::FlatParams;

    #[test]
    fn front_to_back_occlusion_zero_sky_under_mountain() {
        let mut t = Terrain::new_flat(&FlatParams {
            size_exp: 6,
            base_height: 20.0,
            color: Color32::from_rgb(60, 100, 40),
        });
        // Parede alta na frente da câmera em x = 35..45
        for y in 0..64 {
            for x in 35..45 {
                let i = t.idx(x, y);
                t.heights[i] = 220.0;
                t.albedo[i] = Color32::from_rgb(180, 40, 40);
            }
        }
        t.rebuild_all();
        let sky = Sky::procedural(1);
        let cam = Camera { x: 10.0, y: 32.0, height: 80.0, yaw: 0.0, pitch: 0.0, fov_deg: 75.0 };
        let mut s = RenderSettings::default();
        s.water_enabled = false;
        s.fog_enabled = false;
        s.threads = 1;
        let mut r = VoxelRenderer::new(64, 64);
        let proj = r.render(&t, &sky, &cam, &s, None, 0.0);
        // Coluna central deve atingir a parede (depth finito na linha do horizonte)
        let center_idx = 32 * 64 + 32;
        assert!(r.depth[center_idx] < 50.0);
        // Pick no centro da tela deve retornar algo próximo à parede
        let hit = r.pick(&t, &s, &proj, 32.0, 32.0);
        assert!(hit.is_some());
    }

    #[test]
    fn water_plane_depth_blending() {
        let mut t = Terrain::new_flat(&FlatParams {
            size_exp: 6,
            base_height: 10.0,
            color: Color32::from_rgb(200, 180, 100),
        });
        // Margem rasa em x=20..30 (altura 55 para water_level 60)
        for y in 0..64 {
            for x in 20..30 {
                let i = t.idx(x, y);
                t.heights[i] = 55.0;
            }
        }
        t.rebuild_all();
        let sky = Sky::procedural(2);
        let mut s = RenderSettings::default();
        s.water_enabled = true;
        s.water_level = 60.0;
        s.water_opaque_depth = 30.0;
        s.water_color = Color32::from_rgb(0, 80, 200);
        s.water_reflectivity = 0.0;
        s.fog_enabled = false;
        s.threads = 1;
        let cam = Camera { x: 15.0, y: 32.0, height: 110.0, yaw: 0.0, pitch: -0.3, fov_deg: 75.0 };
        let mut r = VoxelRenderer::new(64, 64);
        r.render(&t, &sky, &cam, &s, None, 0.0);
        // Todos os pixels abaixo do horizonte devem ter sido preenchidos com profundidade <= draw_distance
        assert!(r.depth[32 * 64 + 60] < s.draw_distance);
    }
}
