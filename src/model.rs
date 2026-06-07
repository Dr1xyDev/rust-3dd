use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UVMapping {
    pub u: f32,
    pub v: f32,
    pub u_scale: f32,
    pub v_scale: f32,
}

impl Default for UVMapping {
    fn default() -> Self {
        UVMapping {
            u: 0.0,
            v: 0.0,
            u_scale: 1.0,
            v_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(default)]
    pub colors: Vec<f32>,
    #[serde(default)]
    pub face_textures: Vec<Option<String>>,
    #[serde(default)]
    pub face_uvs: Vec<UVMapping>,
}

impl Block {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let mut face_textures = Vec::with_capacity(6);
        let mut face_uvs = Vec::with_capacity(6);
        for _ in 0..6 {
            face_textures.push(None);
            face_uvs.push(UVMapping::default());
        }
        Block {
            x,
            y,
            z,
            colors: vec![1.0, 1.0, 1.0, 1.0],
            face_textures,
            face_uvs,
        }
    }

    pub fn set_face_texture(&mut self, face: usize, texture: &str) {
        if face < 6 {
            self.face_textures[face] = Some(texture.to_string());
        }
    }

    pub fn set_face_uv(&mut self, face: usize, u: f32, v: f32, u_scale: f32, v_scale: f32) {
        if face < 6 {
            self.face_uvs[face] = UVMapping {
                u,
                v,
                u_scale,
                v_scale,
            };
        }
    }

    #[allow(dead_code)]
    pub fn get_face_texture(&self, face: usize) -> Option<&str> {
        if face < 6 {
            self.face_textures[face].as_deref()
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_face_uv(&self, face: usize) -> &UVMapping {
        if face < 6 {
            &self.face_uvs[face]
        } else {
            static DEFAULT: UVMapping = UVMapping {
                u: 0.0,
                v: 0.0,
                u_scale: 1.0,
                v_scale: 1.0,
            };
            &DEFAULT
        }
    }

    pub fn key(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub blocks: HashMap<(i32, i32, i32), Block>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            blocks: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, x: i32, y: i32, z: i32) {
        let block = Block::new(x, y, z);
        self.blocks.insert(block.key(), block);
    }

    pub fn remove_block(&mut self, x: i32, y: i32, z: i32) {
        self.blocks.remove(&(x, y, z));
    }

    pub fn set_block_texture(&mut self, x: i32, y: i32, z: i32, face: usize, texture: &str) {
        if let Some(block) = self.blocks.get_mut(&(x, y, z)) {
            block.set_face_texture(face, texture);
        }
    }

    pub fn set_block_uv(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        face: usize,
        u: f32,
        v: f32,
        u_scale: f32,
        v_scale: f32,
    ) {
        if let Some(block) = self.blocks.get_mut(&(x, y, z)) {
            block.set_face_uv(face, u, v, u_scale, v_scale);
        }
    }

    pub fn get_block_count(&self) -> usize {
        self.blocks.len()
    }

    #[allow(dead_code)]
    pub fn has_block_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.blocks.contains_key(&(x, y, z))
    }

    #[allow(dead_code)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&Block> {
        self.blocks.get(&(x, y, z))
    }

    #[allow(dead_code)]
    pub fn get_block_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Block> {
        self.blocks.get_mut(&(x, y, z))
    }

    pub fn get_all_blocks(&self) -> Vec<&Block> {
        self.blocks.values().collect()
    }

    pub fn get_all_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        for block in self.blocks.values() {
            let cube_verts = generate_cube_vertices(block);
            vertices.extend_from_slice(&cube_verts);
        }
        vertices
    }

    pub fn get_all_normals(&self) -> Vec<f32> {
        let mut normals = Vec::new();
        for _ in self.blocks.values() {
            let cube_norms = generate_cube_normals();
            normals.extend_from_slice(&cube_norms);
        }
        normals
    }

    pub fn get_all_uvs(&self) -> Vec<f32> {
        let mut uvs = Vec::new();
        for block in self.blocks.values() {
            let cube_uvs = generate_cube_uvs(block);
            uvs.extend_from_slice(&cube_uvs);
        }
        uvs
    }

    pub fn get_all_indices(&self) -> Vec<i32> {
        let mut indices = Vec::new();
        let mut offset: i32 = 0;
        for _block in self.blocks.values() {
            let cube_indices = generate_cube_indices(offset);
            indices.extend_from_slice(&cube_indices);
            offset += 24;
        }
        indices
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

pub struct Engine {
    pub models: HashMap<u64, Model>,
    next_model_id: u64,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            models: HashMap::new(),
            next_model_id: 1,
        }
    }

    pub fn create_model(&mut self) -> u64 {
        let id = self.next_model_id;
        self.next_model_id += 1;
        self.models.insert(id, Model::new());
        id
    }

    pub fn destroy_model(&mut self, id: u64) -> bool {
        self.models.remove(&id).is_some()
    }

    pub fn get_model(&self, id: u64) -> Option<&Model> {
        self.models.get(&id)
    }

    pub fn get_model_mut(&mut self, id: u64) -> Option<&mut Model> {
        self.models.get_mut(&id)
    }
}

fn generate_cube_vertices(block: &Block) -> [f32; 72] {
    let x = block.x as f32;
    let y = block.y as f32;
    let z = block.z as f32;
    let s = 1.0f32;

    [
        // Front face
        x, y, z + s,       x + s, y, z + s,       x + s, y + s, z + s,       x, y + s, z + s,
        // Back face
        x + s, y, z,       x, y, z,               x, y + s, z,               x + s, y + s, z,
        // Top face
        x, y + s, z + s,   x + s, y + s, z + s,   x + s, y + s, z,           x, y + s, z,
        // Bottom face
        x, y, z,           x + s, y, z,           x + s, y, z + s,           x, y, z + s,
        // Right face
        x + s, y, z + s,   x + s, y, z,           x + s, y + s, z,           x + s, y + s, z + s,
        // Left face
        x, y, z,           x, y, z + s,           x, y + s, z + s,           x, y + s, z,
    ]
}

fn generate_cube_normals() -> [f32; 72] {
    [
        0.0, 0.0, 1.0,    0.0, 0.0, 1.0,    0.0, 0.0, 1.0,    0.0, 0.0, 1.0,
        0.0, 0.0, -1.0,   0.0, 0.0, -1.0,   0.0, 0.0, -1.0,   0.0, 0.0, -1.0,
        0.0, 1.0, 0.0,    0.0, 1.0, 0.0,    0.0, 1.0, 0.0,    0.0, 1.0, 0.0,
        0.0, -1.0, 0.0,   0.0, -1.0, 0.0,   0.0, -1.0, 0.0,   0.0, -1.0, 0.0,
        1.0, 0.0, 0.0,    1.0, 0.0, 0.0,    1.0, 0.0, 0.0,    1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0,   -1.0, 0.0, 0.0,   -1.0, 0.0, 0.0,   -1.0, 0.0, 0.0,
    ]
}

fn generate_cube_uvs(block: &Block) -> [f32; 48] {
    let mut uvs = [0.0f32; 48];
    for face in 0..6 {
        let uv_map = &block.face_uvs[face];
        let base = face * 8;
        uvs[base] = uv_map.u;
        uvs[base + 1] = uv_map.v;
        uvs[base + 2] = uv_map.u + uv_map.u_scale;
        uvs[base + 3] = uv_map.v;
        uvs[base + 4] = uv_map.u + uv_map.u_scale;
        uvs[base + 5] = uv_map.v + uv_map.v_scale;
        uvs[base + 6] = uv_map.u;
        uvs[base + 7] = uv_map.v + uv_map.v_scale;
    }
    uvs
}

fn generate_cube_indices(offset: i32) -> [i32; 36] {
    let o = offset;
    [
        o, o + 1, o + 2,     o, o + 2, o + 3,
        o + 4, o + 5, o + 6, o + 4, o + 6, o + 7,
        o + 8, o + 9, o + 10, o + 8, o + 10, o + 11,
        o + 12, o + 13, o + 14, o + 12, o + 14, o + 15,
        o + 16, o + 17, o + 18, o + 16, o + 18, o + 19,
        o + 20, o + 21, o + 22, o + 20, o + 22, o + 23,
    ]
}
