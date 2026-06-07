use crate::export::ExportOptions;
use crate::mesh;
use crate::model::Model;

use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

const GLB_MAGIC: u32 = 0x46546C67;
const GLB_VERSION: u32 = 2;
const JSON_CHUNK_TYPE: u32 = 0x4E4F534A;
const BIN_CHUNK_TYPE: u32 = 0x004E4942;

pub fn export_gltf(model: &Model, path: &str, options: &ExportOptions) -> bool {
    let scene_mesh = mesh::build_scene_mesh(model);

    let path_obj = Path::new(path);
    let parent = path_obj.parent();
    if let Some(dir) = parent {
        if !dir.exists() {
            if fs::create_dir_all(dir).is_err() {
                return false;
            }
        }
    }

    let stem = path_obj.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("scene");

    let bin_filename = format!("{}.bin", stem);

    let json = build_gltf_json(&scene_mesh, &bin_filename, options);
    if json.is_empty() {
        return false;
    }

    if fs::write(path, json.as_bytes()).is_err() {
        return false;
    }

    let bin_path = if let Some(dir) = parent {
        dir.join(&bin_filename)
    } else {
        Path::new(&bin_filename).to_path_buf()
    };

    if write_binary_buffer(&scene_mesh, bin_path.to_str().unwrap_or("scene.bin")).is_err() {
        return false;
    }

    true
}

pub fn export_glb(model: &Model, path: &str, options: &ExportOptions) -> bool {
    let scene_mesh = mesh::build_scene_mesh(model);

    let path_obj = Path::new(path);
    if let Some(dir) = path_obj.parent() {
        if !dir.exists() {
            if fs::create_dir_all(dir).is_err() {
                return false;
            }
        }
    }

    let bin_filename = "buffer.bin";
    let json_str = build_gltf_json(&scene_mesh, bin_filename, options);

    let mut json_bytes = json_str.into_bytes();
    pad_to_4_bytes(&mut json_bytes);

    let mut bin_data = build_binary_buffer_bytes(&scene_mesh);
    pad_to_4_bytes(&mut bin_data);

    let total_length: u32 = 12 + 8 + (json_bytes.len() as u32) + 8 + (bin_data.len() as u32);

    let mut glb = Vec::with_capacity(total_length as usize);
    glb.write_u32::<LittleEndian>(GLB_MAGIC).unwrap();
    glb.write_u32::<LittleEndian>(GLB_VERSION).unwrap();
    glb.write_u32::<LittleEndian>(total_length).unwrap();

    glb.write_u32::<LittleEndian>(json_bytes.len() as u32).unwrap();
    glb.write_u32::<LittleEndian>(JSON_CHUNK_TYPE).unwrap();
    glb.extend_from_slice(&json_bytes);

    glb.write_u32::<LittleEndian>(bin_data.len() as u32).unwrap();
    glb.write_u32::<LittleEndian>(BIN_CHUNK_TYPE).unwrap();
    glb.extend_from_slice(&bin_data);

    fs::write(path, glb).is_ok()
}

fn build_gltf_json(scene_mesh: &mesh::MeshData, bin_uri: &str, _options: &ExportOptions) -> String {
    let vertex_count = scene_mesh.vertices.len() / 3;
    let normal_count = scene_mesh.normals.len() / 3;
    let uv_count = scene_mesh.uvs.len() / 2;
    let index_count = scene_mesh.indices.len();

    let vertex_byte_length = scene_mesh.vertices.len() * 4;
    let normal_byte_length = scene_mesh.normals.len() * 4;
    let uv_byte_length = scene_mesh.uvs.len() * 4;
    let index_byte_length = scene_mesh.indices.len() * 4;
    let total_buffer_length = vertex_byte_length + normal_byte_length + uv_byte_length + index_byte_length;

    let (pos_min, pos_max) = compute_bounds(&scene_mesh.vertices, 3);

    let mut json = String::new();
    json.push_str("{");

    json.push_str("\"asset\":{");
    json.push_str("\"version\":\"2.0\",");
    json.push_str("\"generator\":\"3D Builder MC Rust Engine\"");
    json.push_str("},");

    json.push_str("\"scene\":0,");

    json.push_str("\"scenes\":[{");
    json.push_str("\"name\":\"Scene\",");
    json.push_str("\"nodes\":[0]");
    json.push_str("}],");

    json.push_str("\"nodes\":[{");
    json.push_str("\"name\":\"RootNode\",");
    json.push_str("\"mesh\":0");
    json.push_str("}],");

    json.push_str("\"meshes\":[{");
    json.push_str("\"name\":\"SceneMesh\",");
    json.push_str("\"primitives\":[{");
    json.push_str("\"attributes\":{");
    json.push_str("\"POSITION\":0,");
    json.push_str("\"NORMAL\":1,");
    json.push_str("\"TEXCOORD_0\":2");
    json.push_str("},");
    json.push_str("\"indices\":3,");
    json.push_str("\"mode\":4");
    json.push_str("}]");
    json.push_str("}],");

    json.push_str("\"accessors\":[");
    json.push_str("{");
    json.push_str(&format!("\"bufferView\":0,\"componentType\":5126,\"count\":{},\"type\":\"VEC3\",", vertex_count));
    json.push_str(&format!("\"max\":[{},{},{}],\"min\":[{},{},{}]", pos_max[0], pos_max[1], pos_max[2], pos_min[0], pos_min[1], pos_min[2]));
    json.push_str("},");
    json.push_str("{");
    json.push_str(&format!("\"bufferView\":1,\"componentType\":5126,\"count\":{},\"type\":\"VEC3\"", normal_count));
    json.push_str("},");
    json.push_str("{");
    json.push_str(&format!("\"bufferView\":2,\"componentType\":5126,\"count\":{},\"type\":\"VEC2\"", uv_count));
    json.push_str("},");
    json.push_str("{");
    json.push_str(&format!("\"bufferView\":3,\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"", index_count));
    json.push_str("}");
    json.push_str("],");

    json.push_str("\"bufferViews\":[");
    json.push_str(&format!("{{\"buffer\":0,\"byteOffset\":0,\"byteLength\":{},\"target\":34962}},", vertex_byte_length));
    json.push_str(&format!("{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34962}},", vertex_byte_length, normal_byte_length));
    json.push_str(&format!("{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34962}},", vertex_byte_length + normal_byte_length, uv_byte_length));
    json.push_str(&format!("{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34963}}", vertex_byte_length + normal_byte_length + uv_byte_length, index_byte_length));
    json.push_str("],");

    json.push_str(&format!("\"buffers\":[{{\"uri\":\"{}\",\"byteLength\":{}}}]", bin_uri, total_buffer_length));

    json.push_str("}");

    json
}

fn write_binary_buffer(scene_mesh: &mesh::MeshData, path: &str) -> Result<(), std::io::Error> {
    let data = build_binary_buffer_bytes(scene_mesh);
    fs::write(path, data)
}

fn build_binary_buffer_bytes(scene_mesh: &mesh::MeshData) -> Vec<u8> {
    let vertex_byte_length = scene_mesh.vertices.len() * 4;
    let normal_byte_length = scene_mesh.normals.len() * 4;
    let uv_byte_length = scene_mesh.uvs.len() * 4;
    let index_byte_length = scene_mesh.indices.len() * 4;
    let total = vertex_byte_length + normal_byte_length + uv_byte_length + index_byte_length;

    let mut buffer = Vec::with_capacity(total);

    for v in &scene_mesh.vertices {
        buffer.write_f32::<LittleEndian>(*v).unwrap();
    }
    for n in &scene_mesh.normals {
        buffer.write_f32::<LittleEndian>(*n).unwrap();
    }
    for uv in &scene_mesh.uvs {
        buffer.write_f32::<LittleEndian>(*uv).unwrap();
    }
    for idx in &scene_mesh.indices {
        buffer.write_u32::<LittleEndian>(*idx as u32).unwrap();
    }

    buffer
}

fn compute_bounds(vertices: &[f32], stride: usize) -> (Vec<f32>, Vec<f32>) {
    let mut mins = vec![f32::MAX; stride];
    let mut maxs = vec![f32::MIN; stride];

    for chunk in vertices.chunks(stride) {
        for (i, &val) in chunk.iter().enumerate() {
            if val < mins[i] {
                mins[i] = val;
            }
            if val > maxs[i] {
                maxs[i] = val;
            }
        }
    }

    if vertices.is_empty() {
        mins = vec![0.0; stride];
        maxs = vec![0.0; stride];
    }

    (mins, maxs)
}

fn pad_to_4_bytes(data: &mut Vec<u8>) {
    let padding = (4 - (data.len() % 4)) % 4;
    for _ in 0..padding {
        data.push(0x20);
    }
}
