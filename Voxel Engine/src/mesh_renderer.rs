//! Malhas 3D Wavefront (.obj) texturizadas + rasterizador em software.
//!
//! O rasterizador usa exatamente a mesma projeção do Voxel Space
//! ([`Projection`]) e escreve nos mesmos buffers column-major de cor e
//! profundidade, então modelos ficam corretamente ocultos atrás de montanhas
//! e o terreno fica oculto atrás dos modelos. Interpolação perspectivamente
//! correta de UV (1/z, u/z, v/z), recorte no plano próximo e névoa.

use crate::math::{mix_rgb, mul_rgb, modulate, wrap_delta, Vec3};
use crate::pyrsutils::prelude::*;
use crate::terrain::Terrain;
use crate::voxel_renderer::{Projection, RenderSettings};
use eframe::egui::Color32;

const NEAR: f32 = 0.5;

// ============================================================================
// Textura
// ============================================================================
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color32>,
    pub name: String,
}

impl Texture {
    pub fn load(path: &str) -> Result<Self, String> {
        if !isfile(path) {
            return Err(format!("Textura não encontrada: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir '{path}': {e}"))?;
        let rgb = img.to_rgb8();
        let (w, h) = (rgb.width() as usize, rgb.height() as usize);
        let pixels = rgb.pixels().map(|p| Color32::from_rgb(p[0], p[1], p[2])).collect();
        Ok(Self { width: w, height: h, pixels, name: basename(path) })
    }

    /// Amostragem nearest com repetição (UV do OBJ tem v=0 embaixo).
    #[inline(always)]
    pub fn sample(&self, u: f32, v: f32) -> Color32 {
        let fu = u - u.floor();
        let fv = 1.0 - (v - v.floor());
        let x = ((fu * self.width as f32) as usize).min(self.width - 1);
        let y = ((fv * self.height as f32) as usize).min(self.height - 1);
        self.pixels[y * self.width + x]
    }
}

// ============================================================================
// Malha
// ============================================================================
pub const NO_UV: u32 = u32::MAX;

pub struct Mesh {
    pub name: String,
    pub positions: Vec<Vec3>,
    pub uvs: Vec<[f32; 2]>,
    /// Cada vértice do triângulo: `[índice_posição, índice_uv]`.
    pub tris: Vec<[[u32; 2]; 3]>,
    pub min_y: f32,
    pub radius: f32,
    pub texture: Option<Texture>,
    pub base_color: Color32,
}

impl Mesh {
    fn finalize(&mut self) {
        self.min_y = self.positions.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        if !self.min_y.is_finite() {
            self.min_y = 0.0;
        }
        self.radius = self.positions.iter().map(|p| p.length()).fold(0.0, f32::max).max(0.01);
    }
}

fn resolve_index(tok: &str, count: usize) -> Result<u32, String> {
    let i: i64 = tok.parse().map_err(|_| format!("índice inválido '{tok}'"))?;
    let r = if i > 0 { i - 1 } else if i < 0 { count as i64 + i } else { -1 };
    if r < 0 {
        return Err(format!("índice fora do intervalo '{tok}'"));
    }
    Ok(r as u32)
}

/// Parser Wavefront OBJ (v, vt, f com polígonos/fan, índices negativos, mtllib).
/// Retorna a malha e a lista de bibliotecas de material referenciadas.
pub fn parse_obj(src: &str, name: &str) -> Result<(Mesh, Vec<String>), String> {
    let mut mesh = Mesh {
        name: name.to_string(),
        positions: Vec::new(),
        uvs: Vec::new(),
        tris: Vec::new(),
        min_y: 0.0,
        radius: 1.0,
        texture: None,
        base_color: Color32::from_rgb(200, 200, 200),
    };
    let mut mtllibs = Vec::new();
    let mut corners: Vec<[u32; 2]> = Vec::with_capacity(8);
    for (nl, raw) in enumerate(src.lines()) {
        let line = strip(raw);
        if line.is_empty() || startswith(line, "#") {
            continue;
        }
        let mut it = line.split_whitespace();
        let tag = it.next().unwrap_or("");
        let err = |e: String| format!("{name}:{}: {e}", nl + 1);
        match tag {
            "v" => {
                let mut c = [0.0f32; 3];
                for v in c.iter_mut() {
                    *v = it.next().and_then(|t| t.parse().ok()).ok_or_else(|| err("vértice inválido".into()))?;
                }
                mesh.positions.append_item(Vec3::new(c[0], c[1], c[2]));
            }
            "vt" => {
                let u = it.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
                let v = it.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
                mesh.uvs.append_item([u, v]);
            }
            "f" => {
                corners.clear();
                for tok in it {
                    let parts = split(tok, "/");
                    let pi = resolve_index(parts[0], len(&mesh.positions)).map_err(err)?;
                    let ti = match parts.get(1) {
                        Some(t) if !t.is_empty() => resolve_index(t, len(&mesh.uvs)).map_err(err)?,
                        _ => NO_UV,
                    };
                    corners.append_item([pi, ti]);
                }
                for k in 1..corners.len().saturating_sub(1) {
                    mesh.tris.append_item([corners[0], corners[k], corners[k + 1]]);
                }
            }
            "mtllib" => mtllibs.append_item(strip(&line[6..]).to_string()),
            _ => {}
        }
    }
    let (np, nt) = (mesh.positions.len() as u32, mesh.uvs.len() as u32);
    for tri in &mesh.tris {
        for c in tri {
            if c[0] >= np || (c[1] != NO_UV && c[1] >= nt) {
                return Err(format!("{name}: face referencia vértice inexistente"));
            }
        }
    }
    if mesh.tris.is_empty() {
        return Err(format!("{name}: nenhuma face encontrada"));
    }
    mesh.finalize();
    Ok((mesh, mtllibs))
}

/// Carrega `.obj` do disco; se houver `mtllib` com `map_Kd`, carrega a textura automaticamente.
pub fn load_obj(path: &str) -> Result<(Mesh, String), String> {
    let src = read_text(path).map_err(|e| format!("Falha ao ler '{path}': {e}"))?;
    let (mut mesh, libs) = parse_obj(&src, &basename(path))?;
    let dir = dirname(path);
    let mut info = format!("Modelo '{}' ({} tris).", mesh.name, mesh.tris.len());
    for lib in libs {
        let mtl_path = join_path(&[&dir, &lib]);
        let Ok(mtl) = read_text(&mtl_path) else { continue };
        for raw in mtl.lines() {
            let line = strip(raw);
            if startswith(line, "Kd ") {
                let v: Vec<f32> = line[3..].split_whitespace().filter_map(|t| t.parse().ok()).collect();
                if v.len() >= 3 {
                    let c = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
                    mesh.base_color = Color32::from_rgb(c(v[0]), c(v[1]), c(v[2]));
                }
            } else if startswith(line, "map_Kd") && mesh.texture.is_none() {
                if let Some(tex_name) = line.split_whitespace().last() {
                    let tex_path = join_path(&[&dirname(&mtl_path), tex_name]);
                    match Texture::load(&tex_path) {
                        Ok(t) => {
                            info = format!("{info} Textura '{}' via MTL.", t.name);
                            mesh.texture = Some(t);
                        }
                        Err(e) => info = format!("{info} ({e})"),
                    }
                }
            }
        }
    }
    Ok((mesh, info))
}

const HOUSE_OBJ: &str = "\
# Casa procedural embutida (paredes + telhado de duas águas)
v -10 0 -8\nv 10 0 -8\nv 10 0 8\nv -10 0 8
v -10 12 -8\nv 10 12 -8\nv 10 12 8\nv -10 12 8
v -10 20 0\nv 10 20 0
vt 0 0\nvt 1 0\nvt 1 0.5\nvt 0 0.5
vt 0 0.5\nvt 1 0.5\nvt 1 1\nvt 0 1
vt 0 0.35\nvt 1 0.35\nvt 0.5 0.5
f 4/1 3/2 7/3 8/4
f 2/1 1/2 5/3 6/4
f 3/1 2/2 6/3 7/4
f 1/1 4/2 8/3 5/4
f 8/5 7/6 10/7 9/8
f 6/5 5/6 9/7 10/8
f 7/9 6/10 10/11
f 5/9 8/10 9/11
";

/// Modelo de demonstração (passa pelo mesmo parser OBJ) com textura procedural.
pub fn builtin_house() -> Mesh {
    let (mut mesh, _) = parse_obj(HOUSE_OBJ, "casa_embutida.obj").expect("OBJ embutido válido");
    let (w, h) = (128usize, 128usize);
    let mut pixels = vec![Color32::BLACK; w * h];
    for y in 0..h {
        for x in 0..w {
            pixels[y * w + x] = if y < h / 2 {
                // Telhado (v ∈ [0.5, 1]): telhas vermelhas em fileiras.
                let row = y / 8;
                let edge = y % 8 == 7 || (x + row * 6) % 12 == 0;
                if edge { Color32::from_rgb(90, 30, 24) } else { Color32::from_rgb(170, 58, 40) }
            } else {
                // Paredes (v ∈ [0, 0.5]): tijolos + janelas.
                let yy = y - h / 2;
                let win = (yy > 18 && yy < 40) && ((x > 18 && x < 44) || (x > 84 && x < 110));
                let row = yy / 6;
                let mortar = yy % 6 == 5 || (x + (row % 2) * 8) % 16 == 0;
                if win {
                    Color32::from_rgb(60, 90, 120)
                } else if mortar {
                    Color32::from_rgb(200, 190, 170)
                } else {
                    Color32::from_rgb(196, 160, 112)
                }
            };
        }
    }
    mesh.texture = Some(Texture { width: w, height: h, pixels, name: "procedural".into() });
    mesh
}

const BRIDGE_OBJ: &str = "\
# Ponte procedural embutida (pilares + tabuleiro + guarda-corpo)
v -16 0 -6\nv -10 0 -6\nv -10 0 6\nv -16 0 6
v -16 6 -6\nv -10 6 -6\nv -10 6 6\nv -16 6 6
v 10 0 -6\nv 16 0 -6\nv 16 0 6\nv 10 0 6
v 10 6 -6\nv 16 6 -6\nv 16 6 6\nv 10 6 6
v -20 6 -6.5\nv 20 6 -6.5\nv 20 6 6.5\nv -20 6 6.5
v -20 8.5 -6.5\nv 20 8.5 -6.5\nv 20 8.5 6.5\nv -20 8.5 6.5
v -20 8.5 6.2\nv 20 8.5 6.2\nv 20 11.5 6.2\nv -20 11.5 6.2
v -20 8.5 -6.2\nv 20 8.5 -6.2\nv 20 11.5 -6.2\nv -20 11.5 -6.2
vt 0.02 0.02\nvt 0.98 0.02\nvt 0.98 0.48\nvt 0.02 0.48
vt 0.02 0.52\nvt 1.98 0.52\nvt 1.98 0.98\nvt 0.02 0.98
f 4/1 3/2 7/3 8/4\nf 2/1 1/2 5/3 6/4\nf 3/1 2/2 6/3 7/4\nf 1/1 4/2 8/3 5/4
f 12/1 11/2 15/3 16/4\nf 10/1 9/2 13/3 14/4\nf 11/1 10/2 14/3 15/4\nf 9/1 12/2 16/3 13/4
f 20/5 19/6 23/7 24/8\nf 18/5 17/6 21/7 22/8\nf 19/5 18/6 22/7 23/8\nf 17/5 20/6 24/7 21/8
f 24/5 23/6 22/7 21/8\nf 17/5 18/6 19/7 20/8
f 25/5 26/6 27/7 28/8\nf 29/5 30/6 31/7 32/8
";

/// Ponte de pedra/madeira embutida com textura procedural.
pub fn builtin_bridge() -> Mesh {
    let (mut mesh, _) = parse_obj(BRIDGE_OBJ, "ponte_embutida.obj").expect("OBJ embutido válido");
    let (w, h) = (128usize, 128usize);
    let mut pixels = vec![Color32::BLACK; w * h];
    for y in 0..h {
        for x in 0..w {
            pixels[y * w + x] = if y < h / 2 {
                // Tabuleiro de madeira (v ∈ [0.5, 1.0])
                let plank = x % 16 == 0 || y % 32 == 0;
                if plank { Color32::from_rgb(76, 50, 30) } else { Color32::from_rgb(138, 96, 56) }
            } else {
                // Pilares de pedra (v ∈ [0.0, 0.5])
                let yy = y - h / 2;
                let row = yy / 10;
                let mortar = yy % 10 == 0 || (x + (row % 2) * 10) % 20 == 0;
                if mortar { Color32::from_rgb(84, 82, 78) } else { Color32::from_rgb(138, 134, 126) }
            };
        }
    }
    mesh.texture = Some(Texture { width: w, height: h, pixels, name: "ponte_proc".into() });
    mesh
}

const TOWER_OBJ: &str = "\
# Torre de vigia procedural embutida
v -5 0 -5\nv 5 0 -5\nv 5 0 5\nv -5 0 5
v -4 22 -4\nv 4 22 -4\nv 4 22 4\nv -4 22 4
v -6.5 22 -6.5\nv 6.5 22 -6.5\nv 6.5 22 6.5\nv -6.5 22 6.5
v -6.5 26 -6.5\nv 6.5 26 -6.5\nv 6.5 26 6.5\nv -6.5 26 6.5
v 0 35 0
vt 0.02 0.02\nvt 0.98 0.02\nvt 0.98 0.48\nvt 0.02 0.48
vt 0.02 0.52\nvt 0.98 0.52\nvt 0.98 0.98\nvt 0.02 0.98\nvt 0.5 0.98
f 4/1 3/2 7/3 8/4\nf 2/1 1/2 5/3 6/4\nf 3/1 2/2 6/3 7/4\nf 1/1 4/2 8/3 5/4
f 12/1 11/2 15/3 16/4\nf 10/1 9/2 13/3 14/4\nf 11/1 10/2 14/3 15/4\nf 9/1 12/2 16/3 13/4
f 9/1 10/2 11/3 12/4\nf 16/1 15/2 14/3 13/4
f 16/5 15/6 17/9\nf 15/5 14/6 17/9\nf 14/5 13/6 17/9\nf 13/5 16/6 17/9
";

/// Torre de vigia embutida com textura procedural.
pub fn builtin_tower() -> Mesh {
    let (mut mesh, _) = parse_obj(TOWER_OBJ, "torre_embutida.obj").expect("OBJ embutido válido");
    let (w, h) = (128usize, 128usize);
    let mut pixels = vec![Color32::BLACK; w * h];
    for y in 0..h {
        for x in 0..w {
            pixels[y * w + x] = if y < h / 2 {
                // Telhado escuro azulado (v ∈ [0.5, 1.0])
                let row = y / 8;
                let seam = y % 8 == 0 || (x + row * 5) % 10 == 0;
                if seam { Color32::from_rgb(38, 48, 66) } else { Color32::from_rgb(66, 86, 118) }
            } else {
                // Pedra da torre (v ∈ [0.0, 0.5])
                let yy = y - h / 2;
                let row = yy / 8;
                let slit = (yy > 14 && yy < 36) && (x > 58 && x < 70);
                let seam = yy % 8 == 0 || (x + (row % 2) * 8) % 16 == 0;
                if slit {
                    Color32::from_rgb(24, 28, 36)
                } else if seam {
                    Color32::from_rgb(92, 88, 84)
                } else {
                    Color32::from_rgb(146, 140, 130)
                }
            };
        }
    }
    mesh.texture = Some(Texture { width: w, height: h, pixels, name: "torre_proc".into() });
    mesh
}

// ============================================================================
// Instâncias
// ============================================================================
#[derive(Clone, Debug)]
pub struct ModelInstance {
    pub mesh: usize,
    pub x: f32,
    pub y: f32,
    /// Deslocamento vertical (relativo ao solo se `snap_to_ground`).
    pub y_offset: f32,
    pub yaw_deg: f32,
    pub scale: f32,
    pub snap_to_ground: bool,
}

impl ModelInstance {
    pub fn new(mesh: usize, x: f32, y: f32) -> Self {
        Self { mesh, x, y, y_offset: 0.0, yaw_deg: 0.0, scale: 1.0, snap_to_ground: true }
    }

    /// Altitude da base do modelo no mundo.
    pub fn base_height(&self, terrain: &Terrain, vscale: f32) -> f32 {
        if self.snap_to_ground {
            terrain.height_at(self.x, self.y) * vscale + self.y_offset
        } else {
            self.y_offset
        }
    }
}

// ============================================================================
// Rasterizador
// ============================================================================
#[derive(Clone, Copy, Default)]
struct RV {
    lat: f32,
    up: f32,
    z: f32,
    u: f32,
    v: f32,
}

impl RV {
    #[inline(always)]
    fn lerp(a: RV, b: RV, t: f32) -> RV {
        RV {
            lat: a.lat + (b.lat - a.lat) * t,
            up: a.up + (b.up - a.up) * t,
            z: a.z + (b.z - a.z) * t,
            u: a.u + (b.u - a.u) * t,
            v: a.v + (b.v - a.v) * t,
        }
    }
}

struct RasterTarget<'a> {
    color: &'a mut [Color32],
    depth: &'a mut [f32],
    proj: &'a Projection,
    fog: Option<(f32, f32)>,
    fog_color: Color32,
}

#[derive(Clone, Copy)]
struct Shade<'a> {
    tex: Option<&'a Texture>,
    tint: Color32,
}

#[derive(Default)]
pub struct MeshRenderer {
    world: Vec<Vec3>,
    view: Vec<[f32; 3]>,
    pub backface_cull: bool,
    pub tris_drawn: usize,
}

impl MeshRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        meshes: &[Mesh],
        instances: &[ModelInstance],
        selected: Option<usize>,
        terrain: &Terrain,
        s: &RenderSettings,
        proj: &Projection,
        fog_color: Color32,
        color: &mut [Color32],
        depth: &mut [f32],
    ) {
        self.tris_drawn = 0;
        let fog = if s.fog_enabled {
            let start = s.draw_distance * s.fog_start.clamp(0.0, 0.99);
            Some((start, 1.0 / (s.draw_distance - start).max(1.0)))
        } else {
            None
        };
        let mut target = RasterTarget { color, depth, proj, fog, fog_color };
        let (size_x, size_y) = (terrain.width as f32, terrain.height as f32);
        let sun = terrain.sun_dir;

        for (ii, inst) in enumerate(instances) {
            let Some(mesh) = meshes.get(inst.mesh) else { continue };
            // Cópia mais próxima da câmera no mapa toroidal.
            let dx = wrap_delta(inst.x - proj.cam_x, size_x);
            let dy = wrap_delta(inst.y - proj.cam_y, size_y);
            let r = mesh.radius * inst.scale;
            let center_z = dx * proj.fwd.0 + dy * proj.fwd.1;
            if center_z < -r || center_z - r > s.draw_distance {
                continue;
            }
            let lateral = dx * proj.right.0 + dy * proj.right.1;
            if lateral.abs() - r > (center_z + r).max(0.0) * proj.tan_half * 1.05 {
                continue; // fora do frustum horizontal
            }
            let base = inst.base_height(terrain, s.vertical_scale) - mesh.min_y * inst.scale;
            let (ax, ay) = (proj.cam_x + dx, proj.cam_y + dy);
            let (sn, cs) = inst.yaw_deg.to_radians().sin_cos();
            let sc = inst.scale;

            // Transforma vértices (buffers reaproveitados — sem alocação no steady-state).
            self.world.clear();
            self.view.clear();
            for p in &mesh.positions {
                // OBJ: X→x, Z→y (mapa), Y→altitude (mantém a quiralidade).
                let rx = (p.x * cs - p.z * sn) * sc;
                let ry = (p.x * sn + p.z * cs) * sc;
                let wpos = Vec3::new(ax + rx, ay + ry, base + p.y * sc);
                let (relx, rely) = (wpos.x - proj.cam_x, wpos.y - proj.cam_y);
                self.view.append_item([
                    relx * proj.right.0 + rely * proj.right.1,
                    wpos.z - proj.cam_h,
                    relx * proj.fwd.0 + rely * proj.fwd.1,
                ]);
                self.world.append_item(wpos);
            }

            let tint_sel = selected == Some(ii);
            for tri in &mesh.tris {
                let [a, b, c] = [tri[0][0] as usize, tri[1][0] as usize, tri[2][0] as usize];
                let (va, vb, vc) = (self.view[a], self.view[b], self.view[c]);
                if va[2] < NEAR && vb[2] < NEAR && vc[2] < NEAR {
                    continue;
                }
                let n = self.world[b].sub(self.world[a]).cross(self.world[c].sub(self.world[a])).normalize();
                let light = 0.42 + 0.58 * n.dot(sun).abs();
                let mut tint = mul_rgb(if mesh.texture.is_some() { Color32::WHITE } else { mesh.base_color }, light);
                if tint_sel {
                    tint = mix_rgb(tint, Color32::from_rgb(255, 220, 60), 0.35);
                }
                let uv = |k: usize| {
                    let t = tri[k][1];
                    if t == NO_UV { [0.0, 0.0] } else { mesh.uvs[t as usize] }
                };
                let mk = |v: [f32; 3], t: [f32; 2]| RV { lat: v[0], up: v[1], z: v[2], u: t[0], v: t[1] };
                let verts = [mk(va, uv(0)), mk(vb, uv(1)), mk(vc, uv(2))];
                let shade = Shade { tex: mesh.texture.as_ref(), tint };
                self.clip_and_raster(&verts, shade, &mut target);
            }
        }
    }

    fn clip_and_raster(&mut self, tri: &[RV; 3], shade: Shade, t: &mut RasterTarget) {
        let mut poly = [RV::default(); 4];
        let mut n = 0;
        for i in 0..3 {
            let (a, b) = (tri[i], tri[(i + 1) % 3]);
            let (ain, bin) = (a.z >= NEAR, b.z >= NEAR);
            if ain {
                poly[n] = a;
                n += 1;
            }
            if ain != bin {
                poly[n] = RV::lerp(a, b, (NEAR - a.z) / (b.z - a.z));
                n += 1;
            }
        }
        for k in 1..n.saturating_sub(1) {
            if self.raster(poly[0], poly[k], poly[k + 1], shade, t) {
                self.tris_drawn += 1;
            }
        }
    }

    fn raster(&self, p0: RV, p1: RV, p2: RV, shade: Shade, t: &mut RasterTarget) -> bool {
        let pr = t.proj;
        let (w, h) = (pr.width as i32, pr.height as i32);
        let hw = pr.width as f32 * 0.5;
        // Projeção idêntica à do Voxel Space.
        let project = |p: RV| {
            let iz = 1.0 / p.z;
            [hw + p.lat * pr.focal * iz, pr.horizon - p.up * pr.focal * iz, iz, p.u * iz, p.v * iz]
        };
        let (a, b, c) = (project(p0), project(p1), project(p2));
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() < 1e-6 || (self.backface_cull && area > 0.0) {
            return false;
        }
        let minx = a[0].min(b[0]).min(c[0]).floor().max(0.0) as i32;
        let maxx = a[0].max(b[0]).max(c[0]).ceil().min((w - 1) as f32) as i32;
        let miny = a[1].min(b[1]).min(c[1]).floor().max(0.0) as i32;
        let maxy = a[1].max(b[1]).max(c[1]).ceil().min((h - 1) as f32) as i32;
        if minx > maxx || miny > maxy {
            return false;
        }
        let inv_area = 1.0 / area;
        // Funções de aresta: e(P) = (B−A)×(P−A); lineares em y → incremento constante.
        let edge = |p: &[f32; 5], q: &[f32; 5], x: f32, y: f32| (q[0] - p[0]) * (y - p[1]) - (q[1] - p[1]) * (x - p[0]);
        let (s0, s1) = ((c[0] - b[0]) * inv_area, (a[0] - c[0]) * inv_area);
        let hh = pr.height;
        for x in minx..=maxx {
            let px = x as f32 + 0.5;
            let py0 = miny as f32 + 0.5;
            let mut w0 = edge(&b, &c, px, py0) * inv_area;
            let mut w1 = edge(&c, &a, px, py0) * inv_area;
            let base = x as usize * hh;
            for y in miny..=maxy {
                let w2 = 1.0 - w0 - w1;
                if w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5 {
                    let iz = w0 * a[2] + w1 * b[2] + w2 * c[2];
                    let z = 1.0 / iz;
                    let idx = base + y as usize;
                    if z < t.depth[idx] {
                        let mut col = match shade.tex {
                            Some(tex) => {
                                let u = (w0 * a[3] + w1 * b[3] + w2 * c[3]) * z;
                                let v = (w0 * a[4] + w1 * b[4] + w2 * c[4]) * z;
                                modulate(tex.sample(u, v), shade.tint)
                            }
                            None => shade.tint,
                        };
                        if let Some((start, inv)) = t.fog {
                            let f = (z - start) * inv;
                            if f > 0.0 {
                                col = mix_rgb(col, t.fog_color, f);
                            }
                        }
                        t.color[idx] = col;
                        t.depth[idx] = z;
                    }
                }
                w0 += s0;
                w1 += s1;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::FlatParams;
    use crate::voxel_renderer::{Camera, VoxelRenderer};
    use crate::sky::Sky;

    #[test]
    fn parse_builtin_models_and_negative_indices() {
        let house = builtin_house();
        let bridge = builtin_bridge();
        let tower = builtin_tower();
        assert!(!house.tris.is_empty());
        assert!(!bridge.tris.is_empty());
        assert!(!tower.tris.is_empty());

        let neg = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nf -3/-3 -2/-2 -1/-1\n";
        let (m, _) = parse_obj(neg, "neg.obj").expect("índices negativos válidos");
        assert_eq!(m.tris.len(), 1);
    }

    #[test]
    fn zbuffer_occludes_model_behind_mountain() {
        let mut t = Terrain::new_flat(&FlatParams {
            size_exp: 7,
            base_height: 10.0,
            color: Color32::from_rgb(80, 120, 60),
        });
        // Montanha entre câmera (x=20) e modelo (x=90)
        for y in 0..128 {
            for x in 40..60 {
                let i = t.idx(x, y);
                t.heights[i] = 260.0;
            }
        }
        t.rebuild_all();
        let sky = Sky::procedural(3);
        let cam = Camera { x: 20.0, y: 64.0, height: 80.0, yaw: 0.0, pitch: 0.0, fov_deg: 75.0 };
        let mut s = RenderSettings::default();
        s.water_enabled = false;
        s.threads = 1;
        let mut vr = VoxelRenderer::new(64, 64);
        let proj = vr.render(&t, &sky, &cam, &s, None, 0.0);

        let meshes = vec![builtin_house()];
        let inst = vec![ModelInstance::new(0, 90.0, 64.0)];
        let mut mr = MeshRenderer::default();
        let before = vr.color.clone();
        mr.render(&meshes, &inst, None, &t, &s, &proj, sky.fog_color, &mut vr.color, &mut vr.depth);
        // Atrás da montanha alta, nenhum pixel do modelo pode sobrescrever o framebuffer
        assert_eq!(before, vr.color);
    }
}
