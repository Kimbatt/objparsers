
extern crate bitflags;

use std::fs::File;
use std::io::{self, BufRead};

fn read_vertex<'a, Iter>(params_iter: &'a mut Iter) -> Result<(f32, f32, f32), Box<dyn std::error::Error>>
where
    Iter: Iterator<Item = &'a [u8]>
{
    let mut vertex = [0f32; 3];
    let mut count = 0;

    for segment in params_iter
    {
        vertex[count] = std::str::from_utf8(segment)?.parse::<f32>()?;
        count += 1;
        if count == 3
        {
            break;
        }
    }

    if count < 3
    {
        Err(format!("3 vertex coordinates are required, only {} found", count).into())
    }
    else
    {
        Ok((vertex[0], vertex[1], vertex[2]))
    }
}

fn read_vertex_texcoord<'a, Iter>(params_iter: &'a mut Iter) -> Result<(f32, f32), Box<dyn std::error::Error>>
where
    Iter: Iterator<Item = &'a [u8]>
{
    let mut vertex = [0f32; 2];
    let mut count = 0;

    for segment in params_iter
    {
        vertex[count] = std::str::from_utf8(segment)?.parse::<f32>()?;
        count += 1;
        if count == 2
        {
            break;
        }
    }

    if count < 2
    {
        Err(format!("2 coordinates are required, only {} found", count).into())
    }
    else
    {
        Ok((vertex[0], vertex[1]))
    }
}

struct ObjVertexRelative
{
    position_index: i32,
    texcoord_index: Option<i32>,
    normal_index: Option<i32>
}

struct ObjVertexAbsolute
{
    position_index: u32,
    texcoord_index: Option<u32>,
    normal_index: Option<u32>
}

impl std::hash::Hash for ObjVertexAbsolute
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        state.write_u32(self.position_index);
        state.write_u32(if let Some(val) = self.texcoord_index { val } else { 0 });
        state.write_u32(if let Some(val) = self.normal_index { val } else { 0 });
        state.finish();
    }
}

impl PartialEq for ObjVertexAbsolute
{
    fn eq(&self, other: &Self) -> bool
    {
        self.position_index == other.position_index &&
        self.texcoord_index == other.texcoord_index &&
        self.normal_index == other.normal_index
    }
}

impl Eq for ObjVertexAbsolute {}

const FACE_TYPE_INDEX_ONLY: u8 = 0b001;
const FACE_TYPE_INDEX_AND_TEXCOORD: u8 = 0b011;
const FACE_TYPE_INDEX_AND_NORMAL: u8 = 0b101;
const FACE_TYPE_INDEX_AND_TEXCOORD_AND_NORMAL: u8 = 0b111;

fn read_face<'a, Iter>(params_iter: &'a mut Iter, temp_face_data: &mut Vec<ObjVertexRelative>) -> Result<u8, Box<dyn std::error::Error>>
where
    Iter: Iterator<Item = &'a [u8]>
{
    temp_face_data.clear();
    let mut line_face_type = None;

    for segment in params_iter
    {
        // there can be any number of indices

        // possible formats:
        // f 1 2 3
        // f 1/1 2/2 3/3
        // f 1//1 2//2 3//3
        // f 1/1/1 2/2/2 3/3/3

        let mut current_indices = [None; 3];
        let mut idx = 0;
        let mut current_face_type = 0_u8;
        for number_str in segment.split(|ch| *ch == b'/')
        {
            if idx >= 3
            {
                return Err("More than 3 face indices found".into());
            }

            current_indices[idx] = if number_str.is_empty()
            {
                None
            }
            else
            {
                current_face_type |= 1 << idx;
                Some(std::str::from_utf8(number_str)?.parse::<i32>()?)
            };

            idx += 1;
        }

        temp_face_data.push(ObjVertexRelative
        {
            position_index: if let Some(pos) = current_indices[0] { pos } else { return Err("Position index is required".into()); },
            texcoord_index: current_indices[1],
            normal_index: current_indices[2]
        });

        if let Some(face_type) = line_face_type
        {
            match current_face_type
            {
                FACE_TYPE_INDEX_ONLY |
                FACE_TYPE_INDEX_AND_TEXCOORD |
                FACE_TYPE_INDEX_AND_NORMAL |
                FACE_TYPE_INDEX_AND_TEXCOORD_AND_NORMAL =>
                {
                    if current_face_type != face_type
                    {
                        return Err("Inconsistent face type".into());
                    }
                },
                _ =>
                {
                    return Err("Unrecognized face type".into());
                }
            };
        }
        else
        {
            line_face_type = Some(current_face_type);
        }
    }

    line_face_type.ok_or("Unknown face type".into())
}

bitflags!
{
    pub struct ObjParseFeatures: u32
    {
        const NONE = 0x00;
        const LOAD_VERTEX_NORMALS = 0x0001;
        const LOAD_VERTEX_TEXCOORDS = 0x0002;

        const LOAD_OBJECTS = 0x0100;
        const LOAD_GROUPS = 0x0200;
        const LOAD_MATERIALS = 0x0400;

        const LOAD_ALL =
            Self::LOAD_VERTEX_NORMALS.bits |
            Self::LOAD_VERTEX_TEXCOORDS.bits |
            Self::LOAD_OBJECTS.bits |
            Self::LOAD_GROUPS.bits |
            Self::LOAD_MATERIALS.bits;

    }
}

#[derive(Copy, Clone)]
enum LineType
{
    Vertex,
    Face,
    VertexTexcoord,
    VertexNormal,
}

pub fn load_obj(file_path: &str, parse_features: ObjParseFeatures) -> Result<(), Box<dyn std::error::Error>>
{
    let load_vertex_normals = (parse_features & ObjParseFeatures::LOAD_VERTEX_NORMALS) != ObjParseFeatures::NONE;
    let load_vertex_texcoords = (parse_features & ObjParseFeatures::LOAD_VERTEX_TEXCOORDS) != ObjParseFeatures::NONE;
    let load_objects = (parse_features & ObjParseFeatures::LOAD_OBJECTS) != ObjParseFeatures::NONE;
    let load_groups = load_objects && (parse_features & ObjParseFeatures::LOAD_GROUPS) != ObjParseFeatures::NONE;
    let load_materials = (parse_features & ObjParseFeatures::LOAD_MATERIALS) != ObjParseFeatures::NONE;

    let load_vertex_pos_only = 
        (parse_features & ObjParseFeatures::LOAD_VERTEX_TEXCOORDS) == ObjParseFeatures::NONE &&
        (parse_features & ObjParseFeatures::LOAD_VERTEX_NORMALS) == ObjParseFeatures::NONE;

    let mut vertices = Vec::<(f32, f32, f32)>::with_capacity(128);
    let mut texcoords = Vec::<(f32, f32)>::with_capacity(128);
    let mut normals = Vec::<(f32, f32, f32)>::with_capacity(128);
    let mut indices = Vec::<u32>::with_capacity(128);
    let mut vertex_index_map = std::collections::hash_map::HashMap::<ObjVertexAbsolute, u32>::with_capacity(128);
    let mut vertex_count = 0;

    let mut temp_face_data = Vec::<ObjVertexRelative>::with_capacity(16);
    let mut temp_face_indices = Vec::<u32>::with_capacity(16);

    let mut lines = Vec::<LineType>::with_capacity(10);
    lines.push(LineType::Vertex);
    lines.push(LineType::Face);

    if load_vertex_texcoords
    {
        lines.push(LineType::VertexTexcoord);
    }

    if load_vertex_normals
    {
        lines.push(LineType::VertexNormal);
    }

    let mut file_face_type = None;

    for line in io::BufReader::new(File::open(file_path)?).lines().flatten()
    {
        let line = line.as_bytes();
        let mut split_iter = line.split(|ch| ch.is_ascii_whitespace());
        if let Some(cmd) = split_iter.next()
        {
            match cmd
            {
                b"v" =>
                {
                    vertices.push(read_vertex(&mut split_iter)?);
                },
                b"vt" if load_vertex_texcoords =>
                {
                    texcoords.push(read_vertex_texcoord(&mut split_iter)?);
                },
                b"vn" if load_vertex_normals =>
                {
                    normals.push(read_vertex(&mut split_iter)?);
                },
                b"f" =>
                {
                    // check face type
                    let current_face_type = read_face(&mut split_iter, &mut temp_face_data)?;
                    match file_face_type
                    {
                        Some(face_type) =>
                        {
                            if face_type != current_face_type
                            {
                                return Err("Inconsistent face types found across multiple lines".into());
                            }
                        },
                        None =>
                        {
                            // first face
                            file_face_type = Some(current_face_type);
                        }
                    };

                    // if the index is negative, then it refers to relative vertices (-1 refers to the currently last vertex in the list, -2 to the second last, etc.)
                    let map_position_index = |index: i32| -> u32
                    {
                        (if index < 0 { vertices.len() as i32 + index } else { index - 1 }) as u32
                    };

                    let map_texcoord_index = |index: i32| -> u32
                    {
                        (if index < 0 { texcoords.len() as i32 + index } else { index - 1 }) as u32
                    };

                    let map_normal_index = |index: i32| -> u32
                    {
                        (if index < 0 { normals.len() as i32 + index } else { index - 1 }) as u32
                    };

                    temp_face_indices.clear();
                    for vertex in temp_face_data.iter()
                    {
                        let vertex_absolute = ObjVertexAbsolute
                        {
                            position_index: map_position_index(vertex.position_index),
                            texcoord_index: if let Some(idx) = vertex.texcoord_index { Some(map_texcoord_index(idx)) } else { None },
                            normal_index: if let Some(idx) = vertex.normal_index { Some(map_normal_index(idx)) } else { None },
                        };

                        let vertex_index = if load_vertex_pos_only
                        {
                            vertex_absolute.position_index
                        }
                        else
                        {
                            match vertex_index_map.entry(vertex_absolute)
                            {
                                std::collections::hash_map::Entry::Occupied(entry) =>
                                {
                                    *entry.get()
                                },
                                std::collections::hash_map::Entry::Vacant(entry) =>
                                {
                                    let idx = vertex_count;
                                    vertex_count += 1;
                                    entry.insert(idx);
                                    idx
                                }
                            }
                        };

                        temp_face_indices.push(vertex_index);
                    }

                    if temp_face_indices.len() < 3
                    {
                        return Err("At least 3 vertex indices are required".into());
                    }

                    let idx0 = temp_face_indices[0];
                    for i in 2..temp_face_indices.len()
                    {
                        let idx1 = temp_face_indices[i - 1];
                        let idx2 = temp_face_indices[i];

                        // vertices are in counter-clockwise order
                        indices.push(idx0);
                        indices.push(idx2);
                        indices.push(idx1);
                    }
                },
                b"o" if load_objects =>
                {
                    // TODO
                },
                b"g" if load_groups =>
                {
                    // TODO
                },
                b"mtllib" if load_materials =>
                {
                    // TODO
                },
                _ => { }
            };
        }
    }

    println!("vertices: {}", vertices.len());
    println!("texcoords: {}", texcoords.len());
    println!("normals: {}", normals.len());
    println!("indices: {}", indices.len());

    Ok(())
}
