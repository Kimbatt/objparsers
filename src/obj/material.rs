
pub struct Color
{
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Default for Color
{
    fn default() -> Self
    {
        Self { r: 1.0, g: 1.0, b: 1.0 }
    }
}

pub struct ObjTexture
{
    pub data: Vec<u8>
}

impl ObjTexture
{
    pub fn from_file_path(path: &str) -> Result<ObjTexture, Box<dyn std::error::Error>>
    {
        Ok(ObjTexture { data: std::fs::read(path)? })
    }
}

pub struct ObjMaterial
{
    pub name: String,
    pub alpha: f32,

    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub specular_exponent: f32,

    pub ambient_texture: Option<ObjTexture>,
    pub diffuse_texture: Option<ObjTexture>,
    pub bump_map: Option<ObjTexture>
}

impl ObjMaterial
{
    pub fn default_name() -> String
    {
        "Default".into()
    }

    pub fn new(name: String) -> ObjMaterial
    {
        ObjMaterial
        {
            name,
            alpha: 1.0,
            ambient_color: Default::default(),
            diffuse_color: Default::default(),
            specular_color: Default::default(),
            specular_exponent: 1.0,
            ambient_texture: None,
            diffuse_texture: None,
            bump_map: None,
        }
    }
}