use crate::model::Model;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ExportFormat {
    GLB,
    GLTF,
    OBJ,
    M3X,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExportOptions {
    pub include_textures: bool,
    pub include_materials: bool,
    pub scale: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            include_textures: true,
            include_materials: true,
            scale: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn export_model(model: &Model, format: ExportFormat, path: &str, options: &ExportOptions) -> bool {
    match format {
        ExportFormat::GLB => {
            crate::gltf_exporter::export_glb(model, path, options)
        }
        ExportFormat::GLTF => {
            crate::gltf_exporter::export_gltf(model, path, options)
        }
        ExportFormat::OBJ => {
            crate::obj_exporter::export_obj(model, path, options)
        }
        ExportFormat::M3X => {
            crate::m3x_exporter::export_m3x(model, path, options)
        }
    }
}

#[allow(dead_code)]
pub fn get_format_from_extension(path: &str) -> Option<ExportFormat> {
    if path.ends_with(".glb") {
        Some(ExportFormat::GLB)
    } else if path.ends_with(".gltf") {
        Some(ExportFormat::GLTF)
    } else if path.ends_with(".obj") {
        Some(ExportFormat::OBJ)
    } else if path.ends_with(".m3x") {
        Some(ExportFormat::M3X)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn get_extension(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::GLB => ".glb",
        ExportFormat::GLTF => ".gltf",
        ExportFormat::OBJ => ".obj",
        ExportFormat::M3X => ".m3x",
    }
}

#[allow(dead_code)]
pub fn get_format_name(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::GLB => "GLB (GL Binary)",
        ExportFormat::GLTF => "glTF",
        ExportFormat::OBJ => "Wavefront OBJ",
        ExportFormat::M3X => "M3X (Proprietary)",
    }
}
