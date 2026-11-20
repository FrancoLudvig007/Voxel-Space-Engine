//! Aplicação principal e interface egui/eframe do motor Voxel Space.

use crate::brush::{Brush, BrushMode, PaintSource};
use crate::file_browser::{AssetKind, FileBrowser};
use crate::math::{mix_rgb, worker_threads};
use crate::mesh_renderer::{
    builtin_bridge, builtin_house, builtin_tower, load_obj, Mesh, MeshRenderer, ModelInstance,
    Texture,
};
use crate::pyrsutils::prelude::*;
use crate::sky::Sky;
use crate::terrain::{FlatParams, NoiseParams, Terrain, HEIGHTMAP_RANGE, MAX_HEIGHT};
use crate::voxel_renderer::{BrushOverlay, Camera, Projection, RenderSettings, VoxelRenderer};
use eframe::egui::{self, Color32};
use std::f32::consts::TAU;

const MINIMAP_SIZE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportTool {
    Navigate,
    Sculpt,
    PlaceModel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftTab {
    Engine,
    Terrain,
    Brush,
    Models,
}

pub struct VoxelSpaceApp {
    terrain: Terrain,
    sky: Sky,
    cam: Camera,
    settings: RenderSettings,
    voxel: VoxelRenderer,
    mesh_renderer: MeshRenderer,
    meshes: Vec<Mesh>,
    instances: Vec<ModelInstance>,
    selected_mesh: usize,
    selected_instance: Option<usize>,
    brush: Brush,
    browser: FileBrowser,

    noise_params: NoiseParams,
    flat_params: FlatParams,
    sky_seed: u64,
    export_prefix: String,

    move_speed: f32,
    mouse_sens: f32,
    ground_collision: bool,
    follow_terrain: bool,
    ground_clearance: f32,

    tool: ViewportTool,
    left_tab: LeftTab,
    show_minimap: bool,
    minimap_heightmap: bool,
    linear_filter: bool,
    viewport_tex: Option<egui::TextureHandle>,
    minimap_tex: Option<egui::TextureHandle>,
    minimap_img: egui::ColorImage,
    minimap_rev: u64,
    minimap_state_tag: u64,
    hover_world: Option<(f32, f32)>,
    hover_from_minimap: bool,

    status_msg: String,
    status_is_err: bool,
    start_time: f64,
    last_frame_time: f64,
    fps_smooth: f32,
    frame_ms: f32,
    voxel_ms: f32,
    mesh_ms: f32,
    present_ms: f32,
}

impl VoxelSpaceApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let settings = RenderSettings::default();
        let noise_params = NoiseParams::default();
        let water_raw = settings.water_level / settings.vertical_scale.max(0.01);
        let terrain = Terrain::new_perlin(&noise_params, water_raw);
        let sky = Sky::procedural(77);
        let voxel = VoxelRenderer::new(800, 600);
        let meshes = vec![builtin_house(), builtin_bridge(), builtin_tower()];

        let now = perf_counter();
        let mut app = Self {
            terrain,
            sky,
            cam: Camera {
                x: 512.0,
                y: 512.0,
                height: 150.0,
                yaw: 0.4,
                pitch: -0.08,
                fov_deg: 75.0,
            },
            settings,
            voxel,
            mesh_renderer: MeshRenderer::default(),
            meshes,
            instances: Vec::new(),
            selected_mesh: 0,
            selected_instance: None,
            brush: Brush::default(),
            browser: FileBrowser::new(),
            noise_params,
            flat_params: FlatParams::default(),
            sky_seed: 77,
            export_prefix: "terreno".to_string(),
            move_speed: 140.0,
            mouse_sens: 0.004,
            ground_collision: true,
            follow_terrain: false,
            ground_clearance: 14.0,
            tool: ViewportTool::Navigate,
            left_tab: LeftTab::Engine,
            show_minimap: true,
            minimap_heightmap: false,
            linear_filter: true,
            viewport_tex: None,
            minimap_tex: None,
            minimap_img: egui::ColorImage::new([MINIMAP_SIZE, MINIMAP_SIZE], Color32::BLACK),
            minimap_rev: u64::MAX,
            minimap_state_tag: u64::MAX,
            hover_world: None,
            hover_from_minimap: false,
            status_msg: "Motor Voxel Space pronto. Arraste com o mouse para olhar e WASD para voar.".into(),
            status_is_err: false,
            start_time: now,
            last_frame_time: now,
            fps_smooth: 60.0,
            frame_ms: 16.6,
            voxel_ms: 0.0,
            mesh_ms: 0.0,
            present_ms: 0.0,
        };
        app.reset_camera();
        app.spawn_demo_instances();
        app
    }

    fn reset_camera(&mut self) {
        let cx = self.terrain.width as f32 * 0.5;
        let cy = self.terrain.height as f32 * 0.5;
        let ground = self.terrain.height_at(cx, cy) * self.settings.vertical_scale;
        let base = if self.settings.water_enabled {
            ground.max(self.settings.water_level)
        } else {
            ground
        };
        self.cam.x = cx;
        self.cam.y = cy;
        self.cam.height = base + 65.0;
        self.cam.yaw = 0.4;
        self.cam.pitch = -0.08;
    }

    fn spawn_demo_instances(&mut self) {
        self.instances.clear();
        let (fx, fy) = self.cam.forward();
        let (rx, ry) = self.cam.right();
        let specs = [
            (0usize, 145.0f32, 0.0f32, 22.0f32, 1.4f32),
            (1usize, 175.0, -52.0, -35.0, 1.3),
            (2usize, 215.0, 58.0, 15.0, 1.35),
        ];
        for (mesh_idx, dist, lat, yaw_deg, scale) in specs {
            if mesh_idx < len(&self.meshes) {
                let wx = (self.cam.x + fx * dist + rx * lat).rem_euclid(self.terrain.width as f32);
                let wy = (self.cam.y + fy * dist + ry * lat).rem_euclid(self.terrain.height as f32);
                let mut inst = ModelInstance::new(mesh_idx, wx, wy);
                inst.yaw_deg = yaw_deg;
                inst.scale = scale;
                self.instances.append_item(inst);
            }
        }
        if !self.instances.is_empty() {
            self.selected_instance = Some(0);
        }
    }

    fn set_status(&mut self, msg: impl Into<String>, is_err: bool) {
        self.status_msg = msg.into();
        self.status_is_err = is_err;
    }

    fn water_raw(&self) -> f32 {
        self.settings.water_level / self.settings.vertical_scale.max(0.01)
    }

    fn spawn_instance_at(&mut self, mesh_idx: usize, x: f32, y: f32) {
        if mesh_idx >= len(&self.meshes) {
            self.set_status("Nenhum modelo .OBJ selecionado.", true);
            return;
        }
        let wx = x.rem_euclid(self.terrain.width as f32);
        let wy = y.rem_euclid(self.terrain.height as f32);
        let inst = ModelInstance::new(mesh_idx, wx, wy);
        self.instances.append_item(inst);
        let idx = len(&self.instances) - 1;
        self.selected_instance = Some(idx);
        let nome = &self.meshes[mesh_idx].name;
        self.set_status(format!("Instância #{idx} ('{nome}') posicionada em ({wx:.1}, {wy:.1})."), false);
    }

    fn place_model_in_front(&mut self) {
        let (fx, fy) = self.cam.forward();
        self.spawn_instance_at(self.selected_mesh, self.cam.x + fx * 120.0, self.cam.y + fy * 120.0);
    }

    fn handle_asset(&mut self, kind: AssetKind, path: &str) {
        match kind {
            AssetKind::Heightmap => {
                let wr = self.water_raw();
                match self.terrain.load_heightmap(path, wr) {
                    Ok(m) => {
                        self.reset_camera();
                        self.set_status(m, false);
                    }
                    Err(e) => self.set_status(e, true),
                }
            }
            AssetKind::Colormap => match self.terrain.load_colormap(path) {
                Ok(m) => self.set_status(m, false),
                Err(e) => self.set_status(e, true),
            },
            AssetKind::Sky => match Sky::load(path, self.sky.horizon_frac) {
                Ok(sky) => {
                    let msg = format!("Céu '{}' ({}x{}) carregado.", sky.name, sky.width, sky.height);
                    self.sky = sky;
                    self.set_status(msg, false);
                }
                Err(e) => self.set_status(e, true),
            },
            AssetKind::WaterTexture => match self.voxel.load_water_texture(path) {
                Ok(m) => self.set_status(m, false),
                Err(e) => self.set_status(e, true),
            },
            AssetKind::BrushTexture => match self.brush.load_paint_texture(path) {
                Ok(m) => self.set_status(m, false),
                Err(e) => self.set_status(e, true),
            },
            AssetKind::ObjModel => match load_obj(path) {
                Ok((mesh, info)) => {
                    self.meshes.append_item(mesh);
                    let idx = len(&self.meshes) - 1;
                    self.selected_mesh = idx;
                    self.place_model_in_front();
                    self.set_status(format!("{info} Posicionado à frente da câmera."), false);
                }
                Err(e) => self.set_status(e, true),
            },
            AssetKind::MeshTexture(mesh_idx) => match Texture::load(path) {
                Ok(tex) => {
                    if let Some(m) = self.meshes.get_mut(mesh_idx) {
                        let tname = tex.name.clone();
                        let mname = m.name.clone();
                        m.texture = Some(tex);
                        self.set_status(format!("Textura '{tname}' aplicada ao modelo '{mname}'."), false);
                    }
                }
                Err(e) => self.set_status(e, true),
            },
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped {
            let Some(pb) = f.path else { continue };
            let path = pb.to_string_lossy().into_owned();
            let l = lower(&path);
            if endswith(&l, ".obj") {
                self.handle_asset(AssetKind::ObjModel, &path);
            } else if endswith(&l, ".png")
                || endswith(&l, ".jpg")
                || endswith(&l, ".jpeg")
                || endswith(&l, ".bmp")
                || endswith(&l, ".tga")
            {
                let name = lower(&basename(&path));
                let kind = if find(&name, "sky") >= 0 || find(&name, "ceu") >= 0 {
                    AssetKind::Sky
                } else if find(&name, "height") >= 0 || find(&name, "elev") >= 0 || find(&name, "dem") >= 0 {
                    AssetKind::Heightmap
                } else if find(&name, "water") >= 0 || find(&name, "agua") >= 0 {
                    AssetKind::WaterTexture
                } else {
                    AssetKind::Colormap
                };
                self.handle_asset(kind, &path);
            }
        }
    }

    fn update_camera(&mut self, ctx: &egui::Context, dt: f32) {
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Num1) {
                    self.tool = ViewportTool::Navigate;
                } else if i.key_pressed(egui::Key::Num2) {
                    self.tool = ViewportTool::Sculpt;
                } else if i.key_pressed(egui::Key::Num3) {
                    self.tool = ViewportTool::PlaceModel;
                }
                if i.key_pressed(egui::Key::M) {
                    self.show_minimap = !self.show_minimap;
                }

                let boost = if i.modifiers.shift { 2.5 } else { 1.0 };
                let spd = self.move_speed * boost * dt;
                let (fx, fy) = self.cam.forward();
                let (rx, ry) = self.cam.right();

                if i.key_down(egui::Key::W) || i.key_down(egui::Key::ArrowUp) {
                    self.cam.x += fx * spd;
                    self.cam.y += fy * spd;
                }
                if i.key_down(egui::Key::S) || i.key_down(egui::Key::ArrowDown) {
                    self.cam.x -= fx * spd;
                    self.cam.y -= fy * spd;
                }
                if i.key_down(egui::Key::D) || i.key_down(egui::Key::ArrowRight) {
                    self.cam.x += rx * spd;
                    self.cam.y += ry * spd;
                }
                if i.key_down(egui::Key::A) || i.key_down(egui::Key::ArrowLeft) {
                    self.cam.x -= rx * spd;
                    self.cam.y -= ry * spd;
                }
                if i.key_down(egui::Key::E) || i.key_down(egui::Key::Space) {
                    self.cam.height += spd * 0.7;
                }
                if i.key_down(egui::Key::Q) || i.key_down(egui::Key::C) {
                    self.cam.height -= spd * 0.7;
                }
            });
        }

        self.cam.x = self.cam.x.rem_euclid(self.terrain.width as f32);
        self.cam.y = self.cam.y.rem_euclid(self.terrain.height as f32);

        let ground = self.terrain.height_at(self.cam.x, self.cam.y) * self.settings.vertical_scale;
        if self.follow_terrain {
            self.cam.height = ground + self.ground_clearance;
        } else if self.ground_collision {
            let min_h = ground + self.ground_clearance;
            if self.cam.height < min_h {
                self.cam.height = min_h;
            }
        }
        self.cam.height = self.cam.height.clamp(-200.0, 2000.0);
        self.cam.pitch = self.cam.pitch.clamp(-0.9, 0.9);
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Voxel Space Engine");
                ui.separator();

                ui.selectable_value(&mut self.tool, ViewportTool::Navigate, "✈ Navegar [1]");
                ui.selectable_value(&mut self.tool, ViewportTool::Sculpt, "🖌 Esculpir [2]");
                ui.selectable_value(&mut self.tool, ViewportTool::PlaceModel, "📦 Posicionar [3]");
                ui.separator();

                if ui.button("📂 Heightmap").clicked() {
                    self.browser.open_for(AssetKind::Heightmap);
                }
                if ui.button("🎨 Colormap").clicked() {
                    self.browser.open_for(AssetKind::Colormap);
                }
                if ui.button("🌌 Céu").clicked() {
                    self.browser.open_for(AssetKind::Sky);
                }
                if ui.button("📦 .OBJ").clicked() {
                    self.browser.open_for(AssetKind::ObjModel);
                }
                if ui.button("⬜ Plano").clicked() {
                    self.terrain.apply_flat(&self.flat_params);
                    self.reset_camera();
                    self.set_status("Terreno plano inicializado.", false);
                }
                ui.toggle_value(&mut self.show_minimap, "🗺 Mapa 2D [M]");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fps_color = if self.fps_smooth >= 45.0 {
                        Color32::LIGHT_GREEN
                    } else if self.fps_smooth >= 24.0 {
                        Color32::YELLOW
                    } else {
                        Color32::LIGHT_RED
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "Tris: {} | {}x{} | V:{:.1}ms M:{:.1}ms | {:.1} ms | {:.0} FPS",
                            self.mesh_renderer.tris_drawn,
                            self.voxel.width,
                            self.voxel.height,
                            self.voxel_ms,
                            self.mesh_ms,
                            self.frame_ms,
                            self.fps_smooth
                        ))
                        .monospace()
                        .color(fps_color),
                    );
                });
            });
        });
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let color = if self.status_is_err {
                    Color32::LIGHT_RED
                } else {
                    Color32::LIGHT_GRAY
                };
                ui.colored_label(color, &self.status_msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ground = self.terrain.height_at(self.cam.x, self.cam.y) * self.settings.vertical_scale;
                    let hover = match self.hover_world {
                        Some((hx, hy)) => {
                            let hz = self.terrain.height_at(hx, hy) * self.settings.vertical_scale;
                            format!("Cursor ({hx:.0},{hy:.0}) H={hz:.1} | ")
                        }
                        None => String::new(),
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "{hover}Cam ({:.1},{:.1}) Alt={:.1} Solo={ground:.1} Yaw={:.0}° Pitch={:.2} | WASD/EQ/Shift + RMB",
                            self.cam.x,
                            self.cam.y,
                            self.cam.height,
                            self.cam.yaw.to_degrees().rem_euclid(360.0),
                            self.cam.pitch
                        ))
                        .monospace()
                        .small(),
                    );
                });
            });
        });
    }

    fn draw_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .default_width(310.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.left_tab, LeftTab::Engine, "🎥 Motor");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Terrain, "🏔 Terreno");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Brush, "🖌 Pincel");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Models, "📦 Modelos");
                });
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.left_tab {
                    LeftTab::Engine => self.draw_tab_engine(ui),
                    LeftTab::Terrain => self.draw_tab_terrain(ui),
                    LeftTab::Brush => self.draw_tab_brush(ui),
                    LeftTab::Models => self.draw_tab_models(ui),
                });
            });
    }

    fn draw_tab_engine(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📷 Câmera e Colisão", |ui| {
            ui.add(egui::Slider::new(&mut self.cam.fov_deg, 30.0..=130.0).text("FOV (°)"));
            ui.add(egui::Slider::new(&mut self.cam.height, -50.0..=800.0).text("Altitude"));
            ui.add(egui::Slider::new(&mut self.cam.pitch, -0.85..=0.85).text("Pitch"));
            ui.add(egui::Slider::new(&mut self.move_speed, 20.0..=500.0).text("Velocidade"));
            ui.checkbox(&mut self.ground_collision, "Colisão com o solo");
            ui.checkbox(&mut self.follow_terrain, "Acompanhar relevo (modo chão)");
            ui.add(egui::Slider::new(&mut self.ground_clearance, 2.0..=120.0).text("Altura mínima"));
            if ui.button("🎯 Centralizar câmera").clicked() {
                self.reset_camera();
            }
        });

        ui.collapsing("🚀 Núcleo Voxel Space (LOD / Render)", |ui| {
            ui.add(egui::Slider::new(&mut self.settings.draw_distance, 200.0..=4500.0).text("Draw distance"));
            ui.add(egui::Slider::new(&mut self.settings.vertical_scale, 0.2..=3.5).text("Escala vertical"));
            ui.add(
                egui::Slider::new(&mut self.settings.lod_factor, 0.001..=0.05)
                    .logarithmic(true)
                    .text("LOD (passo/distância)"),
            );
            ui.add(egui::Slider::new(&mut self.settings.near_step, 0.25..=2.0).text("Passo inicial"));
            ui.add(egui::Slider::new(&mut self.settings.render_scale, 0.35..=1.0).text("Escala interna"));
            let max_t = worker_threads();
            ui.add(egui::Slider::new(&mut self.settings.threads, 0..=max_t).text("Threads (0=auto)"));
            ui.checkbox(&mut self.linear_filter, "Filtro bilinear na viewport");
            ui.checkbox(&mut self.settings.fog_enabled, "Névoa atmosférica");
            if self.settings.fog_enabled {
                ui.add(egui::Slider::new(&mut self.settings.fog_start, 0.1..=0.95).text("Início da névoa"));
            }
        });

        ui.collapsing("🌊 Sistema de Água (Water Plane)", |ui| {
            ui.checkbox(&mut self.settings.water_enabled, "Plano de água ativo");
            ui.add(egui::Slider::new(&mut self.settings.water_level, 0.0..=400.0).text("water_level"));
            ui.add(egui::Slider::new(&mut self.settings.water_opaque_depth, 2.0..=100.0).text("Profund. opaca"));
            ui.add(egui::Slider::new(&mut self.settings.water_reflectivity, 0.0..=0.9).text("Reflexo do céu"));
            ui.horizontal(|ui| {
                ui.label("Cor da água:");
                ui.color_edit_button_srgba(&mut self.settings.water_color);
            });
            ui.checkbox(&mut self.settings.water_waves, "Ondulação animada");
            ui.label(format!("Textura atual: {}", self.voxel.water_tex.name));
            ui.horizontal(|ui| {
                if ui.button("📂 Carregar textura água").clicked() {
                    self.browser.open_for(AssetKind::WaterTexture);
                }
                if ui.button("🌊 Procedural").clicked() {
                    self.voxel.reset_water_texture();
                    self.set_status("Textura de água procedural restaurada.", false);
                }
            });
        });

        ui.collapsing("📊 Desempenho", |ui| {
            ui.monospace(format!("FPS médio:   {:.1}", self.fps_smooth));
            ui.monospace(format!("Frame total: {:.2} ms", self.frame_ms));
            ui.monospace(format!("Voxel Space: {:.2} ms", self.voxel_ms));
            ui.monospace(format!("Malhas 3D:   {:.2} ms", self.mesh_ms));
            ui.monospace(format!("Transposição:{:.2} ms", self.present_ms));
            ui.monospace(format!("Buffer:      {} x {}", self.voxel.width, self.voxel.height));
            ui.monospace(format!("Triângulos:  {}", self.mesh_renderer.tris_drawn));
        });
    }

    fn draw_tab_terrain(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📂 Heightmap & Colormap", |ui| {
            ui.label(format!(
                "Mapa atual: {} ({}x{})",
                self.terrain.name, self.terrain.width, self.terrain.height
            ));
            ui.horizontal(|ui| {
                if ui.button("📂 Heightmap").clicked() {
                    self.browser.open_for(AssetKind::Heightmap);
                }
                if ui.button("📂 Colormap").clicked() {
                    self.browser.open_for(AssetKind::Colormap);
                }
            });
            if ui.button("🎨 Recolorir por altitude (biomas)").clicked() {
                let wr = self.water_raw();
                self.terrain.auto_colorize(wr);
                self.terrain.rebuild_all();
                self.set_status("Colormap recalculado por altitude/inclinação.", false);
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Prefixo:");
                ui.text_edit_singleline(&mut self.export_prefix);
            });
            ui.horizontal(|ui| {
                if ui.button("💾 Exportar Height").clicked() {
                    let p = format!("{}_height.png", self.export_prefix);
                    match self.terrain.export_heightmap(&p) {
                        Ok(m) => self.set_status(m, false),
                        Err(e) => self.set_status(e, true),
                    }
                }
                if ui.button("💾 Exportar Color").clicked() {
                    let p = format!("{}_color.png", self.export_prefix);
                    match self.terrain.export_colormap(&p) {
                        Ok(m) => self.set_status(m, false),
                        Err(e) => self.set_status(e, true),
                    }
                }
            });
        });

        ui.collapsing("🌄 Gerador Procedural (Perlin Noise)", |ui| {
            ui.horizontal(|ui| {
                ui.label("Semente:");
                ui.add(egui::DragValue::new(&mut self.noise_params.seed));
                if ui.button("🎲 Aleatória").clicked() {
                    self.noise_params.seed = randint(1, 999_999) as u64;
                }
            });
            let side = 1u32 << self.noise_params.size_exp;
            ui.add(egui::Slider::new(&mut self.noise_params.size_exp, 7..=11).text(format!("Resolução ({side}²)")));
            ui.add(egui::Slider::new(&mut self.noise_params.base_freq, 1..=16).text("Frequência base"));
            ui.add(egui::Slider::new(&mut self.noise_params.octaves, 1..=10).text("Oitavas fBm"));
            ui.add(egui::Slider::new(&mut self.noise_params.persistence, 0.2..=0.75).text("Persistência"));
            ui.add(egui::Slider::new(&mut self.noise_params.amplitude, 50.0..=HEIGHTMAP_RANGE * 2.0).text("Amplitude"));
            ui.add(egui::Slider::new(&mut self.noise_params.exponent, 0.5..=3.0).text("Expoente relevo"));
            ui.checkbox(&mut self.noise_params.ridged, "Cordilheiras (Ridged)");
            ui.checkbox(&mut self.noise_params.auto_color, "Gerar Colormap automático");
            if ui.button("⚡ Gerar terreno Perlin").clicked() {
                let wr = self.water_raw();
                self.terrain.generate_perlin(&self.noise_params, wr);
                self.reset_camera();
                self.set_status(format!("Terreno Perlin gerado ({}).", self.terrain.name), false);
            }
        });

        ui.collapsing("⬜ Modo Terreno Plano", |ui| {
            let side = 1u32 << self.flat_params.size_exp;
            ui.add(egui::Slider::new(&mut self.flat_params.size_exp, 7..=11).text(format!("Resolução ({side}²)")));
            ui.add(egui::Slider::new(&mut self.flat_params.base_height, 0.0..=400.0).text("Altura base"));
            ui.horizontal(|ui| {
                ui.label("Cor base:");
                ui.color_edit_button_srgba(&mut self.flat_params.color);
            });
            if ui.button("⬜ Inicializar terreno plano").clicked() {
                self.terrain.apply_flat(&self.flat_params);
                self.reset_camera();
                self.set_status("Terreno plano criado para escultura livre.", false);
            }
        });

        ui.collapsing("☀ Iluminação Direcional do Terreno", |ui| {
            let mut changed = ui.checkbox(&mut self.terrain.lighting, "Sombreamento por normais").changed();
            if self.terrain.lighting {
                changed |= ui.add(egui::Slider::new(&mut self.terrain.sun_azimuth, 0.0..=360.0).text("Azimute sol (°)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.terrain.sun_elevation, 5.0..=88.0).text("Elevação sol (°)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.terrain.ambient, 0.05..=0.9).text("Luz ambiente")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.terrain.relief, 0.2..=4.0).text("Força do relevo")).changed();
            }
            if changed {
                self.terrain.set_sun(self.terrain.sun_azimuth, self.terrain.sun_elevation);
                self.terrain.rebuild_all();
            }
        });

        ui.collapsing("🌌 Céu Panorâmico (Sky Rendering)", |ui| {
            ui.label(format!("Céu atual: {} ({}x{})", self.sky.name, self.sky.width, self.sky.height));
            let mut hfrac = self.sky.horizon_frac;
            if ui.add(egui::Slider::new(&mut hfrac, 0.1..=1.0).text("Linha do horizonte")).changed() {
                self.sky.set_horizon_frac(hfrac);
            }
            ui.horizontal(|ui| {
                if ui.button("📂 Carregar imagem céu").clicked() {
                    self.browser.open_for(AssetKind::Sky);
                }
                if ui.button("☁ Novo céu procedural").clicked() {
                    self.sky_seed = randint(1, 9999) as u64;
                    self.sky = Sky::procedural(self.sky_seed);
                    self.set_status("Céu procedural regenerado.", false);
                }
            });
        });
    }

    fn draw_tab_brush(&mut self, ui: &mut egui::Ui) {
        if self.tool != ViewportTool::Sculpt {
            ui.colored_label(Color32::YELLOW, "Ative '🖌 Esculpir [2]' para usar o pincel na viewport.");
            if ui.button("Ativar ferramenta Esculpir").clicked() {
                self.tool = ViewportTool::Sculpt;
            }
            ui.separator();
        }
        ui.strong("Modo de Operação");
        for m in BrushMode::ALL {
            ui.radio_value(&mut self.brush.mode, m, m.label());
        }
        ui.separator();
        ui.add(egui::Slider::new(&mut self.brush.radius, 2.0..=150.0).text("Raio (scroll)"));
        ui.add(egui::Slider::new(&mut self.brush.strength, 0.1..=6.0).text("Intensidade"));
        ui.add(egui::Slider::new(&mut self.brush.hardness, 0.0..=0.95).text("Dureza da borda"));

        if self.brush.mode == BrushMode::Flatten {
            ui.separator();
            ui.add(egui::Slider::new(&mut self.brush.flatten_height, 0.0..=MAX_HEIGHT).text("Altura alvo"));
            ui.checkbox(&mut self.brush.flatten_pick_on_click, "Capturar altura no clique");
        }

        ui.separator();
        ui.strong("Pintura no Colormap (Cor / Textura)");
        ui.checkbox(&mut self.brush.paint_while_sculpting, "Pintar também ao esculpir");
        ui.add(egui::Slider::new(&mut self.brush.paint_opacity, 0.05..=1.0).text("Opacidade pintura"));
        ui.horizontal(|ui| {
            ui.label("Cor:");
            ui.color_edit_button_srgba(&mut self.brush.paint_color);
        });

        let mut src = self.brush.paint_source;
        egui::ComboBox::from_label("Fonte de pintura")
            .selected_text(src.label())
            .show_ui(ui, |ui| {
                for s in PaintSource::ALL {
                    ui.selectable_value(&mut src, s, s.label());
                }
            });
        if src != self.brush.paint_source {
            self.brush.set_paint_source(src);
        }
        if self.brush.paint_source != PaintSource::SolidColor {
            ui.add(egui::Slider::new(&mut self.brush.paint_tex_scale, 0.1..=4.0).text("Escala da textura"));
            ui.checkbox(&mut self.brush.tint_texture, "Tingir textura com cor");
            ui.small(format!("Textura ativa: {}", self.brush.paint_tex_name));
        }
        if ui.button("📂 Carregar textura externa do pincel").clicked() {
            self.browser.open_for(AssetKind::BrushTexture);
        }
        ui.separator();
        ui.small("Dica: segure Ctrl para inverter Elevar ↔ Rebaixar. Você também pode esculpir direto no Minimapa 2D.");
    }

    fn draw_tab_models(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📂 Carregar .OBJ").clicked() {
                self.browser.open_for(AssetKind::ObjModel);
            }
            if ui.button("➕ Colocar à frente").clicked() {
                self.place_model_in_front();
            }
        });
        ui.checkbox(&mut self.mesh_renderer.backface_cull, "Backface culling");
        if ui.button("🖱 Posicionar com clique na viewport [3]").clicked() {
            self.tool = ViewportTool::PlaceModel;
        }
        ui.separator();
        ui.strong("Biblioteca de Malhas (.OBJ)");
        let mut load_tex_for: Option<usize> = None;
        let mut clear_tex_for: Option<usize> = None;
        for (i, m) in enumerate(&self.meshes) {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_mesh, i, format!("{} ({}t)", m.name, m.tris.len()));
                if ui.small_button("🖼 Textura").clicked() {
                    load_tex_for = Some(i);
                }
                if m.texture.is_some() && ui.small_button("✕").clicked() {
                    clear_tex_for = Some(i);
                }
            });
        }
        if let Some(i) = load_tex_for {
            self.browser.open_for(AssetKind::MeshTexture(i));
        }
        if let Some(i) = clear_tex_for {
            self.meshes[i].texture = None;
        }
        if let Some(m) = self.meshes.get_mut(self.selected_mesh) {
            ui.horizontal(|ui| {
                ui.label("Cor sem textura:");
                ui.color_edit_button_srgba(&mut m.base_color);
            });
        }

        ui.separator();
        ui.strong(format!("Instâncias na cena ({})", len(&self.instances)));
        let mut to_remove: Option<usize> = None;
        for i in range(len(&self.instances) as i64) {
            let idx = i as usize;
            let inst = &self.instances[idx];
            let mname = self.meshes.get(inst.mesh).map(|m| m.name.as_str()).unwrap_or("?");
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.selected_instance,
                    Some(idx),
                    format!("#{idx} {mname} ({:.0},{:.0})", inst.x, inst.y),
                );
                if ui.small_button("🗑").clicked() {
                    to_remove = Some(idx);
                }
            });
        }
        if let Some(idx) = to_remove {
            self.instances.pop_at(Some(idx));
            self.selected_instance = if self.instances.is_empty() {
                None
            } else {
                Some(idx.min(len(&self.instances) - 1))
            };
        }

        if let Some(idx) = self.selected_instance {
            if idx < len(&self.instances) {
                ui.separator();
                ui.strong(format!("Transformação da instância #{idx}"));
                let w = self.terrain.width as f32;
                let h = self.terrain.height as f32;
                {
                    let inst = &mut self.instances[idx];
                    ui.add(egui::Slider::new(&mut inst.x, 0.0..=w).text("X"));
                    ui.add(egui::Slider::new(&mut inst.y, 0.0..=h).text("Y"));
                    ui.add(egui::Slider::new(&mut inst.y_offset, -100.0..=300.0).text("Alt. offset"));
                    ui.add(egui::Slider::new(&mut inst.yaw_deg, -180.0..=180.0).text("Rotação (°)"));
                    ui.add(egui::Slider::new(&mut inst.scale, 0.1..=10.0).text("Escala"));
                    ui.checkbox(&mut inst.snap_to_ground, "Apoiar no terreno");
                }
                ui.horizontal(|ui| {
                    if ui.button("👁 Focar câmera").clicked() {
                        let inst = &self.instances[idx];
                        let ground = inst.base_height(&self.terrain, self.settings.vertical_scale);
                        self.cam.x = (inst.x - 90.0).rem_euclid(w);
                        self.cam.y = inst.y;
                        self.cam.height = ground + 45.0;
                        self.cam.yaw = 0.0;
                        self.cam.pitch = -0.18;
                    }
                    if ui.button("📋 Duplicar").clicked() {
                        let mut copy = self.instances[idx].clone();
                        copy.x = (copy.x + 25.0).rem_euclid(w);
                        self.instances.append_item(copy);
                        self.selected_instance = Some(len(&self.instances) - 1);
                    }
                });
            }
        }
    }

    fn rebuild_minimap_image(&mut self) {
        let water_tag = if self.settings.water_enabled {
            self.settings.water_level.to_bits() as u64
        } else {
            u64::MAX
        };
        let state_tag = (self.minimap_heightmap as u64)
            ^ (water_tag << 1)
            ^ ((self.settings.vertical_scale.to_bits() as u64) << 33);
        if self.minimap_rev == self.terrain.revision && self.minimap_state_tag == state_tag {
            return;
        }
        self.minimap_rev = self.terrain.revision;
        self.minimap_state_tag = state_tag;

        let (tw, th_map) = (self.terrain.width, self.terrain.height);
        let inv_range = 1.0 / HEIGHTMAP_RANGE;
        for my in 0..MINIMAP_SIZE {
            let ty = my * th_map / MINIMAP_SIZE;
            for mx in 0..MINIMAP_SIZE {
                let tx = mx * tw / MINIMAP_SIZE;
                let cell = self.terrain.cells[ty * tw + tx];
                let col = if self.minimap_heightmap {
                    let g = ((cell.h * inv_range).clamp(0.0, 1.0) * 255.0) as u8;
                    Color32::from_rgb(g, g, g)
                } else {
                    let h = cell.h * self.settings.vertical_scale;
                    if self.settings.water_enabled && h < self.settings.water_level {
                        let d = ((self.settings.water_level - h) / self.settings.water_opaque_depth.max(1.0)).min(1.0);
                        mix_rgb(cell.color, self.settings.water_color, d * (2.0 - d) * 0.85)
                    } else {
                        cell.color
                    }
                };
                self.minimap_img.pixels[my * MINIMAP_SIZE + mx] = col;
            }
        }
    }

    fn draw_minimap_panel(&mut self, ctx: &egui::Context, dt: f32) {
        if !self.show_minimap {
            return;
        }
        self.rebuild_minimap_image();
        let tex = self.minimap_tex.get_or_insert_with(|| {
            ctx.load_texture("minimap", self.minimap_img.clone(), egui::TextureOptions::LINEAR)
        });
        tex.set(self.minimap_img.clone(), egui::TextureOptions::LINEAR);
        let tex_id = tex.id();

        egui::SidePanel::right("minimap_panel")
            .exact_width(276.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("🗺 Visão Superior 2D");
                    ui.checkbox(&mut self.minimap_heightmap, "Heightmap");
                });
                let size = egui::vec2(MINIMAP_SIZE as f32, MINIMAP_SIZE as f32);
                let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                ui.painter().image(
                    tex_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                let (tw, th) = (self.terrain.width as f32, self.terrain.height as f32);
                let to_screen = |wx: f32, wy: f32| {
                    egui::pos2(
                        rect.left() + wx.rem_euclid(tw) / tw * rect.width(),
                        rect.top() + wy.rem_euclid(th) / th * rect.height(),
                    )
                };

                // Desenha instâncias de modelos
                for (i, inst) in enumerate(&self.instances) {
                    let c = if self.selected_instance == Some(i) { Color32::YELLOW } else { Color32::CYAN };
                    ui.painter().circle_filled(to_screen(inst.x, inst.y), 3.5, c);
                }

                // Desenha câmera + cone de visão
                let cpos = to_screen(self.cam.x, self.cam.y);
                let half_fov = self.cam.fov_deg.to_radians() * 0.5;
                let ray_len = 28.0;
                for ang in [self.cam.yaw - half_fov, self.cam.yaw + half_fov] {
                    let tip = egui::pos2(cpos.x + ang.cos() * ray_len, cpos.y + ang.sin() * ray_len);
                    ui.painter().line_segment([cpos, tip], egui::Stroke::new(1.5, Color32::YELLOW));
                }
                ui.painter().circle_filled(cpos, 4.0, Color32::RED);

                // Interação no minimapa (esculpir 2D, posicionar modelo ou teleportar câmera)
                if let Some(pos) = resp.hover_pos() {
                    if rect.contains(pos) {
                        let wx = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0) * tw;
                        let wy = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0) * th;
                        self.hover_world = Some((wx, wy));
                        self.hover_from_minimap = true;

                        let r_px = (self.brush.radius / tw * rect.width()).max(2.0);
                        if self.tool == ViewportTool::Sculpt {
                            ui.painter().circle_stroke(
                                pos,
                                r_px,
                                egui::Stroke::new(1.5, self.brush.mode.overlay_color()),
                            );
                        }

                        let lmb_down = resp.is_pointer_button_down_on()
                            && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
                        let rmb_down = resp.is_pointer_button_down_on()
                            && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));

                        match self.tool {
                            ViewportTool::Sculpt if lmb_down => {
                                if ctx.input(|i| i.pointer.primary_pressed())
                                    && self.brush.mode == BrushMode::Flatten
                                    && self.brush.flatten_pick_on_click
                                {
                                    self.brush.flatten_height = self.terrain.height_at(wx, wy);
                                }
                                let ctrl = ctx.input(|i| i.modifiers.ctrl);
                                self.brush.apply(&mut self.terrain, wx, wy, dt, ctrl);
                            }
                            ViewportTool::PlaceModel if resp.clicked_by(egui::PointerButton::Primary) => {
                                self.spawn_instance_at(self.selected_mesh, wx, wy);
                            }
                            ViewportTool::Navigate if lmb_down => {
                                self.cam.x = wx;
                                self.cam.y = wy;
                            }
                            _ => {}
                        }
                        if rmb_down {
                            self.cam.x = wx;
                            self.cam.y = wy;
                        }
                    }
                }

                ui.separator();
                ui.small("• Esculpir: clique/arraste para editar 2D\n• Botão direito (ou Navegar): teleporta câmera\n• Posicionar: clique para colocar .OBJ");
            });
    }

    fn draw_viewport(&mut self, ctx: &egui::Context, dt: f32) {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
            let avail = ui.available_size();
            let scale = self.settings.render_scale.clamp(0.25, 1.0);
            let rw = ((avail.x.max(1.0) * scale) as usize).clamp(160, 1920);
            let rh = ((avail.y.max(1.0) * scale) as usize).clamp(120, 1080);
            self.voxel.resize(rw, rh);

            let (rect, resp) = ui.allocate_exact_size(avail, egui::Sense::click_and_drag());

            // Rotação da câmera (RMB sempre, ou LMB em Navegar)
            let rmb_drag = resp.dragged_by(egui::PointerButton::Secondary);
            let lmb_drag = resp.dragged_by(egui::PointerButton::Primary);
            if rmb_drag || (self.tool == ViewportTool::Navigate && lmb_drag) {
                let delta = resp.drag_delta();
                self.cam.yaw = (self.cam.yaw + delta.x * self.mouse_sens).rem_euclid(TAU);
                self.cam.pitch = (self.cam.pitch - delta.y * self.mouse_sens * 0.45).clamp(-0.9, 0.9);
            }

            // Scroll: raio do pincel ou velocidade da câmera
            if resp.hovered() {
                let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll.abs() > 0.1 {
                    if self.tool == ViewportTool::Sculpt {
                        self.brush.radius = (self.brush.radius + scroll * 0.08).clamp(2.0, 150.0);
                    } else {
                        self.move_speed = (self.move_speed + scroll * 0.5).clamp(20.0, 600.0);
                    }
                }
            }

            // Picking / escultura / colocação de modelos na viewport 3D
            let proj_pick = Projection::new(&self.cam, rw, rh);
            if !self.hover_from_minimap {
                self.hover_world = None;
            }
            if let Some(pos) = resp.hover_pos() {
                if rect.contains(pos) {
                    let sx = (pos.x - rect.left()) / rect.width().max(1.0) * rw as f32;
                    let sy = (pos.y - rect.top()) / rect.height().max(1.0) * rh as f32;
                    self.hover_world = self.voxel.pick(&self.terrain, &self.settings, &proj_pick, sx, sy);
                    self.hover_from_minimap = false;
                }
            }

            if let Some((wx, wy)) = self.hover_world {
                if !self.hover_from_minimap {
                    let lmb_down = resp.is_pointer_button_down_on()
                        && ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
                    if self.tool == ViewportTool::Sculpt && lmb_down {
                        if ctx.input(|i| i.pointer.primary_pressed())
                            && self.brush.mode == BrushMode::Flatten
                            && self.brush.flatten_pick_on_click
                        {
                            self.brush.flatten_height = self.terrain.height_at(wx, wy);
                        }
                        let ctrl = ctx.input(|i| i.modifiers.ctrl);
                        self.brush.apply(&mut self.terrain, wx, wy, dt, ctrl);
                    } else if self.tool == ViewportTool::PlaceModel && resp.clicked_by(egui::PointerButton::Primary) {
                        self.spawn_instance_at(self.selected_mesh, wx, wy);
                    }
                }
            }

            let overlay = if self.tool == ViewportTool::Sculpt {
                self.hover_world.map(|(x, y)| BrushOverlay {
                    x,
                    y,
                    radius: self.brush.radius,
                    color: self.brush.mode.overlay_color(),
                })
            } else {
                None
            };

            let elapsed = (perf_counter() - self.start_time) as f32;
            let t0 = perf_counter();
            let proj = self.voxel.render(&self.terrain, &self.sky, &self.cam, &self.settings, overlay, elapsed);
            let t1 = perf_counter();
            self.mesh_renderer.render(
                &self.meshes,
                &self.instances,
                self.selected_instance,
                &self.terrain,
                &self.settings,
                &proj,
                self.sky.fog_color,
                &mut self.voxel.color,
                &mut self.voxel.depth,
            );
            let t2 = perf_counter();
            let img = self.voxel.present();
            let t3 = perf_counter();

            self.voxel_ms = ((t1 - t0) * 1000.0) as f32;
            self.mesh_ms = ((t2 - t1) * 1000.0) as f32;
            self.present_ms = ((t3 - t2) * 1000.0) as f32;

            let filter = if self.linear_filter {
                egui::TextureOptions::LINEAR
            } else {
                egui::TextureOptions::NEAREST
            };
            let tex = self.viewport_tex.get_or_insert_with(|| ctx.load_texture("viewport", img.clone(), filter));
            tex.set(img, filter);
            ui.painter().image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        });
    }
}

impl eframe::App for VoxelSpaceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = perf_counter();
        let raw_dt = (now - self.last_frame_time).max(1e-5) as f32;
        self.last_frame_time = now;
        let dt = raw_dt.min(0.1);
        let inst_fps = (1.0 / raw_dt).min(999.0);
        self.fps_smooth = self.fps_smooth * 0.9 + inst_fps * 0.1;
        self.frame_ms = self.frame_ms * 0.9 + (raw_dt * 1000.0) * 0.1;

        self.hover_from_minimap = false;
        self.handle_dropped_files(ctx);
        if let Some((kind, path)) = self.browser.show(ctx) {
            self.handle_asset(kind, &path);
        }
        self.update_camera(ctx, dt);
        self.draw_top_bar(ctx);
        self.draw_status_bar(ctx);
        self.draw_left_panel(ctx);
        self.draw_minimap_panel(ctx, dt);
        self.draw_viewport(ctx, dt);

        ctx.request_repaint();
    }
}
