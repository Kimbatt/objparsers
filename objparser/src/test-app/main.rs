
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    for file in  std::fs::read_dir(
        // "res/testmodels/common-3d-test-models/"
        // "res/testmodels/assimp-test-models/"
        // "res/testexport/"
        "res/"
    )?
    {
        let file_path = file?.path();
        if let Some(ext) = file_path.extension()
        {
            if let Some(ext_str) = ext.to_str()
            {
                if ext_str.ne("obj")
                {
                    continue;
                }
            }
        }
        else
        {
            continue;
        }

        let file_path_str = file_path.to_str().ok_or("Path error")?;
        let start = std::time::Instant::now();

        println!("loading model: {}", file_path_str);
        let result = match objparser::obj::obj::load_obj(file_path_str,
            objparser::obj::obj::ObjParseFeatures::LOAD_ALL)
        {
            Ok(res) => res,
            Err(err) =>
            {
                println!("{}", (*err).to_string());
                continue;
            }
        };

        let end = std::time::Instant::now();
        println!("done in {}ms", end.duration_since(start).as_millis());

        println!("objects: {}", result.objects.len());
        println!("vertices: {}", &result.vertex_buffer.len());
        println!("texcoords: {}", if let Some(texcoords) = &result.texcoord_buffer { texcoords.len() } else { 0 });
        println!("normals: {}", if let Some(normals) = &result.normal_buffer { normals.len() } else { 0 });

        println!("indices: {}", result.objects.iter().map(|object| object.indices.len()).sum::<usize>());

        result.export(format!("res/testexport/{}",
            std::path::Path::new(file_path_str).file_name()
            .ok_or("Path error")?
            .to_str()
            .ok_or("Path error")?).as_str())?;
    }


    Ok(())
}
