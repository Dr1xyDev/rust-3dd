use crate::model::{Block, Model, UVMapping};

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
    pub indices: Vec<i32>,
}

impl MeshData {
    pub fn new() -> Self {
        MeshData {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

pub fn generate_cube_mesh(block: &Block) -> MeshData {
    let mut mesh = MeshData::new();

    let x = block.x as f32;
    let y = block.y as f32;
    let z = block.z as f32;
    let s = 1.0f32;

    let face_verts: [[f32; 12]; 6] = [
        // Front (+Z)
        [x, y, z + s, x + s, y, z + s, x + s, y + s, z + s, x, y + s, z + s],
        // Back (-Z)
        [x + s, y, z, x, y, z, x, y + s, z, x + s, y + s, z],
        // Top (+Y)
        [x, y + s, z + s, x + s, y + s, z + s, x + s, y + s, z, x, y + s, z],
        // Bottom (-Y)
        [x, y, z, x + s, y, z, x + s, y, z + s, x, y, z + s],
        // Right (+X)
        [x + s, y, z + s, x + s, y, z, x + s, y + s, z, x + s, y + s, z + s],
        // Left (-X)
        [x, y, z, x, y, z + s, x, y + s, z + s, x, y + s, z],
    ];

    let face_normals: [[f32; 12]; 6] = [
        [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0],
        [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0],
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    ];

    for face_idx in 0..6 {
        let uv_map = &block.face_uvs[face_idx];

        let face_uvs = [
            uv_map.u, uv_map.v,
            uv_map.u + uv_map.u_scale, uv_map.v,
            uv_map.u + uv_map.u_scale, uv_map.v + uv_map.v_scale,
            uv_map.u, uv_map.v + uv_map.v_scale,
        ];

        mesh.vertices.extend_from_slice(&face_verts[face_idx]);
        mesh.normals.extend_from_slice(&face_normals[face_idx]);
        mesh.uvs.extend_from_slice(&face_uvs);

        let base = (mesh.vertex_count() - 4) as i32;
        mesh.indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);
    }

    mesh
}

pub fn merge_meshes(meshes: Vec<MeshData>) -> MeshData {
    let mut merged = MeshData::new();
    let mut vertex_offset: i32 = 0;

    for mesh in meshes {
        merged.vertices.extend_from_slice(&mesh.vertices);
        merged.normals.extend_from_slice(&mesh.normals);
        merged.uvs.extend_from_slice(&mesh.uvs);

        for idx in &mesh.indices {
            merged.indices.push(idx + vertex_offset);
        }

        vertex_offset += mesh.vertex_count() as i32;
    }

    merged
}

pub fn build_scene_mesh(model: &Model) -> MeshData {
    let mut meshes = Vec::new();

    let blocks: Vec<&Block> = model.get_all_blocks();
    let block_positions: std::collections::HashSet<(i32, i32, i32)> =
        model.blocks.keys().cloned().collect();

    for block in &blocks {
        let mut mesh = MeshData::new();

        let x = block.x as f32;
        let y = block.y as f32;
        let z = block.z as f32;
        let s = 1.0f32;

        let face_verts: [[f32; 12]; 6] = [
            [x, y, z + s, x + s, y, z + s, x + s, y + s, z + s, x, y + s, z + s],
            [x + s, y, z, x, y, z, x, y + s, z, x + s, y + s, z],
            [x, y + s, z + s, x + s, y + s, z + s, x + s, y + s, z, x, y + s, z],
            [x, y, z, x + s, y, z, x + s, y, z + s, x, y, z + s],
            [x + s, y, z + s, x + s, y, z, x + s, y + s, z, x + s, y + s, z + s],
            [x, y, z, x, y, z + s, x, y + s, z + s, x, y + s, z],
        ];

        let face_normals: [[f32; 12]; 6] = [
            [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
        ];

        let adjacent_offsets: [(i32, i32, i32); 6] = [
            (0, 0, 1),   // front
            (0, 0, -1),  // back
            (0, 1, 0),   // top
            (0, -1, 0),  // bottom
            (1, 0, 0),   // right
            (-1, 0, 0),  // left
        ];

        for face_idx in 0..6 {
            let (dx, dy, dz) = adjacent_offsets[face_idx];
            let neighbor_pos = (x as i32 + dx, y as i32 + dy, z as i32 + dz);

            if block_positions.contains(&neighbor_pos) {
                continue;
            }

            let uv_map = &block.face_uvs[face_idx];
            let face_uvs = [
                uv_map.u, uv_map.v,
                uv_map.u + uv_map.u_scale, uv_map.v,
                uv_map.u + uv_map.u_scale, uv_map.v + uv_map.v_scale,
                uv_map.u, uv_map.v + uv_map.v_scale,
            ];

            mesh.vertices.extend_from_slice(&face_verts[face_idx]);
            mesh.normals.extend_from_slice(&face_normals[face_idx]);
            mesh.uvs.extend_from_slice(&face_uvs);

            let base = (mesh.vertex_count() - 4) as i32;
            mesh.indices.extend_from_slice(&[
                base, base + 1, base + 2,
                base, base + 2, base + 3,
            ]);
        }

        if !mesh.is_empty() {
            meshes.push(mesh);
        }
    }

    merge_meshes(meshes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cube_mesh() {
        let block = Block::new(0, 0, 0);
        let mesh = generate_cube_mesh(&block);
        assert_eq!(mesh.vertices.len(), 72);
        assert_eq!(mesh.normals.len(), 72);
        assert_eq!(mesh.uvs.len(), 48);
        assert_eq!(mesh.indices.len(), 36);
    }

    #[test]
    fn test_merge_meshes() {
        let block1 = Block::new(0, 0, 0);
        let block2 = Block::new(1, 0, 0);
        let mesh1 = generate_cube_mesh(&block1);
        let mesh2 = generate_cube_mesh(&block2);
        let merged = merge_meshes(vec![mesh1, mesh2]);
        assert_eq!(merged.vertices.len(), 144);
        assert_eq!(merged.indices.len(), 72);
    }

    #[test]
    fn test_build_scene_mesh_culling() {
        let mut model = Model::new();
        model.add_block(0, 0, 0);
        model.add_block(1, 0, 0);
        let mesh = build_scene_mesh(&model);
        assert!(mesh.vertices.len() < 144);
    }
}
