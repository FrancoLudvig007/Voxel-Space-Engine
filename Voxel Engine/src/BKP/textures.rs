#![allow(dead_code)]
use crate::pyrsutils::prelude::*;
use eframe::egui;

pub struct SwTexture {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub is_pow2_64: bool,
    pub pixels: Vec<egui::Color32>,
    pub handle: egui::TextureHandle,
}

impl SwTexture {
    #[inline(always)]
    pub fn sample(&self, u: f32, v: f32) -> egui::Color32 {
        if self.is_pow2_64 {
            // Caminho rápido O(1) com máscara de bits (estilo MS-DOS 64x64)
            let tx = ((u * 64.0) as i32) & 63;
            let ty = ((v * 64.0) as i32) & 63;
            self.pixels[(ty as usize) * 64 + (tx as usize)]
        } else {
            let uw = u - u.floor();
            let vw = v - v.floor();
            let tx = ((uw * self.width as f32) as usize).min(self.width.saturating_sub(1));
            let ty = ((vw * self.height as f32) as usize).min(self.height.saturating_sub(1));
            self.pixels[ty * self.width + tx]
        }
    }
}

pub struct TextureManager {
    pub list: Vec<SwTexture>,
}

impl TextureManager {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut mgr = Self { list: Vec::new() };
        mgr.init_default_assets(ctx);
        mgr
    }

    pub fn register(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        w: usize,
        h: usize,
        pixels: Vec<egui::Color32>,
    ) {
        let color_image = egui::ColorImage {
            size: [w, h],
            pixels: pixels.clone(),
        };
        let tex_opts = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            ..Default::default()
        };
        let handle = ctx.load_texture(name, color_image, tex_opts);
        self.list.append_item(SwTexture {
            name: name.to_string(),
            width: w,
            height: h,
            is_pow2_64: w == 64 && h == 64,
            pixels,
            handle,
        });
    }

    pub fn load_from_disk(&mut self, ctx: &egui::Context, path: &str) -> Result<String, String> {
        if !exists(path) {
            return Err(format!("Arquivo não encontrado: '{path}'"));
        }
        let img = image::open(path).map_err(|e| format!("Falha ao abrir imagem: {e}"))?;
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        let mut pixels = Vec::with_capacity(w * h);
        for p in rgba.pixels() {
            pixels.append_item(egui::Color32::from_rgba_unmultiplied(
                p[0], p[1], p[2], p[3],
            ));
        }
        let nome = basename(path);
        self.register(ctx, &nome, w, h, pixels);
        Ok(format!("Textura '{nome}' ({w}x{h}) carregada."))
    }

    pub fn scan_current_dir(&mut self, ctx: &egui::Context) -> usize {
        let mut count_loaded = 0;
        if let Ok(arquivos) = listdir(".") {
            for arq in arquivos {
                let b = lower(&arq);
                if (endswith(&b, ".png") || endswith(&b, ".jpg"))
                    && !self.list.iter().any(|t| t.name == arq)
                    && self.load_from_disk(ctx, &arq).is_ok()
                {
                    count_loaded += 1;
                }
            }
        }
        count_loaded
    }

    fn init_default_assets(&mut self, ctx: &egui::Context) {
        seed(1996);
        let padroes = [
            "FloorTile.png",         // 0
            "MetalWall.png",         // 1
            "OxidatedMetalWall.png", // 2
            "WallTile.png",          // 3
            "AutoGateLeftPort.png",  // 4
            "AutoGateRightPort.png", // 5
            "FanFrame1.png",         // 6
            "FanFrame2.png",         // 7
            "FanFrame3.png",         // 8
            "FanFrame4.png",         // 9
            "DestroyedFan.png",      // 10
        ];

        for (idx, nome) in enumerate(padroes) {
            if self.load_from_disk(ctx, nome).is_err() {
                let pixels = Self::generate_dos_texture(idx, 64, 64);
                self.register(ctx, nome, 64, 64, pixels);
            }
        }
        let _ = self.scan_current_dir(ctx);
    }

    fn generate_dos_texture(kind: usize, w: usize, h: usize) -> Vec<egui::Color32> {
        let mut px = vec![egui::Color32::BLACK; w * h];
        for y in 0..h {
            for x in 0..w {
                // Convertido explicitamente para i32 para evitar conflito com i64
                let noise = randint(-12, 12) as i32;
                let border = x < 3 || x >= w - 3 || y < 3 || y >= h - 3;

                let c = match kind {
                    0 => {
                        let grid = (x % 16 == 0) || (y % 16 == 0);
                        let base = (if grid { 45 } else { 70 }) + noise;
                        egui::Color32::from_rgb(
                            base.clamp(20, 255) as u8,
                            (base + 5).clamp(20, 255) as u8,
                            (base + 12).clamp(20, 255) as u8,
                        )
                    }
                    1 => {
                        let panel = (if border || y == h / 2 { 50 } else { 95 }) + noise;
                        let rivet = (x == 6 || x == w - 7) && (y % 12 == 6);
                        let v = (if rivet { 160 } else { panel.clamp(30, 220) }) as u8;
                        egui::Color32::from_rgb(
                            v,
                            (v as i32 + 8).clamp(0, 255) as u8,
                            (v as i32 + 20).clamp(0, 255) as u8,
                        )
                    }
                    2 => {
                        let rust = ((x * 7 + y * 13) % 29) as i32;
                        let r = (110 + rust + noise).clamp(40, 220) as u8;
                        let g = (65 + rust / 2 + noise).clamp(25, 160) as u8;
                        let b = (45 + noise).clamp(20, 100) as u8;
                        egui::Color32::from_rgb(r, g, b)
                    }
                    3 => {
                        let stripe = y > h - 12 && ((x + y) / 6) % 2 == 0;
                        if stripe {
                            egui::Color32::from_rgb(210, 175, 20)
                        } else if (y > 14 && y < 24) || (y > 34 && y < 44) {
                            let glow = (130 - ((y as i32 % 10) - 5).abs() * 14 + noise)
                                .clamp(40, 200) as u8;
                            egui::Color32::from_rgb(glow, glow, glow.saturating_add(15))
                        } else {
                            let v = (65 + noise).clamp(30, 120) as u8;
                            egui::Color32::from_rgb(v, v + 5, v + 10)
                        }
                    }
                    4 | 5 => {
                        let warn = (x < 8 || x > w - 9) && ((y / 6) % 2 == 0);
                        if warn {
                            egui::Color32::from_rgb(200, 140, 20)
                        } else if border {
                            egui::Color32::from_rgb(40, 45, 55)
                        } else {
                            let v = (88 + noise).clamp(40, 150) as u8;
                            egui::Color32::from_rgb(v + 10, v, v)
                        }
                    }
                    6..=9 => {
                        let cx = x as f32 - (w as f32 * 0.5);
                        let cy = y as f32 - (h as f32 * 0.5);
                        let r = (cx * cx + cy * cy).sqrt();
                        if border || r > 27.0 {
                            egui::Color32::from_rgb(80, 85, 95)
                        } else if r < 7.0 {
                            egui::Color32::from_rgb(150, 50, 40)
                        } else {
                            let angle = cy.atan2(cx) + (kind - 6) as f32 * 0.4;
                            if (angle * 4.0).sin() > 0.15 {
                                let v = (115 + noise).clamp(60, 190) as u8;
                                egui::Color32::from_rgb(v, v, v + 10)
                            } else {
                                egui::Color32::TRANSPARENT
                            }
                        }
                    }
                    _ => {
                        let cx = x as f32 - (w as f32 * 0.5);
                        let cy = y as f32 - (h as f32 * 0.5);
                        let r = (cx * cx + cy * cy).sqrt();
                        if border || r > 26.0 {
                            egui::Color32::from_rgb(55, 50, 50)
                        } else if r > 19.0 && ((x + y) % 5 == 0) {
                            egui::Color32::from_rgb(90, 60, 40)
                        } else {
                            egui::Color32::TRANSPARENT
                        }
                    }
                };
                px[y * w + x] = c;
            }
        }
        px
    }
}
