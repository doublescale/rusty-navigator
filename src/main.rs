use rand::SeedableRng;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
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

#[derive(Clone, Copy)]
struct V2<T> {
    x: T,
    y: T,
}

impl<T> V2<T> {
    fn new(x: T, y: T) -> Self {
        V2 { x, y }
    }
}

impl V2<f64> {
    fn normalized(self) -> Self {
        let V2 { x, y } = self;
        let norm = x * x + y * y;
        V2 {
            x: x / norm,
            y: y / norm,
        }
    }

    fn turn_left(self) -> Self {
        V2 {
            x: -self.y,
            y: self.x,
        }
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl<T> std::ops::Add<&V2<T>> for V2<T>
where
    T: std::ops::Add + Copy,
{
    type Output = V2<<T as std::ops::Add>::Output>;

    fn add(self, other: &Self) -> V2<<T as std::ops::Add>::Output> {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for V2<f64> {
    type Output = V2<f64>;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

struct AppState {
    collided: bool,
    heli_pos: V2<f64>,
    heli_vel: V2<f64>,
    ground: Vec<V2<f64>>,
    ceiling: Vec<V2<f64>>,
}

fn init_app_state<T>(rng: &mut T) -> AppState
where
    T: rand::Rng,
{
    let ground: Vec<_> = (0..=5)
        .map(|i| V2::new(i as f64 / 5.0, rng.gen_range(0.1, 0.4)))
        .collect();

    let ceiling: Vec<_> = (0..=5)
        .map(|i| V2::new(i as f64 / 5.0, rng.gen_range(0.6, 0.9)))
        .collect();

    AppState {
        collided: false,
        heli_pos: V2::new(0.1, 0.5),
        heli_vel: V2::new(0.001, 0.0),
        ground,
        ceiling,
    }
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
    let tex_heli = {
        let mut t = texture_creator.load_texture("data/heli.png")?;
        // t.set_color_mod(0, 255, 0);
        t.set_blend_mode(sdl2::render::BlendMode::Add);
        t
    };
    let tex_explosion = {
        let mut t = texture_creator.load_texture("data/explosion.png")?;
        t.set_blend_mode(sdl2::render::BlendMode::Add);
        t
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let mut state = init_app_state(&mut rng);

    let mut render = |state: &AppState| {
        if opts.debug {
            println!("Rendering");
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let points: Vec<Point> = state
            .ground
            .iter()
            .map(|V2 { x, y }| Point::new((x * 1024.0) as i32, ((1.0 - y) * 640.0) as i32))
            .collect();
        canvas.draw_lines(&points[..]).expect("Rendering error");
        let points: Vec<Point> = state
            .ceiling
            .iter()
            .map(|V2 { x, y }| Point::new((x * 1024.0) as i32, ((1.0 - y) * 640.0) as i32))
            .collect();
        canvas.draw_lines(&points[..]).expect("Rendering error");

        if state.collided {
            canvas.copy(
                &tex_explosion,
                None,
                Some(Rect::new(
                    (state.heli_pos.x * 1024.0) as i32 - 32,
                    ((1.0 - state.heli_pos.y) * 640.0) as i32 - 32,
                    64,
                    64,
                )),
            )
        } else {
            canvas.copy(
                &tex_heli,
                None,
                Some(Rect::new(
                    (state.heli_pos.x * 1024.0) as i32 - 32,
                    ((1.0 - state.heli_pos.y) * 640.0) as i32 - 12,
                    64,
                    24,
                )),
            )
        }
        .expect("Rendering error");

        canvas.present();
    };

    'running: loop {
        render(&state);

        for ev in event_pump.poll_iter() {
            if opts.debug {
                println!("{:?}", ev);
            }

            match ev {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => state = init_app_state(&mut rng),

                _ => {}
            }
        }

        if !state.collided {
            state.heli_pos = state.heli_pos + &state.heli_vel;

            let keystate = event_pump.keyboard_state();
            if keystate.is_scancode_pressed(Scancode::Up)
                || keystate.is_scancode_pressed(Scancode::Space)
            {
                state.heli_vel.y += 0.0001;
            } else {
                state.heli_vel.y -= 0.0001;
            }

            state.collided = is_collided(&state);
        }

        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

fn segment_point_distance((seg_start, seg_end): (V2<f64>, V2<f64>), point: V2<f64>) -> f64 {
    (seg_end - seg_start)
        .turn_left()
        .normalized()
        .dot(point - seg_start)
}

fn is_collided(state: &AppState) -> bool {
    fn between((start, end): (f64, f64), x: f64) -> bool {
        x >= start && x <= end
    }
    const HELI_RADIUS: f64 = 0.05;

    let hit_ground = state
        .ground
        .iter()
        .zip(state.ground.iter().skip(1))
        .any(|(start, end)| {
            between((start.x, end.x), state.heli_pos.x)
                && segment_point_distance((*start, *end), state.heli_pos) < HELI_RADIUS
        });
    let hit_ceiling = state
        .ceiling
        .iter()
        .zip(state.ceiling.iter().skip(1))
        .any(|(start, end)| {
            between((start.x, end.x), state.heli_pos.x)
                && segment_point_distance((*end, *start), state.heli_pos) < HELI_RADIUS
        });

    hit_ground || hit_ceiling
}
