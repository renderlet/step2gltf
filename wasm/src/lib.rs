/// Takes a STEP file (as an array of bytes), and returns a triangle mesh.
///
/// Vertices are packed into rows of 9 floats, representing
/// - Position
/// - Normal
/// - Color
///
/// Vertices are rows of three indexes into the triangle array
///

//use gltf_json as json;
use gltf::json;

use std::mem;

use json::validation::Checked::Valid;
use json::validation::USize64;
use std::borrow::Cow;
//use std::io::Write;

use wasm_bindgen::prelude::*;
use log::Level;
use log::info;

#[derive(Copy, Clone, Debug, bytemuck::NoUninit)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
}

/// Calculate bounding coordinates of a list of vertices, used for the clipping distance of the model
fn bounding_coords(points: &[Vertex]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.position;
        for i in 0..3 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}

fn to_padded_byte_vector<T: bytemuck::NoUninit>(data: &[T]) -> Vec<u8> {
    let byte_slice: &[u8] = bytemuck::cast_slice(data);
    let mut new_vec: Vec<u8> = byte_slice.to_owned();


    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }

    new_vec
}

#[wasm_bindgen]
pub fn init_log() {
    console_log::init_with_level(Level::Info).expect("Failed to initialize log");
}

#[wasm_bindgen]
pub fn step_to_triangle_buf(data: String) -> Vec<f32> {
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate; // lol

    let flat = StepFile::strip_flatten(data.as_bytes());
    let step = StepFile::parse(&flat);
    let (mut mesh, _stats) = triangulate(&step);

    let (mut xmin, mut xmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut ymin, mut ymax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut zmin, mut zmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    for pos in mesh.verts.iter().map(|p| p.pos) {
        xmin = xmin.min(pos.x);
        xmax = xmax.max(pos.x);
        ymin = ymin.min(pos.y);
        ymax = ymax.max(pos.y);
        zmin = ymin.min(pos.z);
        zmax = ymax.max(pos.z);
    }
    let scale = (xmax - xmin).max(ymax - ymin).max(zmax - zmin);
    let xc = (xmax + xmin) / 2.0;
    let yc = (ymax + ymin) / 2.0;
    let zc = (zmax + zmin) / 2.0;
    for pos in mesh.verts.iter_mut().map(|p| &mut p.pos) {
        pos.x = (pos.x - xc) / scale * 200.0;
        pos.y = (pos.y - yc) / scale * 200.0;
        pos.z = (pos.z - zc) / scale * 200.0;
    }

    mesh.triangles.iter()
        .flat_map(|v| v.verts.iter())
        .map(|p| &mesh.verts[*p as usize])
        .flat_map(|v| v.pos.iter().chain(&v.norm).chain(&v.color))
        .map(|f| *f as f32)
        .collect()
}

#[wasm_bindgen]
pub fn step_to_gltf(data: String) -> Vec<u8> {
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate; 

    let flat = StepFile::strip_flatten(data.as_bytes());
    let step = StepFile::parse(&flat);
    let (mut mesh, _stats) = triangulate(&step);

    let (mut xmin, mut xmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut ymin, mut ymax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut zmin, mut zmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    for pos in mesh.verts.iter().map(|p| p.pos) {
        xmin = xmin.min(pos.x);
        xmax = xmax.max(pos.x);
        ymin = ymin.min(pos.y);
        ymax = ymax.max(pos.y);
        zmin = ymin.min(pos.z);
        zmax = ymax.max(pos.z);
    }
    let scale = (xmax - xmin).max(ymax - ymin).max(zmax - zmin);
    let xc = (xmax + xmin) / 2.0;
    let yc = (ymax + ymin) / 2.0;
    let zc = (zmax + zmin) / 2.0;
    for pos in mesh.verts.iter_mut().map(|p| &mut p.pos) {
        pos.x = (pos.x - xc) / scale * 200.0;
        pos.y = (pos.y - yc) / scale * 200.0;
        pos.z = (pos.z - zc) / scale * 200.0;
    }

    /*
    let tris: Vec<f32> = mesh.triangles.iter()
        .flat_map(|v| v.verts.iter())
        .map(|p| &mesh.verts[*p as usize])
        .flat_map(|v| v.pos.iter().chain(&v.norm).chain(&v.color))
        .map(|f| *f as f32)
        .collect();
    */

    let tris: Vec<f32> = mesh.triangles.iter()
        .flat_map(|v| v.verts.iter())
        .map(|p| &mesh.verts[*p as usize])
        .flat_map(|v| v.pos.iter().chain(&v.color).chain(&v.norm))
        .map(|f| *f as f32)
        .collect();

    info!("Collected triangles");

    let mut triangle_vertices: Vec<Vertex> = Vec::new();

    for i in (0..tris.len()).step_by(9) {
        triangle_vertices.push(
            Vertex {
                position: [tris[i + 0], tris[i + 1], tris[i + 2]],
                color: [tris[i + 3], tris[i + 4], tris[i + 5]],
                normal: [tris[i + 6], tris[i + 7], tris[i + 8]],
            }
        );
    }

    /*
    let triangle_vertices = vec![
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
    */

    info!("Mapped triangles");

    let (min, max) = bounding_coords(&triangle_vertices);

    let mut root = json::Root::default();

    let buffer_length = triangle_vertices.len() * mem::size_of::<Vertex>();
    let buffer = root.push(json::Buffer {
        byte_length: USize64::from(buffer_length),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None,
    });
    let buffer_view = root.push(json::buffer::View {
        buffer,
        byte_length: USize64::from(buffer_length),
        byte_offset: None,
        byte_stride: Some(json::buffer::Stride(mem::size_of::<Vertex>())),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    let positions = root.push(json::Accessor {
        buffer_view: Some(buffer_view),
        byte_offset: Some(USize64(0)),
        count: USize64::from(triangle_vertices.len()),
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Vec3),
        min: Some(json::Value::from(Vec::from(min))),
        max: Some(json::Value::from(Vec::from(max))),
        name: None,
        normalized: false,
        sparse: None,
    });
    let colors = root.push(json::Accessor {
        buffer_view: Some(buffer_view),
        byte_offset: Some(USize64::from(3 * mem::size_of::<f32>())),
        count: USize64::from(triangle_vertices.len()),
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Vec3),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    });
    let normals = root.push(json::Accessor {
        buffer_view: Some(buffer_view),
        byte_offset: Some(USize64::from(6 * mem::size_of::<f32>())),
        count: USize64::from(triangle_vertices.len()),
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Vec3),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    });

    let primitive = json::mesh::Primitive {
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(Valid(json::mesh::Semantic::Positions), positions);
            map.insert(Valid(json::mesh::Semantic::Colors(0)), colors);
            map.insert(Valid(json::mesh::Semantic::Normals), normals);
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: None,
        material: None,
        mode: Valid(json::mesh::Mode::Triangles),
        targets: None,
    };

    let mesh = root.push(json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    });

    let node = root.push(json::Node {
        mesh: Some(mesh),
        ..Default::default()
    });

    root.push(json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        nodes: vec![node],
    });

    info!("Gltf structs mapped");

    let json_string = json::serialize::to_string(&root).expect("Serialization error");
    let mut json_offset = json_string.len();
    align_to_multiple_of_four(&mut json_offset);

 
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            // N.B., the size of binary glTF file is limited to range of `u32`.
            length: (json_offset + buffer_length)
                .try_into()
                .expect("file size exceeds binary glTF limit"),
        },
        bin: Some(Cow::Owned(to_padded_byte_vector(&triangle_vertices))),
        json: Cow::Owned(json_string.into_bytes()),
    };

    info!("Glb header generated");

    let mut out: Vec<u8> = Vec::new();

    glb.to_writer(&mut out).expect("glTF binary output error");

    info!("Writer written, len {}", out.len());

    out.clone()
    //Vec::new()
}