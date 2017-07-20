#[macro_use] extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

extern crate gammaray;
use gammaray::core;
use gammaray::prim;
use gammaray::prim::Prim;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        st: [f32; 2] = "a_St",
    }

    constant Transform {
        window_frame: [f32; 4] = "u_WindowFrame",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

const SQUARE: [Vertex; 6] = [
    Vertex { pos: [1.0, -1.0], st: [1.0, 0.0] },
    Vertex { pos: [-1.0, -1.0], st: [0.0, 0.0] },
    Vertex { pos: [-1.0, 1.0], st: [0.0, 1.0] },
    Vertex { pos: [1.0, -1.0], st: [1.0, 0.0] },
    Vertex { pos: [-1.0, 1.0], st: [0.0, 1.0] },
    Vertex { pos: [1.0, 1.0], st: [1.0, 1.0] }
];

extern crate image;
use image::ImageBuffer;

extern crate time;
extern crate rayon;
use rayon::prelude::*;

fn gfx_load_texture<F, R>(factory: &mut F) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    where F: gfx::Factory<R>,
          R: gfx::Resources
{
    let c = core::Camera::default();
    println!("Aspect: {}", c.aspect_ratio());
    println!("(-0.22, -0.16, -1.0).normalized {}", core::Vec::new(-0.22, -0.16, -1.0).normalized());
    println!("Window: {:?}", c.window_max());
    println!("Ray -1 -1: {}", c.compute_ray(-1.0, -1.0));
    println!("Ray 1 -1: {}", c.compute_ray(1.0, -1.0));

    let xform_s = core::Mat::translation(&core::Vec::new(0.0, 0.0, -1000.0));
    let s = prim::Sphere::new(&xform_s, 5.0);

    let starts = time::now();
    let mut xx = [0; 512];
    for x in 0..512 {
        xx[x] = x;
    }
    let yy = xx.par_iter().for_each(|&x| {
        for y in 0..512 {
            let ii = core::lerp(-1.0, 1.0, (x as f64 / 511.0));
            let jj = core::lerp(-1.0, 1.0, (y as f64 / 511.0));
            let r = c.compute_ray(ii, jj);
            s.intersect_world(&r);
        }
    });
    let ends = time::now();
    println!("Duration B {:?}", ends - starts);

    use gfx::format::Rgba8;
    let start = time::now();
    let img = ImageBuffer::from_fn(512, 512, |x, y| {
        let ii = core::lerp(-1.0, 1.0, (x as f64 / 511.0));
        let jj = core::lerp(-1.0, 1.0, (y as f64 / 511.0));
        let r = c.compute_ray(ii, jj);
        match s.intersect_world(&r) {
            Some(_) => image::Rgba([255u8, 255u8, 255u8, 255u8]),
            None => image::Rgba([0u8, 0u8, 0u8, 255u8]),
        }
    });
    let end = time::now();
    println!("Duration {:?}", end - start);
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as gfx::texture::Size, height as gfx::texture::Size, gfx::texture::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&img]).unwrap();
    view
}

pub fn main() {
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Square Toy".to_string())
        .with_dimensions(800, 800)
        .with_vsync();
    let (window, mut device, mut factory, color_view, mut depth_view) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex.glsl")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment.glsl")),
        pipe::new()
    ).unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, ());
    let sampler = factory.create_sampler_linear();
    let texture = gfx_load_texture(&mut factory);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: factory.create_constant_buffer(1),
        tex: (texture, sampler),
        out: color_view,
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            use glutin::WindowEvent::*;
            match event {
                KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _)
                | Closed => running = false,
                Resized(_, _) => {
                    gfx_glutin::update_views(&window, &mut data.out, &mut depth_view);
                },
                _ => (),
            }
        });

        let (in_width, in_height) = match window.get_inner_size_pixels() {
            None => (0, 0),
            Some((x, y)) => (x, y)
        };
        let (out_width, out_height) = match window.get_outer_size() {
            None => (0, 0),
            Some((x, y)) => (x, y)
        };
        let left_right_frame = out_width - in_width;
        let top_bottom_frame = out_height - in_height;

        let left = left_right_frame / 2;
        let right = left_right_frame - left;
        let bottom = left; // Assume that left = right = bottom for window frame.
        let top = top_bottom_frame - bottom;

        let transform = Transform {
            window_frame: [
                (left as f32) / (out_width as f32),
                (top as f32) / (out_height as f32),
                (right as f32) / (out_width as f32),
                (bottom as f32) / (out_height as f32)]
        };
        encoder.update_buffer(&data.transform, &[transform], 0)
                .expect("Couldn't update window frame size.");

        encoder.clear(&data.out, BLACK);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
