
extern crate rayon;
extern crate bitflags;

use regex::*;

use std::fs::File;
use std::io::{self, BufRead};

use super::finite_state_machine::*;
use crate::parse_helper::*;

lazy_static::lazy_static!
{
    static ref VERTEX_REGEX: Regex = Regex::new(r"^\s*v\s+(-?\d+(\.\d+)?)\s+(-?\d+(\.\d+)?)\s+(-?\d+(\.\d+)?)").unwrap();

    static ref FACE_REGEX: Regex = Regex::new(r"((-?\d+)/(-?\d+)/(-?\d+))|((-?\d+)//(-?\d+))|((-?\d+)/(-?\d+))|(-?\d+)").unwrap();
    static ref FACE_START_REGEX: Regex = Regex::new(r"^\s*f\s+").unwrap();
}

// try to read 3 float values from a line that was accepted by the vertex recognizer
fn read_vertex(line: &[u8]) -> Option<(f32, f32, f32)>
{
    let mut index = 0;
    let end_index = line.len();
    while index < end_index && FiniteStateMachine::is_whitespace(line[index])
    {
        index += 1;
    }

    index += 1; // skip the letter 'v'

    let mut vertex = [0.0_f32; 3];

    for i in 0..3
    {
        match read_float(line, index)
        {
            Some((val, length)) =>
            {
                index += length + 1; // skip space
                vertex[i] = val;
            },
            None => return None
        };
    }

    Some((vertex[0], vertex[1], vertex[2]))
}

fn read_face(line: &[u8], temp_indices: &mut Vec<i32>) -> bool
{
    let mut index = 0;
    let end_index = line.len();

    while index < end_index && FiniteStateMachine::is_whitespace(line[index])
    {
        index += 1;
    }

    index += 1; // skip the letter 'f'

    loop
    {
        match read_int(line, index)
        {
            ReadIndexResult::Ok(result, length) =>
            {
                temp_indices.push(result);
                index += length + 1;
            },
            ReadIndexResult::Error => return false,
            ReadIndexResult::Finished => return true
        };
    }
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

pub fn load_obj(file_path: &str, parse_features: ObjParseFeatures) -> Result<(), Box<dyn std::error::Error>>
{
    let vertex_recognizer: FiniteStateMachine = FiniteStateMachine::new(3, {
        let mut stc = StateTransitionCollection::new(3);
        stc.set_transition(0, 0, FiniteStateMachine::whitespace_pattern());
        stc.set_transition(0, 1, FiniteStateMachine::single_character_pattern(b'v'));
        stc.set_transition(1, 2, FiniteStateMachine::whitespace_pattern());
        stc
    }, true, vec![2]);
    
    let vertex_normal_recognizer: FiniteStateMachine = FiniteStateMachine::new(4, {
        let mut stc = StateTransitionCollection::new(4);
        stc.set_transition(0, 0, FiniteStateMachine::whitespace_pattern());
        stc.set_transition(0, 1, FiniteStateMachine::single_character_pattern(b'v'));
        stc.set_transition(1, 2, FiniteStateMachine::single_character_pattern(b'n'));
        stc.set_transition(2, 3, FiniteStateMachine::whitespace_pattern());
        stc
    }, true, vec![3]);
    
    let vertex_texcoord_recognizer: FiniteStateMachine = FiniteStateMachine::new(4, {
        let mut stc = StateTransitionCollection::new(4);
        stc.set_transition(0, 0, FiniteStateMachine::whitespace_pattern());
        stc.set_transition(0, 1, FiniteStateMachine::single_character_pattern(b'v'));
        stc.set_transition(1, 2, FiniteStateMachine::single_character_pattern(b't'));
        stc.set_transition(2, 3, FiniteStateMachine::whitespace_pattern());
        stc
    }, true, vec![3]);
    
    let face_recognizer: FiniteStateMachine = FiniteStateMachine::new(3, {
        let mut stc = StateTransitionCollection::new(3);
        stc.set_transition(0, 0, FiniteStateMachine::whitespace_pattern());
        stc.set_transition(0, 1, FiniteStateMachine::single_character_pattern(b'f'));
        stc.set_transition(1, 2, FiniteStateMachine::whitespace_pattern());
        stc
    }, true, vec![2]);

    let mut vertices = Vec::<(f32, f32, f32)>::with_capacity(128);
    let mut indices = Vec::<(u32, u32, u32)>::with_capacity(128);
    let mut temp_face_indices = Vec::<i32>::with_capacity(16);

    for (line_idx, line) in io::BufReader::new(File::open(file_path)?).lines().flatten().enumerate()
    {
        let line = line.as_bytes();
        if vertex_recognizer.test(line)
        {
            if let Some(vertex) = read_vertex(line)
            {
                vertices.push(vertex);
            }
            else
            {
                return Err(format!("Vertex position cannot be converted to a number on line {}", line_idx + 1).into());
            }
        }
        else if face_recognizer.test(line)
        {
            temp_face_indices.clear();
            if !read_face(line, &mut temp_face_indices)
            {
                return Err(format!("Vertex indices cannot be converted to a number on line {}", line_idx + 1).into());
            }
            if temp_face_indices.len() < 3
            {
                return Err(format!("Face contains less than 3 vertex indices on line {}", line_idx + 1).into());
            }

            // triangulation for polygon faces, also works for single triangles (assuming the points are coplanar)
            let vertex_start_index = vertices.len() as u32;
            
            // if the index is negative, then it refers to relative vertices (-1 refers to the currently last vertex in the list, -2 to the second last, etc.)
            let map_index = |index: i32| -> u32
            {
                (if index < 0 { vertex_start_index as i32 + index } else { index - 1 }) as u32
            };

            let index0 = map_index(temp_face_indices[0]);

            for i in 2..temp_face_indices.len()
            {
                let index1 = map_index(temp_face_indices[i - 1]);
                let index2 = map_index(temp_face_indices[i]);
                indices.push((index0, index2, index1));
            }
        }
    }

    Ok(())
}

pub fn load_obj_with_regex(file_path: &str) -> Result<(), Box<dyn std::error::Error>>
{
    // 2 6 9 11 capture groups are the vertex indices that we care about
    const FACE_GROUP_INDICES: [usize; 4] = [2, 6, 9, 11];

    let mut vertices = Vec::<(f32, f32, f32)>::with_capacity(128);
    let mut faces = Vec::<(u32, u32, u32)>::with_capacity(128);
    let mut temp_face_indices = Vec::<u32>::with_capacity(16);

    for line in io::BufReader::new(File::open(file_path)?).lines()
    {
        let line = line?;
        if let Some(m) = VERTEX_REGEX.captures(&line)
        {
            let x = m[1].parse::<f32>()?;
            let y = m[3].parse::<f32>()?;
            let z = m[5].parse::<f32>()?;
            
            vertices.push((x, y, z));
        }
        else if FACE_START_REGEX.is_match(&line)
        {
            temp_face_indices.clear();
            for capture in FACE_REGEX.captures_iter(&line)
            {
                let mut index_value = None;
                for face_group_idx in FACE_GROUP_INDICES.iter()
                {
                    if let Some(c) = capture.get(*face_group_idx)
                    {
                        index_value = Some(str::parse::<i32>(c.as_str())?);
                        break;
                    }
                }

                let index = match index_value
                {
                    Some(idx) => idx,
                    None => return Err("Face index expected".into()) 
                };

                let absolute_index = if index == 0
                {
                    return Err("Face index shouldn't be zero".into());
                }
                else if index < 0
                {
                    // relative indices, -1 points to the last vertex
                    let idx = vertices.len() as i32 + index;
                    if idx < 0
                    {
                        return Err("Relative index is out of range".into());
                    }

                    idx as u32
                }
                else
                {
                    // indices start at 1
                    (index - 1) as u32
                };

                temp_face_indices.push(absolute_index);
            }

            // faces are not always triangles
            let i0 = temp_face_indices[0];
            for i in 2..temp_face_indices.len()
            {
                let i1 = temp_face_indices[i - 1];
                let i2 = temp_face_indices[i];

                faces.push((i0, i1, i2));
            }
        }
    }

    Ok(())
}

