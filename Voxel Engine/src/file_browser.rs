//! Navegador de arquivos embutido (sem dependências nativas), escrito no estilo
//! "pythônico" com `pyrsutils` (`listdir`, `isdir`, `join_path`, `dirname`...).

use crate::pyrsutils::prelude::*;
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Heightmap,
    Colormap,
    Sky,
    WaterTexture,
    BrushTexture,
    ObjModel,
    MeshTexture(usize),
}

impl AssetKind {
    fn title(self) -> &'static str {
        match self {
            Self::Heightmap => "Abrir Heightmap (tons de cinza)",
            Self::Colormap => "Abrir Colormap (textura do solo)",
            Self::Sky => "Abrir textura panorâmica do céu",
            Self::WaterTexture => "Abrir textura da água",
            Self::BrushTexture => "Abrir textura do pincel",
            Self::ObjModel => "Abrir modelo Wavefront (.obj)",
            Self::MeshTexture(_) => "Abrir textura do modelo",
        }
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::ObjModel => &[".obj"],
            _ => &[".png", ".jpg", ".jpeg", ".bmp", ".tga", ".tif", ".tiff"],
        }
    }
}

pub struct FileBrowser {
    pub open: bool,
    kind: AssetKind,
    cwd: String,
    entries: Vec<(String, bool)>,
    path_input: String,
    error: Option<String>,
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl FileBrowser {
    pub fn new() -> Self {
        Self {
            open: false,
            kind: AssetKind::Heightmap,
            cwd: getcwd().unwrap_or_else(|_| ".".into()),
            entries: Vec::new(),
            path_input: String::new(),
            error: None,
        }
    }

    pub fn open_for(&mut self, kind: AssetKind) {
        self.kind = kind;
        self.open = true;
        self.refresh();
    }

    fn refresh(&mut self) {
        self.entries.clear();
        self.error = None;
        match listdir(&self.cwd) {
            Ok(nomes) => {
                let exts = self.kind.extensions();
                for nome in sorted(nomes) {
                    if startswith(&nome, ".") {
                        continue;
                    }
                    let full = join_path(&[&self.cwd, &nome]);
                    if isdir(&full) {
                        self.entries.append_item((nome, true));
                    } else {
                        let l = lower(&nome);
                        if exts.iter().any(|e| endswith(&l, e)) {
                            self.entries.append_item((nome, false));
                        }
                    }
                }
                self.entries.sort_by_key(|(n, d)| (!*d, lower(n)));
            }
            Err(e) => self.error = Some(format!("Não foi possível listar '{}': {e}", self.cwd)),
        }
    }

    fn change_dir(&mut self, dir: String) {
        self.cwd = abspath(&dir).unwrap_or(dir);
        self.refresh();
    }

    /// Desenha a janela; devolve `(tipo, caminho)` quando o usuário escolhe um arquivo.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<(AssetKind, String)> {
        if !self.open {
            return None;
        }
        let mut result = None;
        let mut nav: Option<String> = None;
        let mut open = self.open;
        egui::Window::new(self.kind.title())
            .open(&mut open)
            .default_size([480.0, 440.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("⬆ Acima").clicked() {
                        let d = dirname(&self.cwd);
                        if !d.is_empty() {
                            nav = Some(d);
                        }
                    }
                    if ui.button("🏠 Home").clicked() {
                        nav = Some(expanduser("~"));
                    }
                    if ui.button("📁 Projeto").clicked() {
                        nav = getcwd().ok();
                    }
                    if ui.button("⟳").clicked() {
                        nav = Some(self.cwd.clone());
                    }
                });
                ui.label(egui::RichText::new(&self.cwd).monospace().small());
                if let Some(e) = &self.error {
                    ui.colored_label(egui::Color32::LIGHT_RED, e);
                }
                ui.separator();
                egui::ScrollArea::vertical().max_height(280.0).auto_shrink([false, false]).show(ui, |ui| {
                    for (nome, is_dir) in &self.entries {
                        let full = join_path(&[&self.cwd, nome]);
                        if *is_dir {
                            if ui.selectable_label(false, format!("📁 {nome}")).double_clicked() {
                                nav = Some(full);
                            }
                        } else {
                            let icon = if self.kind == AssetKind::ObjModel { "📦" } else { "🖼" };
                            let resp = ui.selectable_label(self.path_input == full, format!("{icon} {nome}"));
                            if resp.clicked() {
                                self.path_input = full.clone();
                            }
                            if resp.double_clicked() {
                                result = Some((self.kind, full));
                            }
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.path_input).desired_width(330.0).hint_text("caminho do arquivo"));
                    if ui.button("✅ Abrir").clicked() && !self.path_input.is_empty() {
                        result = Some((self.kind, self.path_input.clone()));
                    }
                });
                ui.small("Clique duplo em uma pasta para entrar / em um arquivo para abrir.");
            });
        if let Some(d) = nav {
            self.change_dir(d);
        }
        self.open = open && result.is_none();
        result
    }
}
