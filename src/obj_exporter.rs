use crate::export::ExportOptions;
use crate::mesh;
use crate::model::Model;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn export_obj(model: &Model, path: &str, options: &ExportOptions) -> bool {
    let path_obj = Path::new(path);
    if let Some(dir) = path_obj.parent() {
        if !dir.exists() {
            if fs::create_dir_all(dir).is_err() {
                return false;
            }
        }
    }

    let stem = path_obj.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("scene");

    let mtl_filename = format!("{}.mtl", stem);
    let mtl_path = path_obj.with_extension("mtl");

    let mut obj_content = String::new();
    obj_content.push_str("# 3D Builder MC OBJ Export (Rust Engine)\n");
    obj_content.push_str("# https://dbuild.net\n\n");
    obj_content.push_str(&format!("mtllib {}\n\n", mtl_filename));

    let blocks = model.get_all_blocks();
    let mut vertex_offset: u32 = 1;
    let mut normal_offset: u32 = 1;
    let mut uv_offset: u32 = 1;

    let mut textures_used: HashSet<String> = HashSet::new();

    for (block_idx, block) in blocks.iter().enumerate() {
        let block_mesh = mesh::generate_cube_mesh(block);

        obj_content.push_str(&format!("o Block_{}\n", block_idx));

        for chunk in block_mesh.vertices.chunks(3) {
            obj_content.push_str(&format_vertex(chunk[0], chunk[1], chunk[2]));
        }

        for chunk in block_mesh.uvs.chunks(2) {
            obj_content.push_str(&format_uv(chunk[0], chunk[1]));
        }

        for chunk in block_mesh.normals.chunks(3) {
            obj_content.push_str(&format_normal(chunk[0], chunk[1], chunk[2]));
        }

        let material_name = if let Some(tex) = block.face_textures.get(0).and_then(|t| t.as_ref()) {
            textures_used.insert(tex.clone());
            sanitize_material_name(tex)
        } else {
            "DefaultMaterial".to_string()
        };

        obj_content.push_str(&format!("usemtl {}\n", material_name));

        for chunk in block_mesh.indices.chunks(3) {
            let v1 = chunk[0] as u32 + vertex_offset;
            let v2 = chunk[1] as u32 + vertex_offset;
            let v3 = chunk[2] as u32 + vertex_offset;
            let vt1 = chunk[0] as u32 + uv_offset;
            let vt2 = chunk[1] as u32 + uv_offset;
            let vt3 = chunk[2] as u32 + uv_offset;
            let vn1 = chunk[0] as u32 + normal_offset;
            let vn2 = chunk[1] as u32 + normal_offset;
            let vn3 = chunk[2] as u32 + normal_offset;

            obj_content.push_str(&format!(
                "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                v1, vt1, vn1, v2, vt2, vn2, v3, vt3, vn3
            ));
        }

        obj_content.push('\n');

        vertex_offset += (block_mesh.vertices.len() / 3) as u32;
        normal_offset += (block_mesh.normals.len() / 3) as u32;
        uv_offset += (block_mesh.uvs.len() / 2) as u32;
    }

    if fs::write(path, obj_content.as_bytes()).is_err() {
        return false;
    }

    let mut mtl_content = String::new();
    mtl_content.push_str("# 3D Builder MC MTL Export (Rust Engine)\n\n");

    if textures_used.is_empty() {
        mtl_content.push_str("newmtl DefaultMaterial\n");
        mtl_content.push_str("Ka 0.2 0.2 0.2\n");
        mtl_content.push_str("Kd 0.8 0.8 0.8\n");
        mtl_content.push_str("Ks 0.0 0.0 0.0\n");
        mtl_content.push_str("Ns 1.0\n");
        mtl_content.push_str("d 1.0\n");
        mtl_content.push_str("illum 2\n\n");
    } else {
        for tex in &textures_used {
            let mat_name = sanitize_material_name(tex);
            mtl_content.push_str(&format!("newmtl {}\n", mat_name));
            mtl_content.push_str("Ka 0.2 0.2 0.2\n");
            mtl_content.push_str("Kd 0.8 0.8 0.8\n");
            mtl_content.push_str("Ks 0.0 0.0 0.0\n");
            mtl_content.push_str("Ns 1.0\n");
            mtl_content.push_str("d 1.0\n");
            mtl_content.push_str("illum 2\n");
            mtl_content.push_str(&format!("map_Kd textures/{}\n\n", tex));
        }

        mtl_content.push_str("newmtl DefaultMaterial\n");
        mtl_content.push_str("Ka 0.2 0.2 0.2\n");
        mtl_content.push_str("Kd 0.8 0.8 0.8\n");
        mtl_content.push_str("Ks 0.0 0.0 0.0\n");
        mtl_content.push_str("Ns 1.0\n");
        mtl_content.push_str("d 1.0\n");
        mtl_content.push_str("illum 2\n\n");
    }

    if fs::write(mtl_path, mtl_content.as_bytes()).is_err() {
        return false;
    }

    if options.include_textures {
        if let Some(parent) = path_obj.parent() {
            let tex_dir = parent.join("textures");
            if !tex_dir.exists() {
                let _ = fs::create_dir_all(&tex_dir);
            }
        }
    }

    true
}

fn format_vertex(x: f32, y: f32, z: f32) -> String {
    format!("v {:.6} {:.6} {:.6}\n", x, y, z)
}

fn format_normal(x: f32, y: f32, z: f32) -> String {
    format!("vn {:.6} {:.6} {:.6}\n", x, y, z)
}

fn format_uv(u: f32, v: f32) -> String {
    format!("vt {:.6} {:.6}\n", u, v)
}

fn sanitize_material_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        "DefaultMaterial".to_string()
    } else {
        sanitized
    }
}
