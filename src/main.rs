use rand::Rng;
use rand::SeedableRng;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;

#[derive(Debug)]
struct AppOptions {
    debug: bool,
}

fn get_app_options() -> AppOptions {
    let args: Vec<_> = std::env::args().skip(1).collect::<Vec<_>>();
    let have = |s: &str| args.contains(&s.to_string());

    if have("-h") {
        println!(
            "Options:\n  \
             -d  Debug (show events)"
        );
        std::process::exit(0);
    }

    AppOptions { debug: have("-d") }
}

fn main() -> Result<(), String> {
    let opts = get_app_options();

    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;
    let window = video_subsystem
        .window("Rusty Navigator", 1024, 640)
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(1024, 640)
        .expect("Unable to set logical size");
    let texture_creator = canvas.texture_creator();
    let texture = {
        let mut t = texture_creator.load_texture("data/heli.png")?;
        // t.set_color_mod(0, 255, 0);
        t.set_blend_mode(sdl2::render::BlendMode::Add);
        t
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let points: Vec<Point> = (0..)
        .map(|i| Point::new(i * 128, rng.gen_range(400, 500)))
        .take_while(|p| p.x <= 1024)
        .collect();

    let mut render = || {
        if opts.debug {
            println!("Rendering");
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.draw_lines(&points[..]).expect("Rendering error");

        canvas
            .copy(&texture, None, Some(Rect::new(10, 200, 64, 24)))
            .expect("Rendering error");

        canvas.present();
    };

    for ev in event_pump.wait_iter() {
        if opts.debug {
            println!("{:?}", ev);
        }

        match ev {
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            }
            | Event::Quit { .. } => break,

            Event::Window {
                win_event: WindowEvent::Exposed,
                ..
            } => render(),

            _ => {}
        }
    }

    // std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
