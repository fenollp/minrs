#[macro_use]
extern crate glium;

extern crate rustc_serialize;
extern crate docopt;

use glium::backend::Facade;

const NAME: &'static str = "4 colours";

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
    let texture = file_to_texture2d(&display, width, height, args.arg_file.as_str()).unwrap();
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
                               vertex: include_str!("../src/vert_2d_140.glsl"),
                               fragment: include_str!("../src/frag_four_2d_140.glsl")
                           }).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(Escape)) => return,
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

fn file_to_texture2d<F: ?Sized>(display: &F, width: u32, height: u32, path: &str) ->
    Result<glium::texture::DepthTexture2d, LoadError>
    where F: Facade + std::marker::Sized
{
    let read_bytes = width * height;
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
            let vec = slice.into_iter().map(|f| *f as f32 / 255f32).collect(); // weird deref here causes need for mut handle for whatever reason
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
