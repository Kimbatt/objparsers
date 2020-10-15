
use super::obj::*;
use std::io::prelude::*;

impl ObjParseResult 
{
    pub fn export(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut writer = std::io::BufWriter::new(std::fs::File::create(file_path)?);

        for vertex_pos in self.positions.iter()
        {
            writer.write_fmt(format_args!("v {} {} {}\n", vertex_pos.0, vertex_pos.1, vertex_pos.2))?;
        }

        let has_texcoords = self.texcoords.is_some();
        let has_normals = self.normals.is_some();

        if let Some(texcoords) = &self.texcoords
        {
            for texcoords in texcoords.iter()
            {
                writer.write_fmt(format_args!("vt {} {}\n", texcoords.0, texcoords.1))?;
            }
        }

        if let Some(normals) = &self.normals
        {
            for normals in normals.iter()
            {
                writer.write_fmt(format_args!("vn {} {} {}\n", normals.0, normals.1, normals.2))?;
            }
        }


        for i in (0..self.indices.len()).step_by(3)
        {
            let idx0 = self.indices[i] + 1;
            let idx1 = self.indices[i + 2] + 1;
            let idx2 = self.indices[i + 1] + 1;
            match (has_texcoords, has_normals)
            {
                (false, false) =>
                {
                    writer.write_fmt(format_args!("f {} {} {}\n", idx0, idx1, idx2))?;
                },
                (true, false) =>
                {
                    writer.write_fmt(format_args!("f {}/{} {}/{} {}/{}\n", idx0, idx0, idx1, idx1, idx2, idx2))?;
                },
                (false, true) =>
                {
                    writer.write_fmt(format_args!("f {}//{} {}//{} {}//{}\n", idx0, idx0, idx1, idx1, idx2, idx2))?;
                },
                (true, true) =>
                {
                    writer.write_fmt(format_args!("f {}/{}/{} {}/{}/{} {}/{}/{}\n", idx0, idx0, idx0, idx1, idx1, idx1, idx2, idx2, idx2))?;
                }
            };
        }

        writer.flush()?;
        Ok(())
    }
}