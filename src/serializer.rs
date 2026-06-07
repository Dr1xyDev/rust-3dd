use crate::model::Model;
use std::fs;
use std::io::{Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

pub fn serialize_model(model: &Model) -> String {
    match serde_json::to_string(model) {
        Ok(json) => json,
        Err(_) => String::from("{}"),
    }
}

pub fn deserialize_model(json: &str, model: &mut Model) -> bool {
    match serde_json::from_str::<Model>(json) {
        Ok(parsed) => {
            model.blocks = parsed.blocks;
            true
        }
        Err(_) => false,
    }
}

pub fn serialize_to_file(model: &Model, path: &str) -> bool {
    let json = serialize_model(model);
    match fs::write(path, json.as_bytes()) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn deserialize_from_file(path: &str, model: &mut Model) -> bool {
    match fs::read_to_string(path) {
        Ok(json) => deserialize_model(&json, model),
        Err(_) => false,
    }
}

pub fn compress_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(data).is_err() {
        return data.to_vec();
    }
    match encoder.finish() {
        Ok(compressed) => compressed,
        Err(_) => data.to_vec(),
    }
}

pub fn decompress_data(data: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Some(decompressed),
        Err(_) => None,
    }
}

pub fn compute_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Block;

    #[test]
    fn test_serialize_deserialize() {
        let mut model = Model::new();
        model.add_block(1, 2, 3);
        model.add_block(4, 5, 6);

        let json = serialize_model(&model);
        assert!(!json.is_empty());

        let mut model2 = Model::new();
        assert!(deserialize_model(&json, &mut model2));
        assert_eq!(model2.get_block_count(), 2);
        assert!(model2.has_block_at(1, 2, 3));
        assert!(model2.has_block_at(4, 5, 6));
    }

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, world! This is a test string for compression.";
        let compressed = compress_data(data);
        let decompressed = decompress_data(&compressed);
        assert!(decompressed.is_some());
        assert_eq!(decompressed.unwrap().as_slice(), data);
    }

    #[test]
    fn test_compute_crc32() {
        let data = b"test data";
        let crc = compute_crc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_empty_model_serialization() {
        let model = Model::new();
        let json = serialize_model(&model);
        assert!(json.contains("blocks"));
    }

    #[test]
    fn test_block_textures_serialization() {
        let mut model = Model::new();
        model.add_block(0, 0, 0);
        model.set_block_texture(0, 0, 0, 0, "grass.png");
        model.set_block_uv(0, 0, 0, 0, 0.0, 0.0, 0.5, 0.5);

        let json = serialize_model(&model);
        let mut model2 = Model::new();
        assert!(deserialize_model(&json, &mut model2));

        let block = model2.get_block(0, 0, 0).unwrap();
        assert_eq!(block.get_face_texture(0), Some("grass.png"));
        let uv = block.get_face_uv(0);
        assert!((uv.u_scale - 0.5).abs() < 0.001);
    }
}
