
extern crate bitflags;
extern crate lexical;

fn format_parse_error<T>(bytes: &[u8]) -> String
{
    format!("Parse error: {}",
    {
        let type_str = std::any::type_name::<T>();
        if let Ok(s) = std::str::from_utf8(bytes.into())
        {
            format!("cannot parse {} as {}", s, type_str)
        }
        else
        {
            type_str.into()
        }
    })
}

fn try_parse_f32(bytes: &[u8]) -> Result<f32, Box<dyn std::error::Error>>
{
    match fast_float::parse::<f32, _>(bytes)
    {
        Ok(val) => Ok(val),
        Err(_) => Err(format_parse_error::<f32>(bytes).into())
    }
}

fn try_parse<T: lexical::FromLexical>(bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>
{
    match lexical::parse::<T, _>(bytes)
    {
        Ok(val) => Ok(val),
        Err(_) => Err(format_parse_error::<T>(bytes).into())
    }
}

fn read_vertex<'a, Iter>(params_iter: &'a mut Iter) -> Result<(f32, f32, f32), Box<dyn std::error::Error>>
where
    Iter: Iterator<Item = &'a [u8]>
{
    let mut vertex = [0f32; 3];
    let mut count = 0;

    for segment in params_iter
    {
        vertex[count] = try_parse_f32(segment)?;
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
        vertex[count] = try_parse_f32(segment)?;
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

#[derive(Copy, Clone)]
pub struct ObjVertexAbsolute
{
    pub position_index: u32,
    pub texcoord_index: Option<u32>,
    pub normal_index: Option<u32>
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
        assert!(!segment.is_empty());
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
                Some(try_parse::<i32>(number_str)?)
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

    line_face_type.ok_or_else(|| "Unknown face type".into())
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

pub struct ObjParseResult
{
    pub objects: Vec<ObjObject>,
    pub vertex_buffer: Vec<Vec3>,
    pub texcoord_buffer: Option<Vec<Vec2>>,
    pub normal_buffer: Option<Vec<Vec3>>
}

pub fn load_obj(file_path: &str, parse_features: ObjParseFeatures) -> Result<ObjParseResult, Box<dyn std::error::Error>>
{
    let file_bytes = std::fs::read(file_path)?;
    load_obj_from_bytes(file_bytes.as_slice(), parse_features)
}

#[repr(C)]
pub struct Vector2<T>
{
    pub x: T,
    pub y: T
}

#[repr(C)]
pub struct Vector3<T>
{
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T> Vector2<T>
{
    fn new(x: T, y: T) -> Self
    {
        Self { x, y }
    }
}

impl<T> Vector3<T>
{
    fn new(x: T, y: T, z: T) -> Self
    {
        Self { x, y, z }
    }
}

type Vec2 = Vector2<f32>;
type Vec3 = Vector3<f32>;

pub struct ObjObject
{
    pub name: Vec<u8>,
    pub indices: Vec<Vector3<ObjVertexAbsolute>>
}

pub fn load_obj_from_bytes(file_bytes: &[u8], parse_features: ObjParseFeatures) -> Result<ObjParseResult, Box<dyn std::error::Error>>
{
    let load_vertex_normals = (parse_features & ObjParseFeatures::LOAD_VERTEX_NORMALS) != ObjParseFeatures::NONE;
    let load_vertex_texcoords = (parse_features & ObjParseFeatures::LOAD_VERTEX_TEXCOORDS) != ObjParseFeatures::NONE;
    let load_objects = (parse_features & ObjParseFeatures::LOAD_OBJECTS) != ObjParseFeatures::NONE;
    let load_groups = load_objects && (parse_features & ObjParseFeatures::LOAD_GROUPS) != ObjParseFeatures::NONE;
    let load_materials = (parse_features & ObjParseFeatures::LOAD_MATERIALS) != ObjParseFeatures::NONE;

    let mut all_objects = Vec::<ObjObject>::new();
    all_objects.push(ObjObject
    {
        name: vec![],
        indices: vec![]
    });

    let mut current_object = &mut all_objects[0];

    let mut vertices = Vec::<Vec3>::with_capacity(128);
    let mut texcoords = Vec::<Vec2>::with_capacity(if load_vertex_texcoords { 128 } else { 0 });
    let mut normals = Vec::<Vec3>::with_capacity(if load_vertex_normals { 128 } else { 0 });

    let mut temp_face_vertices = Vec::<ObjVertexRelative>::with_capacity(16);
    let mut temp_face_vertices_absolute = Vec::<ObjVertexAbsolute>::with_capacity(16);

    let mut file_face_type = None;

    for line in file_bytes.split(|ch| *ch == b'\n' || *ch == b'\r')
    {
        let mut split_iter = line.split(|ch| ch.is_ascii_whitespace()).filter(|segment| !segment.is_empty());
        if let Some(cmd) = split_iter.next()
        {
            match cmd
            {
                b"v" =>
                {
                    let vertex = read_vertex(&mut split_iter)?;
                    vertices.push(Vec3::new(vertex.0, vertex.1, vertex.2));
                },
                b"vt" =>
                {
                    if load_vertex_texcoords
                    {
                        let texcoord = read_vertex_texcoord(&mut split_iter)?;
                        texcoords.push(Vec2::new(texcoord.0, texcoord.1));
                    }
                },
                b"vn" =>
                {
                    if load_vertex_normals
                    {
                        let normal = read_vertex(&mut split_iter)?;
                        normals.push(Vec3::new(normal.0, normal.1, normal.2));
                    }
                },
                b"f" =>
                {
                    // check face type
                    let current_face_type = read_face(&mut split_iter, &mut temp_face_vertices)?;
                    match file_face_type
                    {
                        Some(face_type) =>
                        {
                            if face_type != current_face_type
                            {
                                // return Err("Inconsistent face types found across multiple lines".into());
                            }
                        },
                        None =>
                        {
                            // first face
                            file_face_type = Some(current_face_type);
                        }
                    };

                    // if the index is negative, then it refers to relative vertices (-1 refers to the currently last vertex in the list, -2 to the second last, etc.)
                    let validate_index = |index: i32, current_count: u32| -> Result<u32, Box<dyn std::error::Error>>
                    {
                        if index <= 0
                        {
                            let absolute_index = current_count as i32 + index;
                            if absolute_index < 0
                            {
                                Err(format!("Relative index {} is out of bounds", index).into())
                            }
                            else
                            {
                                Ok(absolute_index as u32)
                            }
                        }
                        else
                        {
                            Ok((index - 1) as u32)
                        }
                    };

                    let map_position_index = |index: i32| -> Result<u32, Box<dyn std::error::Error>>
                    {
                        validate_index(index, vertices.len() as u32)
                    };

                    let map_texcoord_index = |index: i32| -> Result<u32, Box<dyn std::error::Error>>
                    {
                        validate_index(index, texcoords.len() as u32)
                    };

                    let map_normal_index = |index: i32| -> Result<u32, Box<dyn std::error::Error>>
                    {
                        validate_index(index, normals.len() as u32)
                    };

                    temp_face_vertices_absolute.clear();
                    for vertex in temp_face_vertices.iter()
                    {
                        let vertex_absolute = ObjVertexAbsolute
                        {
                            position_index: map_position_index(vertex.position_index)?,
                            texcoord_index: if let Some(idx) = vertex.texcoord_index { Some(map_texcoord_index(idx)?) } else { None },
                            normal_index: if let Some(idx) = vertex.normal_index { Some(map_normal_index(idx)?) } else { None },
                        };

                        if vertex_absolute.position_index as usize >= vertices.len()
                        {
                            return Err(format!("Vertex position index {} is referenced, but the largest possible index is {}",
                                vertex_absolute.position_index, vertices.len() - 1).into());
                        }

                        if load_vertex_texcoords
                        {
                            if let Some(texcoord_index) = vertex_absolute.texcoord_index
                            {
                                if texcoord_index as usize >= texcoords.len()
                                {
                                    return Err(format!("Vertex texcoord index {} is referenced, but the largest possible index is {}",
                                        texcoord_index, texcoords.len() - 1).into());
                                }
                            }
                        }

                        if load_vertex_normals
                        {
                            if let Some(normal_index) = vertex_absolute.normal_index
                            {
                                if normal_index as usize >= normals.len()
                                {
                                    return Err(format!("Vertex normal index {} is referenced, but the largest possible index is {}",
                                        normal_index, normals.len() - 1).into());
                                }
                            }
                        }

                        temp_face_vertices_absolute.push(vertex_absolute);
                    }

                    if temp_face_vertices_absolute.len() < 3
                    {
                        return Err("At least 3 vertex indices are required".into());
                    }

                    let idx0 = temp_face_vertices_absolute[0];
                    for i in 2..temp_face_vertices_absolute.len()
                    {
                        let idx1 = temp_face_vertices_absolute[i - 1];
                        let idx2 = temp_face_vertices_absolute[i];

                        current_object.indices.push(Vector3::new(idx0, idx1, idx2));
                    }
                },
                b"o" if load_objects =>
                {
                    let start_index = if let Some(pos) = line[1..].iter().position(|ch| !ch.is_ascii_whitespace())
                    {
                        pos + 1
                    }
                    else
                    {
                        1
                    };

                    let object_name = &line[start_index..];

                    let current_object_index =
                    {
                        let index = all_objects.iter().enumerate()
                            .find(|(_idx, object)| object.name == object_name)
                            .map(|res| res.0);

                        if let Some(idx) = index
                        {
                            idx
                        }
                        else
                        {
                            let idx = all_objects.len();
                            all_objects.push(ObjObject
                            {
                                name: object_name.to_owned(),
                                indices: vec![]
                            });

                            idx
                        }
                    };

                    current_object = &mut all_objects[current_object_index];
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

    if all_objects[0].indices.is_empty()
    {
        // remove default object if empty
        all_objects.remove(0);
    }

    Ok(ObjParseResult {
        objects: all_objects,
        vertex_buffer: vertices,
        texcoord_buffer: if load_vertex_texcoords && !texcoords.is_empty() { Some(texcoords) } else { None },
        normal_buffer: if load_vertex_normals && !normals.is_empty() { Some(normals) } else { None }
    })
}
