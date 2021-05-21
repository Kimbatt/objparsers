
use super::obj::*;
use std::{fs::File, io::BufWriter};
use std::io::prelude::*;

impl ObjParseResult
{
    pub fn export(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut writer = BufWriter::new(std::fs::File::create(file_path)?);

        for pos in self.vertex_buffer.iter()
        {
            writer.write_fmt(format_args!("v {} {} {}\n", pos.x, pos.y, pos.z))?;
        }

        let has_texcoords = if let Some(texcoords) = &self.texcoord_buffer
        {
            for uv in texcoords.iter()
            {
                writer.write_fmt(format_args!("vt {} {}\n", uv.x, uv.y))?;
            }

            true
        }
        else
        {
            false
        };

        let has_normals = if let Some(normals) = &self.normal_buffer
        {
            for normal in normals.iter()
            {
                writer.write_fmt(format_args!("vn {} {} {}\n", normal.x, normal.y, normal.z))?;
            }

            true
        }
        else
        {
            false
        };

        for obj in self.objects.iter()
        {
            if !obj.name.is_empty()
            {
                writer.write(b"o ")?;
                writer.write(obj.name.as_slice())?;
                writer.write(b"\n")?;
            }

            fn write_positions(writer: &mut BufWriter<File>, px: u32, py: u32, pz: u32) -> Result<(), std::io::Error>
            {
                writer.write_fmt(format_args!("f {} {} {}\n", px + 1, py + 1, pz + 1))
            }

            fn write_positions_and_texcoords(writer: &mut BufWriter<File>, px: u32, py: u32, pz: u32, tx: u32, ty: u32, tz: u32) -> Result<(), std::io::Error>
            {
                writer.write_fmt(format_args!("f {}/{} {}/{} {}/{}\n",
                    px + 1, tx + 1,
                    py + 1, ty + 1,
                    pz + 1, tz + 1
                ))
            }

            fn write_positions_and_normals(writer: &mut BufWriter<File>, px: u32, py: u32, pz: u32, nx: u32, ny: u32, nz: u32) -> Result<(), std::io::Error>
            {
                writer.write_fmt(format_args!("f {}//{} {}//{} {}//{}\n",
                    px + 1, nx + 1,
                    py + 1, ny + 1,
                    pz + 1, nz + 1
                ))
            }

            fn write_all(writer: &mut BufWriter<File>, px: u32, py: u32, pz: u32, tx: u32, ty: u32, tz: u32, nx: u32, ny: u32, nz: u32) -> Result<(), std::io::Error>
            {
                writer.write_fmt(format_args!("f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                    px + 1, tx + 1, nx + 1,
                    py + 1, ty + 1, ny + 1,
                    pz + 1, tz + 1, nz + 1
                ))
            }

            for tri in obj.indices.iter()
            {
                match (has_texcoords, has_normals)
                {
                    (false, false) =>
                    {
                        write_positions(&mut writer, tri.x.position_index, tri.y.position_index, tri.z.position_index)?;
                    },
                    (true, false) =>
                    {
                        match (tri.x.texcoord_index, tri.y.texcoord_index, tri.z.texcoord_index)
                        {
                            (Some(tx), Some(ty), Some(tz)) =>
                            {
                                write_positions_and_texcoords(&mut writer, tri.x.position_index, tri.y.position_index, tri.z.position_index, tx, ty, tz)?;
                            },
                            _ =>
                            {
                                write_positions(&mut writer, tri.x.position_index, tri.y.position_index, tri.z.position_index)?;
                            }
                        };
                    },
                    (false, true) =>
                    {
                        match (tri.x.normal_index, tri.y.normal_index, tri.z.normal_index)
                        {
                            (Some(nx), Some(ny), Some(nz)) =>
                            {
                                write_positions_and_normals(&mut writer, tri.x.position_index, tri.y.position_index, tri.z.position_index, nx, ny, nz)?;
                            },
                            _ =>
                            {
                                write_positions(&mut writer, tri.x.position_index, tri.y.position_index, tri.z.position_index)?;
                            }
                        };
                    },
                    (true, true) =>
                    {
                        let px = tri.x.position_index;
                        let py = tri.y.position_index;
                        let pz = tri.z.position_index;

                        match ((tri.x.texcoord_index, tri.y.texcoord_index, tri.y.texcoord_index), (tri.x.normal_index, tri.y.normal_index, tri.z.normal_index))
                        {
                            ((Some(tx), Some(ty), Some(tz)), (Some(nx), Some(ny), Some(nz))) =>
                            {
                                write_all(&mut writer, px, py, pz, tx, ty, tz, nx, ny, nz)?;
                            },
                            ((Some(tx), Some(ty), Some(tz)), _) =>
                            {
                                write_positions_and_texcoords(&mut writer, px, py, pz, tx, ty, tz)?;
                            },
                            (_, (Some(nx), Some(ny), Some(nz))) =>
                            {
                                write_positions_and_normals(&mut writer, px, py, pz, nx, ny, nz)?;
                            },
                            _ =>
                            {
                                write_positions(&mut writer, px, py, pz)?;
                            }
                        };
                    }
                };
            }
        }

        writer.flush()?;
        Ok(())
    }
}