
extern crate objparser;

#[repr(C)]
pub struct ObjParserHandle
{
    result: objparser::obj::obj::ObjParseResult
}

#[no_mangle]
pub unsafe extern "C" fn parse_obj_from_file_path(file_path_utf8_bytes: *const u8, file_path_byte_count: u32) -> *const ObjParserHandle
{
    match std::panic::catch_unwind(|| parse_obj_from_file_path_internal(file_path_utf8_bytes, file_path_byte_count))
    {
        Ok(parse_result) =>
        {
            match parse_result
            {
                Ok(handle) => Box::into_raw(Box::new(handle)),
                Err(_) => std::ptr::null()
            }
        },
        Err(_) => std::ptr::null() // panic occured
    }
}

#[no_mangle]
pub unsafe extern "C" fn parse_obj(file_content_bytes: *const u8, file_content_byte_count: u32) -> *const ObjParserHandle
{
    let bytes = std::slice::from_raw_parts(file_content_bytes, file_content_byte_count as usize);
    match std::panic::catch_unwind(|| parse_obj_from_bytes_internal(bytes))
    {
        Ok(parse_result) =>
        {
            match parse_result
            {
                Ok(handle) => Box::into_raw(Box::new(handle)),
                Err(_) => std::ptr::null()
            }
        },
        Err(_) => std::ptr::null() // panic occured
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_vertex_count(handle: *const ObjParserHandle) -> u32
{
    match handle.as_ref()
    {
        None => 0,
        Some(handle) => handle.result.positions.len() as u32 / 3
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_vertex_positions(handle: *const ObjParserHandle) -> *const f32
{
    match handle.as_ref()
    {
        None => std::ptr::null(),
        Some(handle) => handle.result.positions.as_ptr()
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_index_count(handle: *const ObjParserHandle) -> u32
{
    match handle.as_ref()
    {
        None => 0,
        Some(handle) => handle.result.indices.len() as u32
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_indices(handle: *const ObjParserHandle) -> *const u32
{
    match handle.as_ref()
    {
        None => std::ptr::null(),
        Some(handle) => handle.result.indices.as_ptr()
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_handle(handle: *mut ObjParserHandle)
{
    // After calling this function, the raw pointer is owned by the resulting Box.
    // Specifically, the Box destructor will call the destructor of T and free the allocated memory.
    // https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw
    Box::from_raw(handle);
}


unsafe fn parse_obj_from_file_path_internal(file_path_utf8_bytes: *const u8, file_path_byte_count: u32) -> Result<ObjParserHandle, Box<dyn std::error::Error>>
{
    let file_path_bytes = std::slice::from_raw_parts(file_path_utf8_bytes, file_path_byte_count as usize);
    let file_path = std::str::from_utf8(file_path_bytes)?;

    let result = objparser::obj::obj::load_obj(file_path, objparser::obj::obj::ObjParseFeatures::NONE)?;

    Ok(ObjParserHandle { result })
}

unsafe fn parse_obj_from_bytes_internal(bytes: &[u8]) -> Result<ObjParserHandle, Box<dyn std::error::Error>>
{
    let result = objparser::obj::obj::load_obj_from_bytes(bytes, objparser::obj::obj::ObjParseFeatures::NONE)?;

    Ok(ObjParserHandle { result })
}
