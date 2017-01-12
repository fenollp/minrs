#[macro_use]
extern crate glium;

extern crate rustc_serialize;
extern crate docopt;

use glium::backend::Facade;

const NAME: &'static str = "minrs";

const USAGE: &'static str = r#"
I kept dreaming of a world I thought I'd never see

Usage:
  minrs <file>
  minrs (-h | --help)
  minrs --version

Options:
  -v, --verbose  Show debug info on stdout.
  -h, --help     Show this screen.
  --version      Show version.
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: String,
    flag_verbose: bool,
    flag_version: bool,
}

fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    match args {
        Args{flag_version: true, ..} =>
            println!(env!("CARGO_PKG_VERSION")),
        _ => {

            use glium::{DisplayBuild, Surface};
            let display = glium::glutin::WindowBuilder::new()
                .with_title(NAME)
                .with_vsync()
                .build_glium()
                .unwrap();
            let version = display.get_opengl_version();
            println!("OpenGL version {:?}", version);
            let (width, height) = display.get_context().get_framebuffer_dimensions();
            println!("{:?}x{:?} = {:?}", width, height, width * height);

            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
            }

            implement_vertex!(Vertex, position);

            let mut shape = vec![];
            let half_width = width as f32 / 2f32;
            let half_height = height as f32 / 2f32;
            for y in 0..height {
                for x in 0..width {
                    let xx = (x as f32 - half_width) / half_width;
                    let yy = (y as f32 - half_height) / half_height;
                    shape.push(Vertex{position: [xx, yy]});
                }
            }
            println!("shape size: {:?}", shape.len());

            let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
            // let texture = load_file_1d(&display, width, height, args.arg_file.as_str()).unwrap();
            // let texture = file_to_texture(&display, width, height, args.arg_file.as_str()).unwrap();
            // let texture = file_to_texture2d(&display, width, height, args.arg_file.as_str()).unwrap();
            let texture = file_to_texture2d_(&display, width, height, args.arg_file.as_str()).unwrap();
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
            let dims = [texture.get_width() as f32,
                        texture.get_height().unwrap_or(1) as f32,
                        texture.get_depth().unwrap_or(1) as f32];
            let uniforms = uniform! {
                tex: &texture,
                window: dims,
            };

            let program = program!(&display,
                                   140 => {
                                       point_size: true,
                                       vertex: include_str!("vert_2d_140.glsl"),
                                       fragment: "
        #version 140

        // uniform sampler1D tex;
        uniform sampler2D tex;
        uniform vec3 window;

        in vec2 pos;
        out vec4 color;

        void main() {
            // float idx = pos.y * window.y + 1*(pos.x * window.x) / (window.x * window.y);

            // color = vec4(texture(tex, idx).rgb, 1);

            // vec4 r = texture(tex, idx + 0);
            // vec4 g = texture(tex, idx + 1);
            // vec4 b = texture(tex, idx + 2);
            // color = vec4(r.r, g.r, b.r, 1);

            vec4 c = texture(tex, pos);
            color = vec4(c.r, c.r, c.r, 1);
        }
        ",
                                   }).unwrap();

            loop {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
                target.finish().unwrap();

                for ev in display.poll_events() {
                    use glium::glutin::VirtualKeyCode;
                    match ev {
                        glium::glutin::Event::Closed => return,
                        glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(VirtualKeyCode::Escape)) => return,
                        _ => ()
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum LoadError {
    Io(std::io::Error),
    Gl(glium::texture::TextureCreationError),
}

fn load_file_1d<F: ?Sized>(display: &F, width: u32, height: u32, path: &str)
                           -> Result<glium::texture::Texture1d, LoadError>
    where F: Facade + std::marker::Sized
{
    let read_bytes = std::cmp::min(8192, width * height);
    println!("trying to read {:?} of {:?}", read_bytes, path);

    let mut buffer = Vec::new();
    let f = try!(std::fs::File::open(path).map_err(LoadError::Io));
    let mut chunk = f.take(read_bytes as u64);
    use std::io::Read;
    let bytes_read = try!(chunk.read_to_end(&mut buffer).map_err(LoadError::Io)) as u32;
    println!("read {:?}", bytes_read);

    make_1d_texture(display, buffer)
}

fn make_1d_texture<F: ?Sized>(display: &F, buffer: std::vec::Vec<u8>)
                              -> Result<glium::texture::Texture1d, LoadError>
    where F: Facade + std::marker::Sized
{
    let side = 3;
    let mut buffers: Vec<(u8, u8, u8)> = vec![];
    for slice in buffer.chunks(side) {
        let vec = slice.to_vec();
        if vec.len() == side {
            buffers.push((vec[0], vec[1], vec[2]));
        }
    }

    use glium::texture::Texture1d;
    let texture = try!(Texture1d::with_format(display, buffer,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                              glium::texture::MipmapsOption::NoMipmap)
                       .map_err(LoadError::Gl));

    println!("texture info: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}"
             ,texture.get_width()
             ,texture.get_height()
             ,texture.get_depth()
             ,texture.kind()
             ,texture.get_texture_type()
             ,texture.get_mipmap_levels()
             ,texture.get_samples()
             ,texture.get_internal_format()
            );
    Ok(texture)
}


fn file_to_texture<F: ?Sized>(display: &F, width: u32, height: u32, path: &str) ->
    Result<glium::texture::DepthTexture1d, LoadError>
    where F: Facade + std::marker::Sized
{
    let read_bytes = std::cmp::min(8192, width * height);
    println!("trying to read {:?} of {:?}", read_bytes, path);

    let mut buffer: Vec<f32> = Vec::with_capacity(read_bytes as usize);
    let f = try!(std::fs::File::open(path).map_err(LoadError::Io));
    use std::io::Read;
    for byte in f.take(read_bytes as u64).bytes() {
        buffer.push((byte.unwrap() as f32) / 255f32);
    }

    make_depth1d_texture(display, buffer)
}

fn make_depth1d_texture<F: ?Sized>(display: &F, buffer: std::vec::Vec<f32>) ->
    Result<glium::texture::DepthTexture1d, LoadError>
    where F: Facade + std::marker::Sized
{
    let texture = try!(glium::texture::DepthTexture1d::new(display, buffer)
                       .map_err(LoadError::Gl));

    println!("texture info: {:?} {:?} {:?} {:?} {:?} {:?}"
             ,texture.get_width()
             ,texture.get_height()
             ,texture.get_depth()
             ,texture.kind()
             ,texture.get_texture_type()
             ,texture.get_mipmap_levels()
            );
    Ok(texture)
}

fn file_to_texture2d<F: ?Sized>(display: &F, width: u32, height: u32, path: &str) ->
    Result<glium::texture::DepthTexture2d, LoadError>
    where F: Facade + std::marker::Sized
{
    let read_bytes = std::cmp::min(8192, width * height);
    println!("trying to read {:?} of {:?}", read_bytes, path);

    let mut buffer: Vec<f32> = Vec::with_capacity(read_bytes as usize);
    let f = try!(std::fs::File::open(path).map_err(LoadError::Io));
    use std::io::Read;
    for byte in f.take(read_bytes as u64).bytes() {
        buffer.push((byte.unwrap() as f32) / 255f32);
    }

    make_depth2d_texture(display, buffer)
}

fn make_depth2d_texture<F: ?Sized>(display: &F, buffer: std::vec::Vec<f32>) ->
    Result<glium::texture::DepthTexture2d, LoadError>
    where F: Facade + std::marker::Sized
{
    // let side = (buffer.len() as f64).sqrt() as usize;
    let side = buffer.len() / 64;
    // let side = 4;
    let mut buffers: Vec<Vec<f32>> = vec![];
    for slice in buffer.chunks(side) {
        let vec = slice.to_vec();
        if vec.len() == side {
            buffers.push(vec);
        }
    }
    let texture = try!(glium::texture::DepthTexture2d::new(display, buffers)
                       .map_err(LoadError::Gl));

    println!("texture info: {:?} {:?} {:?} {:?} {:?} {:?}"
             ,texture.get_width()
             ,texture.get_height()
             ,texture.get_depth()
             ,texture.kind()
             ,texture.get_texture_type()
             ,texture.get_mipmap_levels()
            );
    Ok(texture)
}

fn file_to_texture2d_<F: ?Sized>(display: &F, width: u32, height: u32, path: &str) ->
    Result<glium::texture::DepthTexture2d, LoadError>
    where F: Facade + std::marker::Sized
{
    let read_bytes = width * height;//std::cmp::min(8192, width * height);
    println!("trying to read {:?} of {:?}", read_bytes, path);

    use std::io::Read;
    let f = try!(std::fs::File::open(path).map_err(LoadError::Io));
    let mut handle = f.take(read_bytes as u64);
    let mut buffer: Vec<u8> = vec![];
    let bytes_read = try!(handle.read_to_end(&mut buffer).map_err(LoadError::Io));
    println!("read {:?}", bytes_read);
    let mut buffers: Vec<Vec<f32>> = vec![];
    let side = width as usize;
    for slice in buffer.chunks(side) {
        if slice.len() == side {
            let vec = slice.into_iter().map(|f| *f as f32 / 255f32).collect();
            buffers.push(vec);
        }
    }

    let texture = try!(glium::texture::DepthTexture2d::with_format(display, buffers,
                                                                   glium::texture::DepthFormat::F32,
                                                                   glium::texture::MipmapsOption::NoMipmap)
                       .map_err(LoadError::Gl));

    println!("texture info: {:?} {:?} {:?} {:?} {:?} {:?}"
             ,texture.get_width()
             ,texture.get_height()
             ,texture.get_depth()
             ,texture.kind()
             ,texture.get_texture_type()
             ,texture.get_mipmap_levels()
            );
    Ok(texture)
}
