use crate::entities::{SceneNode, BLOCK_SIZE, MAP_H, MAP_W};
use crate::math3d::{Camera6DOF, Vec3};
use crate::pyrsutils::prelude::*;
use crate::renderer::SoftwareRasterizer;
use crate::textures::TextureManager;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
pub enum EditorTool {
    Select,
    PaintBlock,
    EditCollider,
    PlaceShip,
}

#[derive(PartialEq, Clone, Copy)]
pub enum BottomTab {
    Assets,
    Console,
}

pub struct DescentEditorApp {
    pub map: Matrix<SceneNode>,
    pub backup_map: Option<Matrix<SceneNode>>,
    pub cam: Camera6DOF,
    pub rasterizer: SoftwareRasterizer,
    pub textures: TextureManager,
    pub viewport_3d_tex: Option<egui::TextureHandle>,

    pub is_playing: bool,
    pub is_paused: bool,
    pub show_colliders: bool,

    pub active_tool: EditorTool,
    pub paint_tile_type: i32,
    pub selected_cell: (usize, usize),
    pub selected_tex_idx: usize,
    pub bottom_tab: BottomTab,
    pub import_path_input: String,
    pub map_file_input: String,
    pub logs: Vec<String>,

    pub is_shooting: bool,
    pub shoot_time: f64,
    pub shot_origin: Vec3,
    pub shot_impact: Vec3,

    pub ortho_zoom: f32,
    pub ortho_pan: egui::Vec2,
    pub dragging_handle: Option<usize>,

    pub last_time: f64,
    pub fps: f32,
}

impl DescentEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let default_node = SceneNode::from_tile(0, 0, 1);

        let mut app = Self {
            map: Matrix::new(MAP_H, MAP_W, default_node),
            backup_map: None,
            cam: Camera6DOF::default_ship(),
            rasterizer: SoftwareRasterizer::new(360, 240),
            textures: TextureManager::new(&cc.egui_ctx),
            viewport_3d_tex: None,
            is_playing: false,
            is_paused: false,
            show_colliders: true,
            active_tool: EditorTool::Select,
            paint_tile_type: 1,
            selected_cell: (1, 1),
            selected_tex_idx: 1,
            bottom_tab: BottomTab::Assets,
            import_path_input: String::from("TileSet1.png"),
            map_file_input: String::from("level1.d2map"),
            logs: Vec::new(),
            is_shooting: false,
            shoot_time: 0.0,
            shot_origin: Vec3::new(0.0, 0.0, 0.0),
            shot_impact: Vec3::new(0.0, 0.0, 0.0),
            ortho_zoom: 22.0,
            ortho_pan: egui::vec2(16.0, 16.0),
            dragging_handle: None,
            last_time: perf_counter(),
            fps: 60.0,
        };

        app.init_default_maze();
        app.log("Descent 2 Map Editor (POO + Herança + Polimorfismo) pronto.");
        app
    }

    pub fn log(&mut self, msg: &str) {
        let ts = strftime("%H:%M:%S", &gmtime_now());
        self.logs.append_item(format!("[{ts}] {msg}"));
    }

    fn init_default_maze(&mut self) {
        let raw_map: [[i32; MAP_W]; MAP_H] = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 1, 0, 2, 0, 1, 1, 0, 8, 0, 1, 1, 0, 1],
            [1, 0, 1, 0, 8, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1],
            [1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 10, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 0, 1, 1, 1, 0, 3, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1],
            [1, 0, 1, 1, 1, 1, 1, 1, 0, 3, 3, 3, 3, 1, 1, 1, 0, 1, 0, 1],
            [1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 1],
            [1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1],
            [1, 0, 9, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 9, 0, 0, 0, 1, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 11, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        for z in range(MAP_H as i64) {
            for x in range(MAP_W as i64) {
                let (zu, xu) = (z as usize, x as usize);
                self.map[(zu, xu)] = SceneNode::from_tile(xu, zu, raw_map[zu][xu]);
            }
        }
        self.rasterizer.static_dirty = true;
    }

    fn is_point_solid(&self, world_x: f32, world_z: f32, for_shot: bool) -> bool {
        let gx = ((world_x + BLOCK_SIZE * 0.5) / BLOCK_SIZE).floor() as i32;
        let gz = ((world_z + BLOCK_SIZE * 0.5) / BLOCK_SIZE).floor() as i32;
        if gx < 0 || gx >= MAP_W as i32 || gz < 0 || gz >= MAP_H as i32 {
            return true;
        }
        let cell_origin_x = gx as f32 * BLOCK_SIZE - BLOCK_SIZE * 0.5;
        let cell_origin_z = gz as f32 * BLOCK_SIZE - BLOCK_SIZE * 0.5;
        let lx = (world_x - cell_origin_x) / BLOCK_SIZE;
        let lz = (world_z - cell_origin_z) / BLOCK_SIZE;

        self.map[(gz as usize, gx as usize)]
            .as_entity()
            .blocks_movement(lx, lz, for_shot)
    }

    fn fire_laser(&mut self, now: f64) {
        self.shoot_time = now;
        self.is_shooting = true;
        self.cam.recoil_timer = 0.0;
        self.shot_origin = self.cam.pos;

        let mut ray = self.cam.pos;
        let mut hit_pos = self.cam.pos.add(self.cam.forward.mul(25.0));

        for _ in range(50) {
            ray = ray.add(self.cam.forward.mul(0.5));
            let rx = ((ray.x + BLOCK_SIZE * 0.5) / BLOCK_SIZE).floor() as i32;
            let rz = ((ray.z + BLOCK_SIZE * 0.5) / BLOCK_SIZE).floor() as i32;
            if rx >= 0 && rx < MAP_W as i32 && rz >= 0 && rz < MAP_H as i32 {
                if self.map[(rz as usize, rx as usize)]
                    .as_entity_mut()
                    .on_laser_hit()
                {
                    hit_pos = ray;
                    self.log(&format!("Alvo destruído em ({rx}, {rz})!"));
                    break;
                }
            }
            if self.is_point_solid(ray.x, ray.z, true) {
                hit_pos = ray;
                break;
            }
        }
        self.shot_impact = hit_pos;
    }

    fn update_simulation(&mut self, ctx: &egui::Context, dt: f32, now: f64) {
        let move_speed = 14.0 * dt;
        let rot_speed = 2.4 * dt;
        let mut move_vec = Vec3::new(0.0, 0.0, 0.0);
        let mut disparou = false;

        // Consulta se o teclado está livre ANTES de abrir o lock de ctx.input
        let kb_free = !ctx.wants_keyboard_input();
        if kb_free {
            ctx.input(|i| {
                if i.key_down(egui::Key::W) {
                    move_vec = move_vec.add(self.cam.forward.mul(move_speed));
                }
                if i.key_down(egui::Key::S) {
                    move_vec = move_vec.sub(self.cam.forward.mul(move_speed));
                }
                if i.key_down(egui::Key::A) {
                    move_vec = move_vec.sub(self.cam.right.mul(move_speed));
                }
                if i.key_down(egui::Key::D) {
                    move_vec = move_vec.add(self.cam.right.mul(move_speed));
                }
                if i.key_down(egui::Key::R) {
                    self.cam.pos.y += move_speed;
                }
                if i.key_down(egui::Key::F) {
                    self.cam.pos.y -= move_speed;
                }

                if i.key_down(egui::Key::ArrowUp) {
                    self.cam.pitch(rot_speed);
                }
                if i.key_down(egui::Key::ArrowDown) {
                    self.cam.pitch(-rot_speed);
                }
                if i.key_down(egui::Key::ArrowLeft) {
                    self.cam.yaw(-rot_speed);
                }
                if i.key_down(egui::Key::ArrowRight) {
                    self.cam.yaw(rot_speed);
                }
                if i.key_down(egui::Key::Q) {
                    self.cam.roll(-rot_speed);
                }
                if i.key_down(egui::Key::E) {
                    self.cam.roll(rot_speed);
                }
                if i.key_pressed(egui::Key::Space) {
                    disparou = true;
                }
            });
        }

        if disparou && !self.is_shooting {
            self.fire_laser(now);
        }

        if self.is_playing {
            if !self.is_point_solid(self.cam.pos.x + move_vec.x, self.cam.pos.z, false) {
                self.cam.pos.x += move_vec.x;
            }
            if !self.is_point_solid(self.cam.pos.x, self.cam.pos.z + move_vec.z, false) {
                self.cam.pos.z += move_vec.z;
            }
        } else {
            self.cam.pos = self.cam.pos.add(move_vec);
        }

        self.cam.recoil_timer += dt;

        if !self.is_paused {
            let ship_p = self.cam.pos;
            for z in range(MAP_H as i64) {
                for x in range(MAP_W as i64) {
                    self.map[(z as usize, x as usize)]
                        .as_entity_mut()
                        .update_entity(ship_p, dt);
                }
            }
        }
    }

    fn save_map_to_disk(&mut self) {
        let mut linhas = Vec::new();
        linhas.append_item(format!(
            "SPAWN,{},{},{}",
            self.cam.spawn_pos.x, self.cam.spawn_pos.y, self.cam.spawn_pos.z
        ));
        for z in range(MAP_H as i64) {
            for x in range(MAP_W as i64) {
                let b = self.map[(z as usize, x as usize)].as_entity().base();
                linhas.append_item(format!(
                    "CELL,{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    x,
                    z,
                    b.tile,
                    b.wall_tex,
                    b.floor_tex,
                    b.ceil_tex,
                    b.y_floor,
                    b.y_ceil,
                    b.solid,
                    b.blocks_shot,
                    b.col_min_x,
                    b.col_min_z,
                    b.col_max_x,
                    b.col_max_z
                ));
            }
        }
        let conteudo = join("\n", linhas);
        match write_text(&self.map_file_input, conteudo) {
            Ok(_) => self.log(&format!("Mapa salvo em '{}'", self.map_file_input)),
            Err(e) => self.log(&format!("Erro ao salvar: {e}")),
        }
    }

    fn load_map_from_disk(&mut self) {
        match read_text(&self.map_file_input) {
            Ok(conteudo) => {
                for linha in conteudo.lines() {
                    let p = split(strip(linha), ",");
                    if len(&p) >= 4 && p[0] == "SPAWN" {
                        self.cam.spawn_pos.x = p[1].parse().unwrap_or(4.0);
                        self.cam.spawn_pos.y = p[2].parse().unwrap_or(0.0);
                        self.cam.spawn_pos.z = p[3].parse().unwrap_or(4.0);
                        self.cam.reset_to_spawn();
                    } else if len(&p) >= 15 && p[0] == "CELL" {
                        let x: usize = p[1].parse().unwrap_or(0);
                        let z: usize = p[2].parse().unwrap_or(0);
                        let tile: i32 = p[3].parse().unwrap_or(1);
                        if x < MAP_W && z < MAP_H {
                            let mut node = SceneNode::from_tile(x, z, tile);
                            let b = node.as_entity_mut().base_mut();
                            b.wall_tex = p[4].parse().unwrap_or(1);
                            b.floor_tex = p[5].parse().unwrap_or(0);
                            b.ceil_tex = p[6].parse().unwrap_or(0);
                            b.y_floor = p[7].parse().unwrap_or(-2.0);
                            b.y_ceil = p[8].parse().unwrap_or(2.0);
                            b.solid = p[9].parse().unwrap_or(true);
                            b.blocks_shot = p[10].parse().unwrap_or(true);
                            b.col_min_x = p[11].parse().unwrap_or(0.0);
                            b.col_min_z = p[12].parse().unwrap_or(0.0);
                            b.col_max_x = p[13].parse().unwrap_or(1.0);
                            b.col_max_z = p[14].parse().unwrap_or(1.0);
                            self.map[(z, x)] = node;
                        }
                    }
                }
                self.rasterizer.static_dirty = true;
                self.log(&format!(
                    "Mapa '{}' carregado com sucesso.",
                    self.map_file_input
                ));
            }
            Err(e) => self.log(&format!("Erro ao carregar '{}': {e}", self.map_file_input)),
        }
    }

    fn project_point_to_rect(&self, world_p: Vec3, rect: egui::Rect) -> Option<egui::Pos2> {
        let d = world_p.sub(self.cam.pos);
        let vx = d.dot(self.cam.right);
        let vy = d.dot(self.cam.up);
        let vz = d.dot(self.cam.forward);
        if vz < 0.15 {
            return None;
        }
        let aspect = rect.height() / rect.width();
        let fov_rad = 1.0 / (45.0_f32.to_radians()).tan();
        let px = (vx * aspect * fov_rad) / vz;
        let py = (vy * fov_rad) / vz;
        Some(egui::pos2(
            rect.min.x + (px + 1.0) * 0.5 * rect.width(),
            rect.min.y + (1.0 - (py + 1.0) * 0.5) * rect.height(),
        ))
    }

    fn draw_top_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("unity_top_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Tools:").strong());
                ui.selectable_value(&mut self.active_tool, EditorTool::Select, "🔍 Selecionar");
                ui.selectable_value(
                    &mut self.active_tool,
                    EditorTool::PaintBlock,
                    "🖌 Pintar Bloco",
                );
                ui.selectable_value(
                    &mut self.active_tool,
                    EditorTool::EditCollider,
                    "🟩 Editar Colisão",
                );
                ui.selectable_value(
                    &mut self.active_tool,
                    EditorTool::PlaceShip,
                    "🚀 Posicionar Nave",
                );

                ui.separator();

                let play_label = if self.is_playing {
                    "⏹ Stop"
                } else {
                    "▶ Play"
                };
                if ui
                    .button(egui::RichText::new(play_label).strong())
                    .clicked()
                {
                    if !self.is_playing {
                        self.backup_map = Some(self.map.clone());
                        self.is_playing = true;
                        self.is_paused = false;
                        self.log("Modo Play 6DOF iniciado.");
                    } else {
                        if let Some(bkp) = self.backup_map.take() {
                            self.map = bkp;
                            self.rasterizer.static_dirty = true;
                        }
                        self.is_playing = false;
                        self.is_paused = false;
                        self.log("Modo Play encerrado (Estado original restaurado).");
                    }
                }

                if ui
                    .add_enabled(self.is_playing, egui::Button::new("⏸ Pause"))
                    .clicked()
                {
                    self.is_paused = !self.is_paused;
                }

                if ui.button("🔥 Disparar").clicked() {
                    self.fire_laser(perf_counter());
                }

                ui.separator();

                ui.checkbox(&mut self.show_colliders, "Box Colliders");
                ui.checkbox(&mut self.rasterizer.show_wireframe, "Wireframe 3D");

                let (mut rw, mut rh) = (self.rasterizer.fb_width, self.rasterizer.fb_height);
                egui::ComboBox::from_id_salt("res_selector")
                    .selected_text(format!("VGA {rw}x{rh}"))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(rw == 320, "320x200 (DOS Mode 13h)")
                            .clicked()
                        {
                            rw = 320;
                            rh = 200;
                        }
                        if ui.selectable_label(rw == 360, "360x240 (Mode X)").clicked() {
                            rw = 360;
                            rh = 240;
                        }
                        if ui.selectable_label(rw == 480, "480x300 (SVGA)").clicked() {
                            rw = 480;
                            rh = 300;
                        }
                    });
                self.rasterizer.set_resolution(rw, rh);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("FPS: {:.0}", self.fps)).monospace());
                });
            });
        });
    }

    fn draw_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("unity_hierarchy_panel")
            .resizable(true)
            .default_width(215.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new("🚀 Player Ship (6DOF)")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(format!(
                                "Pos: ({:.1}, {:.1}, {:.1})",
                                self.cam.pos.x, self.cam.pos.y, self.cam.pos.z
                            ));
                            if ui.button("Resetar para Spawn").clicked() {
                                self.cam.reset_to_spawn();
                            }
                        });

                    egui::CollapsingHeader::new("🚪 Portas & 🌀 Ventoinhas")
                        .default_open(true)
                        .show(ui, |ui| {
                            for z in range(MAP_H as i64) {
                                for x in range(MAP_W as i64) {
                                    let (zu, xu) = (z as usize, x as usize);
                                    let node = &self.map[(zu, xu)];
                                    if matches!(node, SceneNode::Door(_) | SceneNode::Fan(_)) {
                                        let label = node.as_entity().hierarchy_label();
                                        let sel = self.selected_cell == (xu, zu);
                                        if ui.selectable_label(sel, label).clicked() {
                                            self.selected_cell = (xu, zu);
                                        }
                                    }
                                }
                            }
                        });
                });
            });
    }

    fn draw_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("unity_inspector_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                let (sx, sz) = self.selected_cell;
                let header_txt = self.map[(sz, sx)].as_entity().hierarchy_label();
                ui.label(egui::RichText::new(format!("📦 {header_txt}")).strong());
                ui.add_space(4.0);

                let tex_names: Vec<String> =
                    self.textures.list.iter().map(|t| t.name.clone()).collect();
                let mut rebuild_node_tile: Option<i32> = None;
                let mut mesh_changed = false;

                {
                    let node = &mut self.map[(sz, sx)];
                    let cur_tile = node.as_entity().base().tile;

                    egui::CollapsingHeader::new("📐 Transform & Tipo de Objeto")
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::ComboBox::from_label("Tipo")
                                .selected_text(match cur_tile {
                                    0 => "0: Corredor Vazio",
                                    1 => "1: Parede Metálica",
                                    2 => "2: Parede Oxidada",
                                    3 => "3: Tubulação (Pipes)",
                                    8 => "8: Porta Automática (X)",
                                    9 => "9: Porta Automática (Z)",
                                    10 => "10: Ventoinha (X)",
                                    11 => "11: Ventoinha (Z)",
                                    _ => "Custom",
                                })
                                .show_ui(ui, |ui| {
                                    for (val, label) in [
                                        (0, "0: Corredor Vazio"),
                                        (1, "1: Parede Metálica"),
                                        (2, "2: Parede Oxidada"),
                                        (3, "3: Tubulação (Pipes)"),
                                        (8, "8: Porta Automática (X)"),
                                        (9, "9: Porta Automática (Z)"),
                                        (10, "10: Ventoinha (X)"),
                                        (11, "11: Ventoinha (Z)"),
                                    ] {
                                        if ui.selectable_label(cur_tile == val, label).clicked() {
                                            rebuild_node_tile = Some(val);
                                        }
                                    }
                                });

                            let b = node.as_entity_mut().base_mut();
                            if ui
                                .add(
                                    egui::Slider::new(&mut b.y_floor, -6.0..=0.0)
                                        .text("Altura Piso"),
                                )
                                .changed()
                            {
                                mesh_changed = true;
                            }
                            if ui
                                .add(
                                    egui::Slider::new(&mut b.y_ceil, 0.5..=6.0).text("Altura Teto"),
                                )
                                .changed()
                            {
                                mesh_changed = true;
                            }
                        });

                    egui::CollapsingHeader::new("🎨 Texturização (Materials)")
                        .default_open(true)
                        .show(ui, |ui| {
                            let b = node.as_entity_mut().base_mut();
                            let mut combo_tex = |ui: &mut egui::Ui, lbl: &str, idx: &mut usize| {
                                let atual = tex_names.get(*idx).cloned().unwrap_or_default();
                                egui::ComboBox::from_label(lbl)
                                    .selected_text(atual)
                                    .show_ui(ui, |ui| {
                                        for (i, nome) in enumerate(&tex_names) {
                                            if ui.selectable_value(idx, i, nome).changed() {
                                                mesh_changed = true;
                                            }
                                        }
                                    });
                            };

                            combo_tex(ui, "Parede / Objeto", &mut b.wall_tex);
                            combo_tex(ui, "Piso (Floor)", &mut b.floor_tex);
                            combo_tex(ui, "Teto (Ceiling)", &mut b.ceil_tex);

                            if ui
                                .button("Aplicar Textura Selecionada nos Assets")
                                .clicked()
                            {
                                b.wall_tex = self.selected_tex_idx;
                                mesh_changed = true;
                            }
                        });

                    egui::CollapsingHeader::new("🟩 Box Collider (Pontos de Colisão)")
                        .default_open(true)
                        .show(ui, |ui| {
                            match node {
                                SceneNode::Door(d) => {
                                    ui.add(
                                        egui::Slider::new(&mut d.open_factor, 0.0..=1.0)
                                            .text("Abertura Porta"),
                                    );
                                    ui.checkbox(&mut d.is_blocking_x, "Bloquear no Eixo X");
                                }
                                SceneNode::Fan(f) => {
                                    ui.checkbox(&mut f.destroyed, "Ventoinha Destruída");
                                    ui.checkbox(&mut f.is_blocking_x, "Alinhar no Eixo X");
                                }
                                SceneNode::Block(_) => {}
                            }

                            let b = node.as_entity_mut().base_mut();
                            ui.checkbox(&mut b.solid, "Sólido (Colisão com a Nave)");
                            ui.checkbox(&mut b.blocks_shot, "Bloqueia Disparo Laser");
                            ui.separator();

                            let max_x = b.col_max_x;
                            ui.add(egui::Slider::new(&mut b.col_min_x, 0.0..=max_x).text("Min X"));
                            let min_x = b.col_min_x;
                            ui.add(egui::Slider::new(&mut b.col_max_x, min_x..=1.0).text("Max X"));

                            let max_z = b.col_max_z;
                            ui.add(egui::Slider::new(&mut b.col_min_z, 0.0..=max_z).text("Min Z"));
                            let min_z = b.col_min_z;
                            ui.add(egui::Slider::new(&mut b.col_max_z, min_z..=1.0).text("Max Z"));
                        });
                }

                if let Some(novo_tile) = rebuild_node_tile {
                    self.map[(sz, sx)] = SceneNode::from_tile(sx, sz, novo_tile);
                    mesh_changed = true;
                }
                if mesh_changed {
                    self.rasterizer.static_dirty = true;
                }

                ui.separator();
                egui::CollapsingHeader::new("🖌 Pincel de Edição Rápida")
                    .default_open(true)
                    .show(ui, |ui| {
                        for (val, nome) in [
                            (0, "Corredor Vazio (0)"),
                            (1, "Parede Metálica (1)"),
                            (2, "Parede Oxidada (2)"),
                            (3, "Tubulação (3)"),
                            (8, "Porta Eixo X (8)"),
                            (9, "Porta Eixo Z (9)"),
                            (10, "Ventoinha X (10)"),
                            (11, "Ventoinha Z (11)"),
                        ] {
                            ui.radio_value(&mut self.paint_tile_type, val, nome);
                        }
                    });
            });
    }

    fn draw_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("unity_bottom_panel")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.bottom_tab,
                        BottomTab::Assets,
                        "📁 Project / Assets (Texturas)",
                    );
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Console, "🖥 Console");
                    ui.separator();
                    ui.label("Arquivo:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.map_file_input).desired_width(120.0),
                    );
                    if ui.button("💾 Salvar Mapa").clicked() {
                        self.save_map_to_disk();
                    }
                    if ui.button("📂 Carregar Mapa").clicked() {
                        self.load_map_from_disk();
                    }
                });
                ui.separator();

                match self.bottom_tab {
                    BottomTab::Assets => {
                        ui.horizontal(|ui| {
                            ui.label("Caminho (.png/.jpg):");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.import_path_input)
                                    .desired_width(210.0),
                            );
                            if ui.button("➕ Importar Textura").clicked() {
                                let p = self.import_path_input.clone();
                                match self.textures.load_from_disk(ctx, &p) {
                                    Ok(msg) => self.log(&msg),
                                    Err(err) => self.log(&err),
                                }
                            }
                            if ui.button("🔄 Escanear Pasta").clicked() {
                                let n = self.textures.scan_current_dir(ctx);
                                self.log(&format!(
                                    "{n} nova(s) textura(s) encontrada(s) no diretório."
                                ));
                            }
                        });
                        ui.add_space(4.0);

                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (sx, sz) = self.selected_cell;
                                for i in 0..len(&self.textures.list) {
                                    let is_sel = self.selected_tex_idx == i;
                                    ui.vertical(|ui| {
                                        let (rect, resp) = ui.allocate_exact_size(
                                            egui::vec2(60.0, 60.0),
                                            egui::Sense::click(),
                                        );
                                        ui.painter().image(
                                            self.textures.list[i].handle.id(),
                                            rect,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
                                            egui::Color32::WHITE,
                                        );
                                        let col = if is_sel {
                                            egui::Color32::from_rgb(0, 220, 255)
                                        } else {
                                            egui::Color32::DARK_GRAY
                                        };
                                        ui.painter().rect_stroke(
                                            rect,
                                            2.0,
                                            egui::Stroke::new(if is_sel { 2.5 } else { 1.0 }, col),
                                            egui::StrokeKind::Middle,
                                        );
                                        if resp.clicked() {
                                            self.selected_tex_idx = i;
                                            self.map[(sz, sx)]
                                                .as_entity_mut()
                                                .base_mut()
                                                .wall_tex = i;
                                            self.rasterizer.static_dirty = true;
                                        }
                                        ui.label(
                                            egui::RichText::new(&self.textures.list[i].name)
                                                .small(),
                                        );
                                    });
                                }
                            });
                        });
                    }
                    BottomTab::Console => {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for l in &self.logs {
                                    ui.monospace(l);
                                }
                            });
                    }
                }
            });
    }

    pub fn draw_2d_ortho_viewport(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 20, 24));

        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.ortho_zoom = (self.ortho_zoom + scroll * 0.05).clamp(10.0, 55.0);
            }
        }
        if resp.dragged_by(egui::PointerButton::Middle)
            || resp.dragged_by(egui::PointerButton::Secondary)
        {
            self.ortho_pan += resp.drag_delta();
        }

        let origin = rect.min + self.ortho_pan;
        let cell_px = self.ortho_zoom;

        for z in range(MAP_H as i64) {
            for x in range(MAP_W as i64) {
                let (zu, xu) = (z as usize, x as usize);
                let b = self.map[(zu, xu)].as_entity().base();
                let min_p = egui::pos2(
                    origin.x + xu as f32 * cell_px,
                    origin.y + zu as f32 * cell_px,
                );
                let max_p = egui::pos2(min_p.x + cell_px, min_p.y + cell_px);
                let c_rect = egui::Rect::from_min_max(min_p, max_p);

                let fill = match b.tile {
                    0 => egui::Color32::from_rgb(30, 34, 42),
                    1 => egui::Color32::from_rgb(75, 90, 115),
                    2 => egui::Color32::from_rgb(125, 75, 50),
                    3 => egui::Color32::from_rgb(145, 130, 55),
                    8 | 9 => egui::Color32::from_rgb(180, 120, 30),
                    10 | 11 => egui::Color32::from_rgb(50, 165, 175),
                    _ => egui::Color32::GRAY,
                };
                painter.rect_filled(c_rect, 0.0, fill);
                painter.rect_stroke(
                    c_rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 45, 55)),
                    egui::StrokeKind::Middle,
                );

                if self.show_colliders && b.solid && b.tile != 0 {
                    let c_min = egui::pos2(
                        min_p.x + b.col_min_x * cell_px,
                        min_p.y + b.col_min_z * cell_px,
                    );
                    let c_max = egui::pos2(
                        min_p.x + b.col_max_x * cell_px,
                        min_p.y + b.col_max_z * cell_px,
                    );
                    painter.rect_stroke(
                        egui::Rect::from_min_max(c_min, c_max),
                        0.0,
                        egui::Stroke::new(1.2, egui::Color32::from_rgb(40, 235, 80)),
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }

        let (sx, sz) = self.selected_cell;
        let sel_min = egui::pos2(
            origin.x + sx as f32 * cell_px,
            origin.y + sz as f32 * cell_px,
        );
        painter.rect_stroke(
            egui::Rect::from_min_size(sel_min, egui::vec2(cell_px, cell_px)),
            0.0,
            egui::Stroke::new(2.0, egui::Color32::YELLOW),
            egui::StrokeKind::Middle,
        );

        let b_sel = self.map[(sz, sx)].as_entity().base();
        let p_min = egui::pos2(
            sel_min.x + b_sel.col_min_x * cell_px,
            sel_min.y + b_sel.col_min_z * cell_px,
        );
        let p_max = egui::pos2(
            sel_min.x + b_sel.col_max_x * cell_px,
            sel_min.y + b_sel.col_max_z * cell_px,
        );
        let corners = [
            p_min,
            egui::pos2(p_max.x, p_min.y),
            p_max,
            egui::pos2(p_min.x, p_max.y),
        ];
        for pt in corners {
            painter.circle_filled(pt, 4.0, egui::Color32::from_rgb(50, 255, 90));
        }

        if let Some(pos) = resp.interact_pointer_pos() {
            if self.active_tool == EditorTool::EditCollider {
                if resp.drag_started_by(egui::PointerButton::Primary) {
                    self.dragging_handle = corners.iter().position(|pt| pt.distance(pos) < 9.0);
                }
                if resp.dragged_by(egui::PointerButton::Primary) {
                    if let Some(h_idx) = self.dragging_handle {
                        let lx = ((pos.x - sel_min.x) / cell_px).clamp(0.0, 1.0);
                        let lz = ((pos.y - sel_min.y) / cell_px).clamp(0.0, 1.0);
                        let b = self.map[(sz, sx)].as_entity_mut().base_mut();
                        match h_idx {
                            0 => {
                                b.col_min_x = lx.min(b.col_max_x);
                                b.col_min_z = lz.min(b.col_max_z);
                            }
                            1 => {
                                b.col_max_x = lx.max(b.col_min_x);
                                b.col_min_z = lz.min(b.col_max_z);
                            }
                            2 => {
                                b.col_max_x = lx.max(b.col_min_x);
                                b.col_max_z = lz.max(b.col_min_z);
                            }
                            _ => {
                                b.col_min_x = lx.min(b.col_max_x);
                                b.col_max_z = lz.max(b.col_min_z);
                            }
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.dragging_handle = None;
                }
            }

            if (resp.clicked() || resp.dragged_by(egui::PointerButton::Primary))
                && self.dragging_handle.is_none()
            {
                let gx = ((pos.x - origin.x) / cell_px).floor() as i32;
                let gz = ((pos.y - origin.y) / cell_px).floor() as i32;
                if gx >= 0 && gx < MAP_W as i32 && gz >= 0 && gz < MAP_H as i32 {
                    let (xu, zu) = (gx as usize, gz as usize);
                    match self.active_tool {
                        EditorTool::Select | EditorTool::EditCollider => {
                            self.selected_cell = (xu, zu);
                        }
                        EditorTool::PaintBlock => {
                            self.selected_cell = (xu, zu);
                            let mut novo = SceneNode::from_tile(xu, zu, self.paint_tile_type);
                            if self.paint_tile_type >= 1 && self.paint_tile_type <= 3 {
                                novo.as_entity_mut().base_mut().wall_tex = self.selected_tex_idx;
                            }
                            self.map[(zu, xu)] = novo;
                            self.rasterizer.static_dirty = true;
                        }
                        EditorTool::PlaceShip => {
                            self.cam.pos.x = xu as f32 * BLOCK_SIZE;
                            self.cam.pos.z = zu as f32 * BLOCK_SIZE;
                            self.cam.spawn_pos = self.cam.pos;
                        }
                    }
                }
            }
        }

        let ship_x = origin.x + (self.cam.pos.x / BLOCK_SIZE + 0.5) * cell_px;
        let ship_z = origin.y + (self.cam.pos.z / BLOCK_SIZE + 0.5) * cell_px;
        let ship_pos = egui::pos2(ship_x, ship_z);
        let dir_end = egui::pos2(
            ship_x + self.cam.forward.x * cell_px * 1.4,
            ship_z + self.cam.forward.z * cell_px * 1.4,
        );
        let fov_l = egui::pos2(
            ship_x + (self.cam.forward.x - self.cam.right.x) * cell_px * 2.2,
            ship_z + (self.cam.forward.z - self.cam.right.z) * cell_px * 2.2,
        );
        let fov_r = egui::pos2(
            ship_x + (self.cam.forward.x + self.cam.right.x) * cell_px * 2.2,
            ship_z + (self.cam.forward.z + self.cam.right.z) * cell_px * 2.2,
        );
        painter.line_segment(
            [ship_pos, fov_l],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 200, 255, 110)),
        );
        painter.line_segment(
            [ship_pos, fov_r],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 200, 255, 110)),
        );
        painter.line_segment(
            [ship_pos, dir_end],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 80, 80)),
        );
        painter.circle_filled(ship_pos, 5.0, egui::Color32::from_rgb(0, 220, 255));

        painter.text(
            rect.min + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            "Viewport 1: 2D Ortho / Top View",
            egui::FontId::monospace(11.5),
            egui::Color32::WHITE,
        );
    }

    pub fn draw_3d_perspective_viewport(&mut self, ui: &mut egui::Ui, rect: egui::Rect, now: f64) {
        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if resp.dragged_by(egui::PointerButton::Secondary) {
            let delta = resp.drag_delta();
            self.cam.yaw(delta.x * 0.007);
            self.cam.pitch(-delta.y * 0.007);
        }
        if resp.clicked_by(egui::PointerButton::Primary) && self.is_playing && !self.is_shooting {
            self.fire_laser(now);
        }

        let painter = ui.painter_at(rect);
        if let Some(tex) = &self.viewport_3d_tex {
            painter.image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        if self.show_colliders {
            let (sx, sz) = self.selected_cell;
            let b = self.map[(sz, sx)].as_entity().base();
            let ox = sx as f32 * BLOCK_SIZE - BLOCK_SIZE * 0.5;
            let oz = sz as f32 * BLOCK_SIZE - BLOCK_SIZE * 0.5;
            let x0 = ox + b.col_min_x * BLOCK_SIZE;
            let x1 = ox + b.col_max_x * BLOCK_SIZE;
            let z0 = oz + b.col_min_z * BLOCK_SIZE;
            let z1 = oz + b.col_max_z * BLOCK_SIZE;
            let (y0, y1) = (b.y_floor, b.y_ceil);

            let verts = [
                Vec3::new(x0, y0, z0),
                Vec3::new(x1, y0, z0),
                Vec3::new(x1, y0, z1),
                Vec3::new(x0, y0, z1),
                Vec3::new(x0, y1, z0),
                Vec3::new(x1, y1, z0),
                Vec3::new(x1, y1, z1),
                Vec3::new(x0, y1, z1),
            ];
            let edges = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(50, 255, 90));
            for (a, b_idx) in edges {
                if let (Some(pa), Some(pb)) = (
                    self.project_point_to_rect(verts[a], rect),
                    self.project_point_to_rect(verts[b_idx], rect),
                ) {
                    painter.line_segment([pa, pb], stroke);
                }
            }
        }

        if self.is_shooting {
            let elapsed = (now - self.shoot_time) as f32;
            if elapsed < 0.18 {
                let travel_t = (elapsed / 0.08).min(1.0);
                let bolt_tip = self.shot_origin.lerp(self.shot_impact, travel_t);
                let gun_l = egui::pos2(rect.center().x - rect.width() * 0.18, rect.max.y - 24.0);
                let gun_r = egui::pos2(rect.center().x + rect.width() * 0.18, rect.max.y - 24.0);

                if let Some(tip_pos) = self.project_point_to_rect(bolt_tip, rect) {
                    let laser_stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 70, 40));
                    painter.line_segment([gun_l, tip_pos], laser_stroke);
                    painter.line_segment([gun_r, tip_pos], laser_stroke);

                    if travel_t >= 1.0 {
                        for i in 0..8 {
                            let ang = i as f32 / 8.0 * std::f32::consts::TAU;
                            let len_s = if i % 2 == 0 { 16.0 } else { 8.0 };
                            let end_p = egui::pos2(
                                tip_pos.x + ang.cos() * len_s,
                                tip_pos.y + ang.sin() * len_s,
                            );
                            painter.line_segment(
                                [tip_pos, end_p],
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 215, 90)),
                            );
                        }
                    }
                }
            } else {
                self.is_shooting = false;
            }
        }

        let center = rect.center();
        let ret_col = egui::Color32::from_rgb(0, 235, 100);
        painter.line_segment(
            [
                center - egui::vec2(10.0, 0.0),
                center - egui::vec2(3.0, 0.0),
            ],
            egui::Stroke::new(1.5, ret_col),
        );
        painter.line_segment(
            [
                center + egui::vec2(3.0, 0.0),
                center + egui::vec2(10.0, 0.0),
            ],
            egui::Stroke::new(1.5, ret_col),
        );
        painter.line_segment(
            [
                center - egui::vec2(0.0, 10.0),
                center - egui::vec2(0.0, 3.0),
            ],
            egui::Stroke::new(1.5, ret_col),
        );
        painter.line_segment(
            [
                center + egui::vec2(0.0, 3.0),
                center + egui::vec2(0.0, 10.0),
            ],
            egui::Stroke::new(1.5, ret_col),
        );

        let mode_str = if self.is_playing {
            "PLAY MODE (6DOF)"
        } else {
            "SCENE PREVIEW"
        };
        painter.text(
            rect.min + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("Viewport 2: 3D Perspective [{mode_str}]"),
            egui::FontId::monospace(11.5),
            egui::Color32::from_rgb(0, 255, 140),
        );
    }
}

impl eframe::App for DescentEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = perf_counter();
        let dt = ((now - self.last_time) as f32).clamp(0.0001, 0.1);
        self.last_time = now;
        self.fps = self.fps * 0.9 + (1.0 / dt) * 0.1;

        self.update_simulation(ctx, dt, now);
        self.rasterizer
            .render(&self.map, &self.cam, &self.textures, now, self.is_playing);

        // Atualiza a textura 3D no contexto ANTES de desenhar os painéis da UI
        let img = egui::ColorImage {
            size: [self.rasterizer.fb_width, self.rasterizer.fb_height],
            pixels: self.rasterizer.color_buf.clone(),
        };
        let tex_opts = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            ..Default::default()
        };
        let tex = self
            .viewport_3d_tex
            .get_or_insert_with(|| ctx.load_texture("dos_3d_fb", img.clone(), tex_opts));
        tex.set(img, tex_opts);

        self.draw_top_toolbar(ctx);
        self.draw_bottom_panel(ctx);
        self.draw_hierarchy_panel(ctx);
        self.draw_inspector_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_rect_before_wrap();
            let half_w = (avail.width() - 6.0) * 0.5;
            let rect_2d = egui::Rect::from_min_size(avail.min, egui::vec2(half_w, avail.height()));
            let rect_3d = egui::Rect::from_min_size(
                egui::pos2(avail.min.x + half_w + 6.0, avail.min.y),
                egui::vec2(half_w, avail.height()),
            );

            self.draw_2d_ortho_viewport(ui, rect_2d);
            self.draw_3d_perspective_viewport(ui, rect_3d, now);
        });

        ctx.request_repaint();
    }
}
