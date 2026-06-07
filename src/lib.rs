mod model;
mod export;
mod mesh;
mod serializer;
mod gltf_exporter;
mod obj_exporter;
mod m3x_exporter;

use std::collections::HashMap;
use std::sync::Mutex;
use std::ptr;

use jni::JNIEnv;
use jni::objects::{JClass, JString, JObjectArray};
use jni::sys::{jint, jlong, jboolean, jfloat, jfloatArray, jintArray, jstring, jbyteArray, JNI_FALSE, JNI_TRUE};

use model::{Engine, Model};

lazy_static::lazy_static! {
    static ref ENGINE: Mutex<Option<Engine>> = Mutex::new(None);
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(
    _vm: jni::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jint {
    let mut engine_guard = ENGINE.lock().unwrap();
    *engine_guard = Some(Engine::new());
    jni::sys::JNI_VERSION_1_6
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeCreateEngine(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let mut engine_guard = ENGINE.lock().unwrap();
    if engine_guard.is_none() {
        *engine_guard = Some(Engine::new());
    }
    1
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeDestroyEngine(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let mut engine_guard = ENGINE.lock().unwrap();
    *engine_guard = None;
    JNI_TRUE
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeCreateModel(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    if let Some(ref mut engine) = *engine_guard {
        let model_id = engine.create_model();
        model_id as jlong
    } else {
        -1
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeDestroyModel(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jboolean {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if engine.destroy_model(model_id as u64) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeAddBlock(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jboolean {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            model.add_block(x, y, z);
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeRemoveBlock(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> jboolean {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            model.remove_block(x, y, z);
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeSetBlockTexture(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    x: jint,
    y: jint,
    z: jint,
    face: jint,
    texture: JString,
) -> jboolean {
    let texture_str: String = match env.get_string(&texture) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            model.set_block_texture(x, y, z, face as usize, &texture_str);
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeSetBlockUV(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    x: jint,
    y: jint,
    z: jint,
    face: jint,
    u: jfloat,
    v: jfloat,
    u_scale: jfloat,
    v_scale: jfloat,
) -> jboolean {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            model.set_block_uv(x, y, z, face as usize, u, v, u_scale, v_scale);
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeBuildMesh(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jboolean {
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            let _mesh = mesh::build_scene_mesh(model);
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeGetVertices(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jfloatArray {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return ptr::null_mut(),
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            let vertices = model.get_all_vertices();
            let arr = env.new_float_array(vertices.len() as i32).unwrap_or(ptr::null_mut());
            if !arr.is_null() {
                env.set_float_array_region(arr, 0, &vertices).ok();
            }
            return arr;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeGetNormals(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jfloatArray {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return ptr::null_mut(),
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            let normals = model.get_all_normals();
            let arr = env.new_float_array(normals.len() as i32).unwrap_or(ptr::null_mut());
            if !arr.is_null() {
                env.set_float_array_region(arr, 0, &normals).ok();
            }
            return arr;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeGetUVs(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jfloatArray {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return ptr::null_mut(),
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            let uvs = model.get_all_uvs();
            let arr = env.new_float_array(uvs.len() as i32).unwrap_or(ptr::null_mut());
            if !arr.is_null() {
                env.set_float_array_region(arr, 0, &uvs).ok();
            }
            return arr;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeGetIndices(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jintArray {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return ptr::null_mut(),
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            let indices = model.get_all_indices();
            let arr = env.new_int_array(indices.len() as i32).unwrap_or(ptr::null_mut());
            if !arr.is_null() {
                env.set_int_array_region(arr, 0, &indices).ok();
            }
            return arr;
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeExportGLB(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    path: JString,
    include_textures: jboolean,
    include_materials: jboolean,
    scale: jfloat,
) -> jboolean {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let options = export::ExportOptions {
        include_textures: include_textures != JNI_FALSE,
        include_materials: include_materials != JNI_FALSE,
        scale,
    };
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            if gltf_exporter::export_glb(model, &path_str, &options) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeExportGLTF(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    path: JString,
    include_textures: jboolean,
    include_materials: jboolean,
    scale: jfloat,
) -> jboolean {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let options = export::ExportOptions {
        include_textures: include_textures != JNI_FALSE,
        include_materials: include_materials != JNI_FALSE,
        scale,
    };
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            if gltf_exporter::export_gltf(model, &path_str, &options) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeExportOBJ(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    path: JString,
    include_textures: jboolean,
    include_materials: jboolean,
    scale: jfloat,
) -> jboolean {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let options = export::ExportOptions {
        include_textures: include_textures != JNI_FALSE,
        include_materials: include_materials != JNI_FALSE,
        scale,
    };
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            if obj_exporter::export_obj(model, &path_str, &options) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeExportM3X(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    path: JString,
    include_textures: jboolean,
    include_materials: jboolean,
    scale: jfloat,
) -> jboolean {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let options = export::ExportOptions {
        include_textures: include_textures != JNI_FALSE,
        include_materials: include_materials != JNI_FALSE,
        scale,
    };
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            if m3x_exporter::export_m3x(model, &path_str, &options) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeImportM3X(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    path: JString,
) -> jboolean {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            if m3x_exporter::import_m3x(&path_str, model) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeSerializeModel(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jstring {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return ptr::null_mut(),
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            let json = serializer::serialize_model(model);
            match env.new_string(json) {
                Ok(s) => return s.into_raw(),
                Err(_) => return ptr::null_mut(),
            }
        }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeDeserializeModel(
    mut env: JNIEnv,
    _class: JClass,
    model_id: jlong,
    json: JString,
) -> jboolean {
    let json_str: String = match env.get_string(&json) {
        Ok(s) => s.into(),
        Err(_) => return JNI_FALSE,
    };
    let mut engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return JNI_FALSE,
    };
    if let Some(ref mut engine) = *engine_guard {
        if let Some(model) = engine.get_model_mut(model_id as u64) {
            if serializer::deserialize_model(&json_str, model) {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        } else {
            JNI_FALSE
        }
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_dbuild_net_RustBridge_nativeGetBlockCount(
    _env: JNIEnv,
    _class: JClass,
    model_id: jlong,
) -> jint {
    let engine_guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    if let Some(ref engine) = *engine_guard {
        if let Some(model) = engine.get_model(model_id as u64) {
            model.get_block_count() as jint
        } else {
            -1
        }
    } else {
        -1
    }
}
