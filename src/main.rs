#[macro_use]
extern crate glium;

fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().with_title("minrs").build_glium().unwrap();
    use glium::backend::Facade;
    let (width, height) = display.get_context().get_framebuffer_dimensions();

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

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let program = program!(&display,
                           140 => {
                               point_size: true,
                               vertex: "
        #version 140

        in vec2 position;

        void main() {
            gl_PointSize = 1;
            gl_Position = vec4(position, 0.0, 1.0);
        }
        ",
                               fragment: "
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
        ",
                           }).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
