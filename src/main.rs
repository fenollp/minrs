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
}

fn main() {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

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
    let texture = file_to_texture2d(&display, width, height, args.arg_file.as_str()).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
    let uniforms = uniform! {
        tex: &texture,
        window: [width as f32, height as f32, 0f32],
    };

    let program = program!(&display,
                           140 => {
                               point_size: true,
                               vertex: "
        #version 140

        uniform vec3 window;

        in vec2 position;
        out vec2 pos;

        void main() {
            gl_PointSize = 1;
            gl_Position = vec4(position, 0, 1);
            pos = position;
        }
        ",
                               fragment: "
        #version 140

        // uniform sampler1D tex;
        uniform sampler2D tex;
        uniform vec3 window;

        in vec2 pos;
        out vec4 color;

        void main() {
            // float idx = pos.x + pos.y * window.y;
            // vec4 r = texture(tex, idx + 0);
            // vec4 g = texture(tex, idx + 1);
            // vec4 b = texture(tex, idx + 2);
            // color = vec4(r.x, g.x, b.x, 1);
            vec4 c = texture(tex, pos);
            color = vec4(c.rgb, 1);
        }
        ",
                           }).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, 9, Some(_)) => return,
                _ => ()
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
    let bytes_read = buffer.len() as u32;
    use glium::texture::pixel_buffer::PixelBuffer;
    let pixelbuffer = PixelBuffer::new_empty(display, bytes_read as usize);
    pixelbuffer.write(buffer.as_slice());
    println!("pixelbuffer size: {:?}", pixelbuffer.get_size());

    use glium::texture::Texture1d;
    let texture = try!(Texture1d::empty_with_format(display,
                                                    glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                                    glium::texture::MipmapsOption::NoMipmap,
                                                    bytes_read)
                       .map_err(LoadError::Gl));

    texture.main_level().raw_upload_from_pixel_buffer(pixelbuffer.as_slice(), 0..bytes_read, 0..1, 0..1);
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
    let side = (buffer.len() as f64).sqrt() as usize;
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
