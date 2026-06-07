use crate::export::ExportOptions;
use crate::model::{Block, Model};
use crate::serializer;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;

const M3X_MAGIC: u32 = 0x4D335820;
const M3X_VERSION: u32 = 1;

const SECTION_SCENE: u32 = 1;
const SECTION_BLOCKS: u32 = 2;
const SECTION_TEXTURES: u32 = 3;
const SECTION_UV: u32 = 4;
const SECTION_CAMERA: u32 = 5;
const SECTION_SETTINGS: u32 = 6;

struct IndexEntry {
    section_type: u32,
    offset: u64,
    compressed_size: u32,
    uncompressed_size: u32,
}

pub fn export_m3x(model: &Model, path: &str, _options: &ExportOptions) -> bool {
    let path_obj = Path::new(path);
    if let Some(dir) = path_obj.parent() {
        if !dir.exists() {
            if fs::create_dir_all(dir).is_err() {
                return false;
            }
        }
    }

    let mut sections: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut index_entries: Vec<IndexEntry> = Vec::new();

    let scene_data = build_scene_section(model);
    let compressed_scene = serializer::compress_data(&scene_data);
    sections.push((SECTION_SCENE, compressed_scene));
    index_entries.push(IndexEntry {
        section_type: SECTION_SCENE,
        offset: 0,
        compressed_size: sections.last().unwrap().1.len() as u32,
        uncompressed_size: scene_data.len() as u32,
    });

    let block_data = build_block_section(model);
    let compressed_blocks = serializer::compress_data(&block_data);
    sections.push((SECTION_BLOCKS, compressed_blocks));
    index_entries.push(IndexEntry {
        section_type: SECTION_BLOCKS,
        offset: 0,
        compressed_size: sections.last().unwrap().1.len() as u32,
        uncompressed_size: block_data.len() as u32,
    });

    let texture_data = build_texture_section(model);
    if texture_data.is_empty() {
        sections.push((SECTION_TEXTURES, Vec::new()));
        index_entries.push(IndexEntry {
            section_type: SECTION_TEXTURES,
            offset: 0,
            compressed_size: 0,
            uncompressed_size: 0,
        });
    } else {
        let compressed_textures = serializer::compress_data(&texture_data);
        sections.push((SECTION_TEXTURES, compressed_textures));
        index_entries.push(IndexEntry {
            section_type: SECTION_TEXTURES,
            offset: 0,
            compressed_size: sections.last().unwrap().1.len() as u32,
            uncompressed_size: texture_data.len() as u32,
        });
    }

    let uv_data = build_uv_section(model);
    let compressed_uv = serializer::compress_data(&uv_data);
    sections.push((SECTION_UV, compressed_uv));
    index_entries.push(IndexEntry {
        section_type: SECTION_UV,
        offset: 0,
        compressed_size: sections.last().unwrap().1.len() as u32,
        uncompressed_size: uv_data.len() as u32,
    });

    let camera_data = build_camera_section();
    let compressed_camera = serializer::compress_data(&camera_data);
    sections.push((SECTION_CAMERA, compressed_camera));
    index_entries.push(IndexEntry {
        section_type: SECTION_CAMERA,
        offset: 0,
        compressed_size: sections.last().unwrap().1.len() as u32,
        uncompressed_size: camera_data.len() as u32,
    });

    let settings_data = build_settings_section();
    let compressed_settings = serializer::compress_data(&settings_data);
    sections.push((SECTION_SETTINGS, compressed_settings));
    index_entries.push(IndexEntry {
        section_type: SECTION_SETTINGS,
        offset: 0,
        compressed_size: sections.last().unwrap().1.len() as u32,
        uncompressed_size: settings_data.len() as u32,
    });

    let header_size: u64 = 16;
    let mut section_data_size: u64 = 0;
    for (_, data) in &sections {
        section_data_size += 12 + data.len() as u64;
    }
    let index_table_size: u64 = 4 + (index_entries.len() as u64 * 20);
    let checksum_size: u64 = 8;
    let total_size = header_size + section_data_size + index_table_size + checksum_size;

    let mut buffer = Vec::with_capacity(total_size as usize);

    buffer.write_u32::<LittleEndian>(M3X_MAGIC).unwrap();
    buffer.write_u32::<LittleEndian>(M3X_VERSION).unwrap();
    buffer.write_u32::<LittleEndian>(0u32).unwrap();
    buffer.write_u32::<LittleEndian>((header_size + section_data_size) as u32).unwrap();

    let mut current_offset = header_size;
    for (i, (section_type, data)) in sections.iter().enumerate() {
        index_entries[i].offset = current_offset;

        buffer.write_u32::<LittleEndian>(*section_type).unwrap();
        buffer.write_u32::<LittleEndian>(data.len() as u32).unwrap();
        buffer.write_u32::<LittleEndian>(index_entries[i].uncompressed_size).unwrap();
        buffer.extend_from_slice(data);

        current_offset += 12 + data.len() as u64;
    }

    buffer.write_u32::<LittleEndian>(index_entries.len() as u32).unwrap();
    for entry in &index_entries {
        buffer.write_u32::<LittleEndian>(entry.section_type).unwrap();
        buffer.write_u64::<LittleEndian>(entry.offset).unwrap();
        buffer.write_u32::<LittleEndian>(entry.compressed_size).unwrap();
        buffer.write_u32::<LittleEndian>(entry.uncompressed_size).unwrap();
    }

    let checksum = serializer::compute_crc32(&buffer);
    buffer.write_u32::<LittleEndian>(checksum).unwrap();
    buffer.write_u32::<LittleEndian>(0u32).unwrap();

    fs::write(path, buffer).is_ok()
}

pub fn import_m3x(path: &str, model: &mut Model) -> bool {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if data.len() < 16 {
        return false;
    }

    let mut cursor = Cursor::new(&data);
    let magic = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    if magic != M3X_MAGIC {
        return false;
    }

    let version = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    if version != M3X_VERSION {
        return false;
    }

    let _flags = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let offset_table_pos = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    if offset_table_pos as usize >= data.len() {
        return false;
    }

    cursor.set_position(offset_table_pos as u64);
    let entry_count = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    for _ in 0..entry_count {
        let section_type = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let offset = cursor.read_u64::<LittleEndian>().unwrap_or(0);
        let compressed_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let uncompressed_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        if section_type == SECTION_BLOCKS && offset as usize + 12 <= data.len() {
            let section_start = offset as usize;
            let mut section_cursor = Cursor::new(&data[section_start..]);

            let _st = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);
            let _cs = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);
            let _us = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);

            if compressed_size as usize <= data.len() - section_start - 12 {
                let compressed_data_start = section_start + 12;
                let compressed_data_end = compressed_data_start + compressed_size as usize;
                if compressed_data_end <= data.len() {
                    let compressed_data = &data[compressed_data_start..compressed_data_end];
                    if let Some(decompressed) = serializer::decompress_data(compressed_data) {
                        if let Ok(json_str) = String::from_utf8(decompressed) {
                            parse_block_section(&json_str, model);
                        }
                    }
                }
            }
        }

        if section_type == SECTION_SCENE && offset as usize + 12 <= data.len() {
            let section_start = offset as usize;
            let mut section_cursor = Cursor::new(&data[section_start..]);

            let _st = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);
            let _cs = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);
            let _us = section_cursor.read_u32::<LittleEndian>().unwrap_or(0);

            if compressed_size as usize <= data.len() - section_start - 12 {
                let compressed_data_start = section_start + 12;
                let compressed_data_end = compressed_data_start + compressed_size as usize;
                if compressed_data_end <= data.len() {
                    let compressed_data = &data[compressed_data_start..compressed_data_end];
                    if let Some(decompressed) = serializer::decompress_data(compressed_data) {
                        // Scene section parsed for metadata, no model changes needed
                        let _ = String::from_utf8(decompressed);
                    }
                }
            }
        }
    }

    true
}

pub fn validate_m3x(path: &str) -> bool {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if data.len() < 16 {
        return false;
    }

    let mut cursor = Cursor::new(&data);
    let magic = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    if magic != M3X_MAGIC {
        return false;
    }

    let version = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    if version != M3X_VERSION {
        return false;
    }

    true
}

fn build_scene_section(model: &Model) -> Vec<u8> {
    let mut json = String::new();
    json.push_str("{");
    json.push_str(&format!("\"name\":\"Scene\",\"blockCount\":{},\"materialCount\":0", model.get_block_count()));
    json.push_str("}");
    json.into_bytes()
}

fn build_block_section(model: &Model) -> Vec<u8> {
    let mut json = String::new();
    json.push('[');

    let blocks: Vec<&Block> = model.get_all_blocks();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push('{');
        json.push_str(&format!("\"x\":{},\"y\":{},\"z\":{}", block.x, block.y, block.z));

        json.push_str(",\"colors\":[");
        for (j, c) in block.colors.iter().enumerate() {
            if j > 0 {
                json.push(',');
            }
            json.push_str(&format!("{}", c));
        }
        json.push(']');

        let texture_name = block.face_textures.get(0)
            .as_ref()
            .and_then(|t| t.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");
        json.push_str(&format!(",\"texture\":\"{}\"", texture_name));

        json.push('}');
    }

    json.push(']');
    json.into_bytes()
}

fn build_texture_section(model: &Model) -> Vec<u8> {
    let mut textures = Vec::new();
    for block in model.blocks.values() {
        for face_tex in &block.face_textures {
            if let Some(tex) = face_tex {
                if !textures.contains(tex) {
                    textures.push(tex.clone());
                }
            }
        }
    }

    if textures.is_empty() {
        return Vec::new();
    }

    let mut json = String::new();
    json.push_str("{\"textures\":[");
    for (i, tex) in textures.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!("{{\"name\":\"{}\",\"size\":0,\"data\":\"\"}}", tex));
    }
    json.push_str("]}");
    json.into_bytes()
}

fn build_uv_section(model: &Model) -> Vec<u8> {
    let mut json = String::new();
    json.push('[');

    let blocks: Vec<&Block> = model.get_all_blocks();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push('{');
        json.push_str(&format!("\"x\":{},\"y\":{},\"z\":{}", block.x, block.y, block.z));
        json.push_str(",\"faceUVs\":[");
        for (f, uv) in block.face_uvs.iter().enumerate() {
            if f > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"u\":{},\"v\":{},\"uScale\":{},\"vScale\":{}}}",
                uv.u, uv.v, uv.u_scale, uv.v_scale
            ));
        }
        json.push(']');
        json.push('}');
    }

    json.push(']');
    json.into_bytes()
}

fn build_camera_section() -> Vec<u8> {
    let json = r#"{"positionX":0.0,"positionY":5.0,"positionZ":10.0,"rotationX":-30.0,"rotationY":0.0,"fov":60.0,"near":0.1,"far":1000.0}"#;
    json.as_bytes().to_vec()
}

fn build_settings_section() -> Vec<u8> {
    let json = r#"{"gridEnabled":true,"gridSize":16,"snapEnabled":true,"renderMode":"solid","showAxes":true}"#;
    json.as_bytes().to_vec()
}

fn parse_block_section(json_str: &str, model: &mut Model) {
    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    let blocks_array = match parsed.as_array() {
        Some(arr) => arr,
        None => return,
    };

    model.clear();

    for block_val in blocks_array {
        let obj = match block_val.as_object() {
            Some(o) => o,
            None => continue,
        };

        let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let z = obj.get("z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        model.add_block(x, y, z);

        if let Some(texture_val) = obj.get("texture").and_then(|v| v.as_str()) {
            if !texture_val.is_empty() {
                model.set_block_texture(x, y, z, 0, texture_val);
            }
        }
    }
}
