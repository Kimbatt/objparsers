
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    for file in  std::fs::read_dir("res/testmodels/common-3d-test-models/")?
    {
        let file_path = file?.path();
        let file_path_str = file_path.to_str().ok_or("Path error")?;
        let start = std::time::Instant::now();
    
        let result = objparser::obj::obj::load_obj(file_path_str,
            objparser::obj::obj::ObjParseFeatures::NONE)?;
    
        let end = std::time::Instant::now();
        println!("done in {}ms", end.duration_since(start).as_millis());
    
        println!("vertices: {}", &result.positions.len());
        println!("texcoords: {}", if let Some(texcoords) = &result.texcoords { texcoords.len() } else { 0 });
        println!("normals: {}", if let Some(normals) = &result.normals { normals.len() } else { 0 });
        println!("indices: {}", &result.indices.len());
    
        result.export(format!("res/testexport/{}",
            std::path::Path::new(file_path_str).file_name()
            .ok_or("Path error")?
            .to_str()
            .ok_or("Path error")?).as_str())?;
    }


    Ok(())
}
