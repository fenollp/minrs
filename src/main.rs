#[macro_use]
extern crate glium;

fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let shape = vec![
        Vertex{ position: [0.0, 0.0] },
        // Vertex{ position: [-0.5, -0.5] },
        // Vertex{ position: [ 0.0,  0.5] },
        // Vertex{ position: [ 0.5, -0.25] },
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let program = program!(&display,
                           140 => {
                               point_size: true,
                               vertex: "
        #version 140

        in vec2 position;

        void main() {
            gl_PointSize = 4;
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

    for (name, attribute) in program.attributes() {
        println!("Name: {} - Type: {:?}", name, attribute.ty);
    }
    println!("gl_PointSize activated: {:?}", program.uses_point_size());

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
