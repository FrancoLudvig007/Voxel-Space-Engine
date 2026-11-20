use crate::entities::{SceneNode, Triangle3D, BLOCK_SIZE, MAP_H, MAP_W};
use crate::math3d::{Camera6DOF, TexCoord, Vec3};
use crate::pyrsutils::prelude::*;
use crate::textures::TextureManager;
use eframe::egui;

pub struct SoftwareRasterizer {
    pub fb_width: usize,
    pub fb_height: usize,
    pub color_buf: Vec<egui::Color32>,
    pub z_buf: Vec<f32>,
    pub cached_static_tris: Vec<Triangle3D>,
    pub frame_tris: Vec<Triangle3D>,
    pub static_dirty: bool,
    pub show_wireframe: bool,
}

impl SoftwareRasterizer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            fb_width: w,
            fb_height: h,
            color_buf: vec![egui::Color32::from_rgb(5, 5, 10); w * h],
            z_buf: vec![f32::INFINITY; w * h],
            cached_static_tris: Vec::with_capacity(2048),
            frame_tris: Vec::with_capacity(2560),
            static_dirty: true,
            show_wireframe: false,
        }
    }

    pub fn set_resolution(&mut self, w: usize, h: usize) {
        if self.fb_width != w || self.fb_height != h {
            self.fb_width = w;
            self.fb_height = h;
            self.color_buf
                .resize(w * h, egui::Color32::from_rgb(5, 5, 10));
            self.z_buf.resize(w * h, f32::INFINITY);
        }
    }

    pub fn rebuild_static_mesh(&mut self, map: &Matrix<SceneNode>) {
        self.cached_static_tris.clear();
        for z in range(MAP_H as i64) {
            for x in range(MAP_W as i64) {
                let (zu, xu) = (z as usize, x as usize);
                let ent = map[(zu, xu)].as_entity();
                let b = ent.base();
                let cx = xu as f32 * BLOCK_SIZE;
                let cz = zu as f32 * BLOCK_SIZE;
                let x0 = cx - BLOCK_SIZE * 0.5;
                let x1 = cx + BLOCK_SIZE * 0.5;
                let z0 = cz - BLOCK_SIZE * 0.5;
                let z1 = cz + BLOCK_SIZE * 0.5;
                let (yf, yc) = (b.y_floor, b.y_ceil);

                if ent.is_empty_space() {
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x0, yf, z1),
                        Vec3::new(x1, yf, z1),
                        Vec3::new(x1, yf, z0),
                        Vec3::new(x0, yf, z0),
                        b.floor_tex,
                        false,
                    );
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x0, yc, z0),
                        Vec3::new(x1, yc, z0),
                        Vec3::new(x1, yc, z1),
                        Vec3::new(x0, yc, z1),
                        b.ceil_tex,
                        false,
                    );
                    continue;
                }

                if zu > 0 && map[(zu - 1, xu)].as_entity().is_empty_space() {
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x0, yc, z0),
                        Vec3::new(x1, yc, z0),
                        Vec3::new(x1, yf, z0),
                        Vec3::new(x0, yf, z0),
                        b.wall_tex,
                        false,
                    );
                }
                if zu + 1 < MAP_H && map[(zu + 1, xu)].as_entity().is_empty_space() {
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x1, yc, z1),
                        Vec3::new(x0, yc, z1),
                        Vec3::new(x0, yf, z1),
                        Vec3::new(x1, yf, z1),
                        b.wall_tex,
                        false,
                    );
                }
                if xu > 0 && map[(zu, xu - 1)].as_entity().is_empty_space() {
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x0, yc, z1),
                        Vec3::new(x0, yc, z0),
                        Vec3::new(x0, yf, z0),
                        Vec3::new(x0, yf, z1),
                        b.wall_tex,
                        false,
                    );
                }
                if xu + 1 < MAP_W && map[(zu, xu + 1)].as_entity().is_empty_space() {
                    Triangle3D::append_quad(
                        &mut self.cached_static_tris,
                        Vec3::new(x1, yc, z0),
                        Vec3::new(x1, yc, z1),
                        Vec3::new(x1, yf, z1),
                        Vec3::new(x1, yf, z0),
                        b.wall_tex,
                        false,
                    );
                }
            }
        }
        self.static_dirty = false;
    }

    fn clip_triangle_z(tri: &Triangle3D) -> ([Triangle3D; 2], usize) {
        let mut in_p = [Vec3::new(0.0, 0.0, 0.0); 3];
        let mut in_uv = [TexCoord::new(0.0, 0.0); 3];
        let mut out_p = [Vec3::new(0.0, 0.0, 0.0); 3];
        let mut out_uv = [TexCoord::new(0.0, 0.0); 3];
        let (mut in_c, mut out_c) = (0usize, 0usize);

        for i in 0..3 {
            if tri.p[i].z >= 0.15 {
                in_p[in_c] = tri.p[i];
                in_uv[in_c] = tri.uv[i];
                in_c += 1;
            } else {
                out_p[out_c] = tri.p[i];
                out_uv[out_c] = tri.uv[i];
                out_c += 1;
            }
        }

        let dummy = *tri;
        if in_c == 0 {
            return ([dummy, dummy], 0);
        }
        if in_c == 3 {
            return ([*tri, dummy], 1);
        }

        let intersect = |pa: Vec3, pb: Vec3, uva: TexCoord, uvb: TexCoord| -> (Vec3, TexCoord) {
            let t = (0.15 - pa.z) / (pb.z - pa.z);
            (
                Vec3::new(pa.x + t * (pb.x - pa.x), pa.y + t * (pb.y - pa.y), 0.15),
                TexCoord::new(uva.u + t * (uvb.u - uva.u), uva.v + t * (uvb.v - uva.v)),
            )
        };

        if in_c == 1 {
            let (p1, uv1) = intersect(in_p[0], out_p[0], in_uv[0], out_uv[0]);
            let (p2, uv2) = intersect(in_p[0], out_p[1], in_uv[0], out_uv[1]);
            (
                [
                    Triangle3D {
                        p: [in_p[0], p1, p2],
                        uv: [in_uv[0], uv1, uv2],
                        normal: tri.normal,
                        tex_idx: tri.tex_idx,
                    },
                    dummy,
                ],
                1,
            )
        } else {
            let (p1, uv1) = intersect(in_p[0], out_p[0], in_uv[0], out_uv[0]);
            let (p2, uv2) = intersect(in_p[1], out_p[0], in_uv[1], out_uv[0]);
            (
                [
                    Triangle3D {
                        p: [in_p[0], in_p[1], p1],
                        uv: [in_uv[0], in_uv[1], uv1],
                        normal: tri.normal,
                        tex_idx: tri.tex_idx,
                    },
                    Triangle3D {
                        p: [in_p[1], p2, p1],
                        uv: [in_uv[1], uv2, uv1],
                        normal: tri.normal,
                        tex_idx: tri.tex_idx,
                    },
                ],
                2,
            )
        }
    }

    fn draw_line_2d(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, col: egui::Color32) {
        let (w, h) = (self.fb_width as i32, self.fb_height as i32);
        let mut x = x0.clamp(-w, w * 2);
        let mut y = y0.clamp(-h, h * 2);
        let tx = x1.clamp(-w, w * 2);
        let ty = y1.clamp(-h, h * 2);

        let dx = (tx - x).abs();
        let dy = -(ty - y).abs();
        let sx = if x < tx { 1 } else { -1 };
        let sy = if y < ty { 1 } else { -1 };
        let mut err = dx + dy;
        let max_steps = (w + h) * 2;

        for _ in 0..max_steps {
            if x >= 0 && x < w && y >= 0 && y < h {
                self.color_buf[y as usize * self.fb_width + x as usize] = col;
            }
            if x == tx && y == ty {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn render(
        &mut self,
        map: &Matrix<SceneNode>,
        cam: &Camera6DOF,
        tex_mgr: &TextureManager,
        now: f64,
        is_playing: bool,
    ) {
        if self.static_dirty {
            self.rebuild_static_mesh(map);
        }

        self.color_buf.fill(egui::Color32::from_rgb(5, 5, 10));
        self.z_buf.fill(f32::INFINITY);

        self.frame_tris.clear();
        self.frame_tris.extend_from_slice(&self.cached_static_tris);

        let anim_frame = ((now * 10.0) as usize) % 4;
        let max_tex = len(&tex_mgr.list).saturating_sub(1);

        for z in range(MAP_H as i64) {
            for x in range(MAP_W as i64) {
                map[(z as usize, x as usize)]
                    .as_entity()
                    .emit_dynamic_geometry(&mut self.frame_tris, anim_frame, max_tex);
            }
        }

        let hover_y = if is_playing {
            ((now * 2.5).sin() as f32) * 0.14
        } else {
            0.0
        };
        let render_cam_pos = Vec3::new(cam.pos.x, cam.pos.y + hover_y, cam.pos.z);

        let recoil_angle = cam.recoil_offset_rad();
        let r_fwd = cam.forward.rotate_axis(cam.right, recoil_angle).normalize();
        let r_up = cam.up.rotate_axis(cam.right, recoil_angle).normalize();
        let r_right = cam.right;

        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let view_up = Vec3::new(
            world_up.dot(r_right),
            world_up.dot(r_up),
            world_up.dot(r_fwd),
        );

        let aspect = self.fb_height as f32 / self.fb_width as f32;
        let fov_rad = 1.0 / (45.0_f32.to_radians()).tan();
        let (fb_w, fb_h) = (self.fb_width as f32, self.fb_height as f32);

        for idx_t in 0..len(&self.frame_tris) {
            let tri = self.frame_tris[idx_t];
            let mut view_tri = tri;
            for i in 0..3 {
                let d = tri.p[i].sub(render_cam_pos);
                view_tri.p[i] = Vec3::new(d.dot(r_right), d.dot(r_up), d.dot(r_fwd));
            }

            // Rejeição rápida de distância e plano traseiro antes do clipping
            if (view_tri.p[0].z < 0.15 && view_tri.p[1].z < 0.15 && view_tri.p[2].z < 0.15)
                || (view_tri.p[0].z > 40.0 && view_tri.p[1].z > 40.0 && view_tri.p[2].z > 40.0)
            {
                continue;
            }

            let view_normal = Vec3::new(
                tri.normal.dot(r_right),
                tri.normal.dot(r_up),
                tri.normal.dot(r_fwd),
            );

            let (clipped_arr, clip_count) = Self::clip_triangle_z(&view_tri);
            for c_idx in 0..clip_count {
                let c_tri = &clipped_arr[c_idx];
                let n_cull = c_tri.p[1].sub(c_tri.p[0]).cross(c_tri.p[2].sub(c_tri.p[0]));
                if n_cull.dot(c_tri.p[0]) >= 0.0 {
                    continue;
                }

                let mut sx = [0.0f32; 3];
                let mut sy = [0.0f32; 3];
                let mut inv_z = [0.0f32; 3];
                let mut u_oz = [0.0f32; 3];
                let mut v_oz = [0.0f32; 3];
                let mut v_col = [[0.0f32; 3]; 3];

                for i in 0..3 {
                    let vp = c_tri.p[i];
                    let iz = 1.0 / vp.z;
                    inv_z[i] = iz;
                    u_oz[i] = c_tri.uv[i].u * iz;
                    v_oz[i] = c_tri.uv[i].v * iz;

                    let px = (vp.x * aspect * fov_rad) * iz;
                    let py = (vp.y * fov_rad) * iz;
                    sx[i] = (px + 1.0) * 0.5 * fb_w;
                    sy[i] = (1.0 - (py + 1.0) * 0.5) * fb_h;

                    let dist = vp.length();
                    let dir_to_cam = if dist > 1e-4 {
                        vp.mul(-1.0 / dist)
                    } else {
                        Vec3::new(0.0, 0.0, -1.0)
                    };
                    let n_dot_l = view_normal.dot(dir_to_cam).max(0.0);
                    let atten = 1.0 / (1.0 + 0.15 * dist + 0.02 * dist * dist);
                    let point_term = n_dot_l * atten * 2.4;

                    let n_dot_up = view_normal.dot(view_up).max(0.0);
                    let ambient_term = 0.22 + n_dot_up * 0.30;

                    let brightness = (ambient_term + point_term).min(1.6);
                    let warm = point_term / (ambient_term + point_term + 1e-4);
                    let fog = (1.0 - vp.z / 40.0).clamp(0.0, 1.0);

                    let r_tint = 0.75 + 0.25 * warm;
                    let g_tint = 0.85;
                    let b_tint = 1.05 - 0.40 * warm;

                    v_col[i] = [
                        (brightness * r_tint * fog).clamp(0.0, 1.5),
                        (brightness * g_tint * fog).clamp(0.0, 1.5),
                        (brightness * b_tint * fog).clamp(0.0, 1.5),
                    ];
                }

                if !sx[0].is_finite()
                    || !sx[1].is_finite()
                    || !sx[2].is_finite()
                    || !sy[0].is_finite()
                    || !sy[1].is_finite()
                    || !sy[2].is_finite()
                {
                    continue;
                }

                let min_x_f = sx[0].min(sx[1]).min(sx[2]).floor();
                let max_x_f = sx[0].max(sx[1]).max(sx[2]).ceil();
                let min_y_f = sy[0].min(sy[1]).min(sy[2]).floor();
                let max_y_f = sy[0].max(sy[1]).max(sy[2]).ceil();

                // Frustum culling 2D: descarta triângulos inteiramente fora da tela
                if max_x_f < 0.0 || min_x_f >= fb_w || max_y_f < 0.0 || min_y_f >= fb_h {
                    continue;
                }

                let min_x = min_x_f.max(0.0) as i32;
                let max_x = max_x_f.min(fb_w - 1.0) as i32;
                let min_y = min_y_f.max(0.0) as i32;
                let max_y = max_y_f.min(fb_h - 1.0) as i32;

                let denom = (sy[1] - sy[2]) * (sx[0] - sx[2]) + (sx[2] - sx[1]) * (sy[0] - sy[2]);
                if denom.abs() < 1e-5 {
                    continue;
                }
                let inv_denom = 1.0 / denom;
                let sw_tex = &tex_mgr.list[c_tri.tex_idx.min(max_tex)];

                let dw0_dx = (sy[1] - sy[2]) * inv_denom;
                let dw1_dx = (sy[2] - sy[0]) * inv_denom;
                let start_fx = min_x as f32 + 0.5;

                for py in min_y..=max_y {
                    let fy = py as f32 + 0.5;
                    let row_offset = py as usize * self.fb_width;

                    let mut w0 = ((sy[1] - sy[2]) * (start_fx - sx[2])
                        + (sx[2] - sx[1]) * (fy - sy[2]))
                        * inv_denom;
                    let mut w1 = ((sy[2] - sy[0]) * (start_fx - sx[2])
                        + (sx[0] - sx[2]) * (fy - sy[2]))
                        * inv_denom;

                    for px in min_x..=max_x {
                        let w2 = 1.0 - w0 - w1;
                        if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                            let iz = w0 * inv_z[0] + w1 * inv_z[1] + w2 * inv_z[2];
                            let z = 1.0 / iz;
                            let idx = row_offset + px as usize;

                            if z < self.z_buf[idx] {
                                let u = (w0 * u_oz[0] + w1 * u_oz[1] + w2 * u_oz[2]) * z;
                                let v = (w0 * v_oz[0] + w1 * v_oz[1] + w2 * v_oz[2]) * z;

                                let texel = sw_tex.sample(u, v);
                                if texel.a() >= 128 {
                                    self.z_buf[idx] = z;
                                    let lr = w0 * v_col[0][0] + w1 * v_col[1][0] + w2 * v_col[2][0];
                                    let lg = w0 * v_col[0][1] + w1 * v_col[1][1] + w2 * v_col[2][1];
                                    let lb = w0 * v_col[0][2] + w1 * v_col[1][2] + w2 * v_col[2][2];

                                    let r = ((texel.r() as f32 * lr).min(255.0)) as u8;
                                    let g = ((texel.g() as f32 * lg).min(255.0)) as u8;
                                    let b = ((texel.b() as f32 * lb).min(255.0)) as u8;
                                    self.color_buf[idx] = egui::Color32::from_rgb(r, g, b);
                                }
                            }
                        }
                        w0 += dw0_dx;
                        w1 += dw1_dx;
                    }
                }

                if self.show_wireframe {
                    let wf_col = egui::Color32::from_rgb(0, 210, 255);
                    self.draw_line_2d(
                        sx[0] as i32,
                        sy[0] as i32,
                        sx[1] as i32,
                        sy[1] as i32,
                        wf_col,
                    );
                    self.draw_line_2d(
                        sx[1] as i32,
                        sy[1] as i32,
                        sx[2] as i32,
                        sy[2] as i32,
                        wf_col,
                    );
                    self.draw_line_2d(
                        sx[2] as i32,
                        sy[2] as i32,
                        sx[0] as i32,
                        sy[0] as i32,
                        wf_col,
                    );
                }
            }
        }
    }
}
