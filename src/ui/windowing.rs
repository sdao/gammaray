use ui::sync;

use gfx;
use gfx::format::Rgba8;
use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin;
use glutin;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        st: [f32; 2] = "a_St",
    }

    constant Transform {
        window_size: [f32; 2] = "u_WindowSize",
        image_size: [f32; 2] = "u_ImageSize",
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

trait FactoryWindowingExt<R: gfx::Resources>: gfx::Factory<R> {
    fn create_texture_mutable_u8<T: gfx::format::TextureFormat>(
        &mut self, kind: gfx::texture::Kind)
        -> Result<
            (gfx::handle::Texture<R, T::Surface>, gfx::handle::ShaderResourceView<R, T::View>,
                gfx::texture::Info),
            gfx::CombinedError>
    {
        let surface = <T::Surface as gfx::format::SurfaceTyped>::get_surface_type();
        let desc = gfx::texture::Info {
            kind: kind,
            levels: 1u8,
            format: surface,
            bind: gfx::memory::SHADER_RESOURCE,
            usage: gfx::memory::Usage::Dynamic,
        };
        let cty = <T::Channel as gfx::format::ChannelTyped>::get_channel_type();
        let raw = try!(self.create_texture_raw(desc, Some(cty), None));
        let levels = (0, raw.get_info().levels - 1);
        let tex = gfx::memory::Typed::new(raw);
        let view = try!(self.view_texture_as_shader_resource::<T>(
                &tex, levels, gfx::format::Swizzle::new()));
        Ok((tex, view, desc))
    }
}

impl<R: gfx::Resources, F: gfx::Factory<R>> FactoryWindowingExt<R> for F {}

pub fn image_preview_window(shared_data: sync::SharedData)
{
    let (width, height) = (shared_data.width as u32, shared_data.height as u32);

    // Initialize window.
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Image Preview".to_string())
        .with_dimensions(width, height)
        .with_vsync();
    let (window, mut device, mut factory, color_view, mut depth_view) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);

    // Setup pipeline.
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vertex.glsl")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/fragment.glsl")),
        pipe::new()
    ).unwrap();

    // Send buffers and uniform data.
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, ());
    let sampler = factory.create_sampler_linear();
    let kind = gfx::texture::Kind::D2(
                width as gfx::texture::Size,
                height as gfx::texture::Size,
                gfx::texture::AaMode::Single);
    let (tex, tex_view, tex_desc) = factory.create_texture_mutable_u8::<Rgba8>(kind).unwrap();
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: factory.create_constant_buffer(1),
        tex: (tex_view, sampler),
        out: color_view,
    };

    // Main loop.
    let mut running = true;
    while running {
        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            use glutin::WindowEvent::*;
            match event {
                KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _) | Closed => {
                    running = false;
                },
                Resized(_, _) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut depth_view);
                },
                _ => (),
            }
        });

        match shared_data.load() {
            Some(guard) => {
                let info = tex_desc.to_image_info(0u8);
                encoder.update_texture::<gfx::format::R8_G8_B8_A8, Rgba8>(
                        &tex, None, info, &guard.get()).unwrap();
            },
            None => {}
        }

        let (in_width, in_height) = match window.get_inner_size_pixels() {
            None => (0, 0),
            Some((x, y)) => (x, y)
        };
        let transform = Transform {
            window_size: [in_width as f32, in_height as f32],
            image_size: [width as f32, height as f32],
        };
        encoder.update_buffer(&data.transform, &[transform], 0).unwrap();

        encoder.clear(&data.out, BLACK);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
