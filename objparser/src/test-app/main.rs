
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let start = std::time::Instant::now();

    let result = objparser::obj::obj::load_obj("res/WoodenToyShapelab.obj", objparser::obj::obj::ObjParseFeatures::LOAD_VERTEX_NORMALS)?;

    let end = std::time::Instant::now();
    println!("done in {}ms", end.duration_since(start).as_millis());

    println!("vertices: {}", &result.positions.len());
    println!("texcoords: {}", if let Some(texcoords) = &result.texcoords { texcoords.len() } else { 0 });
    println!("normals: {}", if let Some(normals) = &result.normals { normals.len() } else { 0 });
    println!("indices: {}", &result.indices.len());

    result.export("res/exported.obj")?;

    Ok(())
}
