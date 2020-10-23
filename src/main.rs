use glium::{glutin, Surface, implement_vertex, uniform};
use glutin::platform::unix::WindowBuilderExtUnix;
use glutin::dpi::{Size, LogicalSize, Position, LogicalPosition};
use std::str::FromStr;
use cgmath::{
    Matrix4 as Matrix,
    Vector3 as Vector,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Vertex {
        return Vertex {
            position: [x, y],
        }
    }
}

implement_vertex!(Vertex, position);

type Vector2<T> = (T, T);

type Error = Box<dyn std::error::Error>;

pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
}

fn compute_window_bounds(desktop_size: Vector2<f64>, anchor: Anchor, gap_v: Vector2<f64>, gap_h: Vector2<f64>, size: f64)
    -> (Vector2<f64>, Vector2<f64>) {
    let position_x = match anchor {
        Anchor::Top | Anchor::Bottom | Anchor::Left => gap_h.0,
        Anchor::Right => desktop_size.0 - gap_h.1 - size,
    };

    let position_y = match anchor {
        Anchor::Bottom => desktop_size.1 - gap_v.1 - size,
        Anchor::Top | Anchor::Right | Anchor::Left => gap_v.0,
    };

    let size_y = match anchor {
        Anchor::Top | Anchor::Bottom => size,
        Anchor::Left | Anchor::Right => desktop_size.1 - gap_v.0 - gap_v.1,
    };

    let size_x = match anchor {
        Anchor::Top | Anchor::Bottom => desktop_size.0 - gap_h.0 - gap_h.1,
        Anchor::Left | Anchor::Right => size,
    };


    ((position_x, position_y), (size_x, size_y))
}

#[derive(Debug)]
struct ColorParseError;

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Color parse error, colors mut be #rrggbb or #rrggbbaa")
    }
}

impl std::error::Error for ColorParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: f32,
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
        if hex_code.chars().nth(0).unwrap() != '#' {
            Err(ColorParseError)?;
        }

        let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;

        let alpha = if hex_code.len() > 7 {
            u8::from_str_radix(&hex_code[7..9], 16)? as f32 / 255.0
        } else {
            1.0
        };

        Ok(Color { r, g, b, a: alpha })
    }
}

impl Color {
    pub fn gl_red(&self) -> f32 {
        self.gl(self.r)
    }

    pub fn gl_green(&self) -> f32 {
        self.gl(self.g)
    }

    pub fn gl_blue(&self) -> f32 {
        self.gl(self.b)
    }

    pub fn gl_alpha(&self) -> f32 {
        self.a
    }

    fn gl(&self, color: u8) -> f32 {
        (color as f32 / 255.0) * self.a
    }
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let dpi = event_loop.primary_monitor().unwrap().scale_factor();
    let window_size = event_loop.primary_monitor().unwrap().size().to_logical(dpi);

    let (pos, size) = compute_window_bounds(
            (window_size.width, window_size.height),
            Anchor::Right,
            (0.0, 0.0), (0.0, 0.0),
            100.0
        );

    let wb = glutin::window::WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(Size::Logical(LogicalSize::new(size.0, size.1)))
        .with_x11_window_type(vec![glutin::platform::unix::XWindowType::Dock]);

    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    display.gl_window().window().set_outer_position(Position::Logical(LogicalPosition::new(pos.0, pos.1)));

    let background = Color::from_str("#00ff0001").unwrap();

    let rectangle = vec![
        Vertex::new(0., 0.),
        Vertex::new(1., 0.),
        Vertex::new(1., 1.),
        Vertex::new(0., 1.),
        Vertex::new(0., 0.),
    ];

    let rectangle_buffer = glium::VertexBuffer::new(&display, &rectangle).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let screenspace: [[f32; 4]; 4] = cgmath::ortho(
            0.0, window_size.width as f32,
            window_size.height as f32, 0.0,
            -1000.0, 1000.0
        ).into();

    let shape_matrix: [[f32; 4]; 4]
        = (Matrix::from_scale(100.) * Matrix::from_translation(Vector::new(10.0, 0.0, -100.0))).into();

    let uniforms = uniform! {
        matrix: screenspace,
        model: shape_matrix,
    };

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        out vec4 position_o;
        uniform mat4 matrix;
        uniform mat4 model;

        void main() {
            gl_Position = matrix * model * vec4(position, 0.0, 1.0);
            position_o = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;
        in vec4 position_o;

        void main() {
            color = vec4(0.5, 0.5, 0.5, 0.5);

            //if(position_o.x < -0.4 || position_o.x > 0.4 || position_o.y < -0.4 || position_o.y > 0.4)
            //{
            //    color = vec4(1, 1, 1, 1);
            //}
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let mut target = display.draw();
        target.clear_color(
            background.gl_red(),
            background.gl_green(),
            background.gl_blue(),
            background.gl_alpha(),
        );

        target.draw(&rectangle_buffer, &indices, &program, &uniforms,
            &Default::default()).unwrap();

        target.finish().unwrap();

        *control_flow = glutin::event_loop::ControlFlow::Wait;
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => *control_flow = glutin::event_loop::ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}
