#![allow(dead_code)]
use crate::math3d::{TexCoord, Vec3};
use crate::pyrsutils::prelude::*;

pub const BLOCK_SIZE: f32 = 4.0;
pub const MAP_W: usize = 20;
pub const MAP_H: usize = 20;

#[derive(Clone, Copy)]
pub struct Triangle3D {
    pub p: [Vec3; 3],
    pub uv: [TexCoord; 3],
    pub normal: Vec3,
    pub tex_idx: usize,
}

impl Triangle3D {
    pub fn append_quad(
        tris: &mut Vec<Triangle3D>,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        tex_idx: usize,
        double_sided: bool,
    ) {
        let normal = v1.sub(v0).cross(v2.sub(v0)).normalize();
        tris.append_item(Triangle3D {
            p: [v0, v1, v2],
            uv: [
                TexCoord::new(0.0, 0.0),
                TexCoord::new(1.0, 0.0),
                TexCoord::new(1.0, 1.0),
            ],
            normal,
            tex_idx,
        });
        tris.append_item(Triangle3D {
            p: [v0, v2, v3],
            uv: [
                TexCoord::new(0.0, 0.0),
                TexCoord::new(1.0, 1.0),
                TexCoord::new(0.0, 1.0),
            ],
            normal,
            tex_idx,
        });
        if double_sided {
            let rev = normal.mul(-1.0);
            tris.append_item(Triangle3D {
                p: [v1, v0, v3],
                uv: [
                    TexCoord::new(0.0, 0.0),
                    TexCoord::new(1.0, 0.0),
                    TexCoord::new(1.0, 1.0),
                ],
                normal: rev,
                tex_idx,
            });
            tris.append_item(Triangle3D {
                p: [v1, v3, v2],
                uv: [
                    TexCoord::new(0.0, 0.0),
                    TexCoord::new(1.0, 1.0),
                    TexCoord::new(0.0, 1.0),
                ],
                normal: rev,
                tex_idx,
            });
        }
    }
}

class! {
    struct MapObject {
        gx: usize,
        gz: usize,
        tile: i32,
        wall_tex: usize,
        floor_tex: usize,
        ceil_tex: usize,
        y_floor: f32,
        y_ceil: f32,
        solid: bool,
        blocks_shot: bool,
        col_min_x: f32,
        col_min_z: f32,
        col_max_x: f32,
        col_max_z: f32,
    }
    impl {
        fn world_center(&self) -> Vec3 {
            Vec3::new(self.gx as f32 * BLOCK_SIZE, 0.0, self.gz as f32 * BLOCK_SIZE)
        }
        fn contains_local_point(&self, lx: f32, lz: f32) -> bool {
            lx >= self.col_min_x
                && lx <= self.col_max_x
                && lz >= self.col_min_z
                && lz <= self.col_max_z
        }
    }
}

class! {
    struct DoorEntity : MapObject {
        open_factor: f32,
        is_blocking_x: bool,
    }
    impl {}
}

class! {
    struct FanEntity : MapObject {
        destroyed: bool,
        is_blocking_x: bool,
    }
    impl {}
}

pub trait SceneEntity {
    fn base(&self) -> &MapObject;
    fn base_mut(&mut self) -> &mut MapObject;
    fn is_empty_space(&self) -> bool;
    fn hierarchy_label(&self) -> String;
    fn update_entity(&mut self, ship_pos: Vec3, dt: f32);
    fn blocks_movement(&self, lx: f32, lz: f32, for_shot: bool) -> bool;
    fn on_laser_hit(&mut self) -> bool;
    fn emit_dynamic_geometry(&self, tris: &mut Vec<Triangle3D>, anim_frame: usize, max_tex: usize);
}

impl SceneEntity for MapObject {
    fn base(&self) -> &MapObject {
        self
    }
    fn base_mut(&mut self) -> &mut MapObject {
        self
    }
    fn is_empty_space(&self) -> bool {
        self.tile == 0
    }
    fn hierarchy_label(&self) -> String {
        format!("Block ({}, {}) [Tipo {}]", self.gx, self.gz, self.tile)
    }
    fn update_entity(&mut self, _ship_pos: Vec3, _dt: f32) {}
    fn blocks_movement(&self, lx: f32, lz: f32, for_shot: bool) -> bool {
        if self.tile == 0 {
            return false;
        }
        let active = if for_shot {
            self.blocks_shot
        } else {
            self.solid
        };
        active && self.contains_local_point(lx, lz)
    }
    fn on_laser_hit(&mut self) -> bool {
        false
    }
    fn emit_dynamic_geometry(&self, _tris: &mut Vec<Triangle3D>, _anim: usize, _max_tex: usize) {}
}

impl SceneEntity for DoorEntity {
    fn base(&self) -> &MapObject {
        &self.base
    }
    fn base_mut(&mut self) -> &mut MapObject {
        &mut self.base
    }
    fn is_empty_space(&self) -> bool {
        true
    }
    fn hierarchy_label(&self) -> String {
        let eixo = if self.is_blocking_x { "X" } else { "Z" };
        format!("Door ({}, {}) [Eixo {eixo}]", self.gx, self.gz)
    }
    fn update_entity(&mut self, ship_pos: Vec3, dt: f32) {
        if ship_pos.dist(self.world_center()) < 14.0 {
            self.open_factor = (self.open_factor + dt * 2.0).min(1.0);
        } else {
            self.open_factor = (self.open_factor - dt * 2.0).max(0.0);
        }
    }
    fn blocks_movement(&self, lx: f32, lz: f32, for_shot: bool) -> bool {
        if self.open_factor >= 0.7 {
            return false;
        }
        let active = if for_shot {
            self.blocks_shot
        } else {
            self.solid
        };
        active && self.contains_local_point(lx, lz)
    }
    fn on_laser_hit(&mut self) -> bool {
        false
    }
    fn emit_dynamic_geometry(&self, tris: &mut Vec<Triangle3D>, _anim: usize, max_tex: usize) {
        let cx = self.gx as f32 * BLOCK_SIZE;
        let cz = self.gz as f32 * BLOCK_SIZE;
        let x0 = cx - BLOCK_SIZE * 0.5;
        let x1 = cx + BLOCK_SIZE * 0.5;
        let z0 = cz - BLOCK_SIZE * 0.5;
        let z1 = cz + BLOCK_SIZE * 0.5;
        let off = self.open_factor * (BLOCK_SIZE * 0.5);
        let (yf, yc) = (self.y_floor, self.y_ceil);
        let tex_l = self.wall_tex.min(max_tex);
        let tex_r = (self.wall_tex + 1).min(max_tex);

        if self.is_blocking_x {
            Triangle3D::append_quad(
                tris,
                Vec3::new(cx, yc, cz - off),
                Vec3::new(cx, yc, z0 - off),
                Vec3::new(cx, yf, z0 - off),
                Vec3::new(cx, yf, cz - off),
                tex_l,
                true,
            );
            Triangle3D::append_quad(
                tris,
                Vec3::new(cx, yc, z1 + off),
                Vec3::new(cx, yc, cz + off),
                Vec3::new(cx, yf, cz + off),
                Vec3::new(cx, yf, z1 + off),
                tex_r,
                true,
            );
        } else {
            Triangle3D::append_quad(
                tris,
                Vec3::new(x0 - off, yc, cz),
                Vec3::new(cx - off, yc, cz),
                Vec3::new(cx - off, yf, cz),
                Vec3::new(x0 - off, yf, cz),
                tex_l,
                true,
            );
            Triangle3D::append_quad(
                tris,
                Vec3::new(cx + off, yc, cz),
                Vec3::new(x1 + off, yc, cz),
                Vec3::new(x1 + off, yf, cz),
                Vec3::new(cx + off, yf, cz),
                tex_r,
                true,
            );
        }
    }
}

impl SceneEntity for FanEntity {
    fn base(&self) -> &MapObject {
        &self.base
    }
    fn base_mut(&mut self) -> &mut MapObject {
        &mut self.base
    }
    fn is_empty_space(&self) -> bool {
        true
    }
    fn hierarchy_label(&self) -> String {
        let st = if self.destroyed {
            "Destruída"
        } else {
            "Ativa"
        };
        format!("Fan ({}, {}) [{st}]", self.gx, self.gz)
    }
    fn update_entity(&mut self, _ship_pos: Vec3, _dt: f32) {}
    fn blocks_movement(&self, lx: f32, lz: f32, for_shot: bool) -> bool {
        if self.destroyed {
            return false;
        }
        let active = if for_shot {
            self.blocks_shot
        } else {
            self.solid
        };
        active && self.contains_local_point(lx, lz)
    }
    fn on_laser_hit(&mut self) -> bool {
        if !self.destroyed {
            self.destroyed = true;
            self.wall_tex = 10;
            return true;
        }
        false
    }
    fn emit_dynamic_geometry(&self, tris: &mut Vec<Triangle3D>, anim_frame: usize, max_tex: usize) {
        let cx = self.gx as f32 * BLOCK_SIZE;
        let cz = self.gz as f32 * BLOCK_SIZE;
        let x0 = cx - BLOCK_SIZE * 0.5;
        let x1 = cx + BLOCK_SIZE * 0.5;
        let z0 = cz - BLOCK_SIZE * 0.5;
        let z1 = cz + BLOCK_SIZE * 0.5;
        let (yf, yc) = (self.y_floor, self.y_ceil);

        let fan_tex = if self.destroyed {
            10.min(max_tex)
        } else if self.wall_tex >= 6 && self.wall_tex <= 9 {
            (6 + anim_frame).min(max_tex)
        } else {
            self.wall_tex.min(max_tex)
        };

        if self.is_blocking_x {
            Triangle3D::append_quad(
                tris,
                Vec3::new(cx, yc, z1),
                Vec3::new(cx, yc, z0),
                Vec3::new(cx, yf, z0),
                Vec3::new(cx, yf, z1),
                fan_tex,
                true,
            );
        } else {
            Triangle3D::append_quad(
                tris,
                Vec3::new(x0, yc, cz),
                Vec3::new(x1, yc, cz),
                Vec3::new(x1, yf, cz),
                Vec3::new(x0, yf, cz),
                fan_tex,
                true,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneNode {
    Block(MapObject),
    Door(DoorEntity),
    Fan(FanEntity),
}

impl SceneNode {
    pub fn from_tile(gx: usize, gz: usize, tile: i32) -> Self {
        let solid = tile != 0;
        let wall_tex = match tile {
            1 => 1,
            2 => 2,
            3 => 3,
            8 | 9 => 4,
            10 | 11 => 6,
            _ => 1,
        };
        let (min_x, min_z, max_x, max_z) = match tile {
            8 | 10 => (0.38, 0.0, 0.62, 1.0),
            9 | 11 => (0.0, 0.38, 1.0, 0.62),
            _ => (0.0, 0.0, 1.0, 1.0),
        };
        let base = MapObject::new(
            gx, gz, tile, wall_tex, 0, 0, -2.0, 2.0, solid, solid, min_x, min_z, max_x, max_z,
        );
        match tile {
            8 => SceneNode::Door(DoorEntity::new(base, 0.0, true)),
            9 => SceneNode::Door(DoorEntity::new(base, 0.0, false)),
            10 => SceneNode::Fan(FanEntity::new(base, false, true)),
            11 => SceneNode::Fan(FanEntity::new(base, false, false)),
            _ => SceneNode::Block(base),
        }
    }

    #[inline]
    pub fn as_entity(&self) -> &dyn SceneEntity {
        match self {
            SceneNode::Block(b) => b,
            SceneNode::Door(d) => d,
            SceneNode::Fan(f) => f,
        }
    }

    #[inline]
    pub fn as_entity_mut(&mut self) -> &mut dyn SceneEntity {
        match self {
            SceneNode::Block(b) => b,
            SceneNode::Door(d) => d,
            SceneNode::Fan(f) => f,
        }
    }
}
