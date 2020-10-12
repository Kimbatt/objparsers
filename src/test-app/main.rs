
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let start = std::time::Instant::now();

    mfp::obj::obj::load_obj("res/WoodenToyShapelab.obj", mfp::obj::obj::ObjParseFeatures::NONE)?;

    let end = std::time::Instant::now();
    println!("done in {}ms", end.duration_since(start).as_millis());

    Ok(())
}