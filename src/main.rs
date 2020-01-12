use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use std::collections::VecDeque;

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

impl std::ops::Add for V2<f64> {
    type Output = V2<f64>;

    fn add(self, other: Self) -> Self {
        Self {
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
    rng: StdRng,
    paused: bool,
    collided: bool,
    heli_pos: V2<f64>,
    heli_vel: V2<f64>,
    tube: VecDeque<(V2<f64>, f64)>,
}

fn init_app_state(rng: StdRng) -> AppState {
    let tube: VecDeque<_> = [(V2::new(0.0, 0.5), 0.4), (V2::new(0.4, 0.5), 0.4)]
        .iter()
        .copied()
        .collect();

    let mut state = AppState {
        rng,
        paused: true,
        collided: false,
        heli_pos: V2::new(0.1, 0.5),
        heli_vel: V2::new(0.0, 0.0),
        tube,
    };
    move_tube(&mut state);
    state
}

impl AppState {
    fn ground<'a>(&'a self) -> impl Iterator<Item = V2<f64>> + 'a {
        self.tube.iter().map(|&(p, r)| p + V2::new(0.0, -r))
    }

    fn ceiling(&self) -> impl Iterator<Item = V2<f64>> + '_ {
        self.tube.iter().map(|&(p, r)| p + V2::new(0.0, r))
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
        t.set_blend_mode(sdl2::render::BlendMode::Add);
        t
    };
    let tex_explosion = {
        let mut t = texture_creator.load_texture("data/explosion.png")?;
        t.set_blend_mode(sdl2::render::BlendMode::Add);
        t
    };

    let mut state = init_app_state(StdRng::seed_from_u64(0));

    let mut render = |state: &AppState| {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let points: Vec<Point> = state
            .ground()
            .map(|V2 { x, y }| Point::new((x * 1024.0) as i32, ((1.0 - y) * 640.0) as i32))
            .collect();
        canvas.draw_lines(&points[..]).expect("Rendering error");
        let points: Vec<Point> = state
            .ceiling()
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
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => state.paused = !state.paused,

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => state = init_app_state(state.rng),

                _ => {}
            }
        }

        let keystate = event_pump.keyboard_state();
        let input_up = keystate.is_scancode_pressed(Scancode::Up)
            || keystate.is_scancode_pressed(Scancode::Space);

        if input_up {
            state.paused = false;
        }

        if !state.collided && !state.paused {
            state.heli_pos = state.heli_pos + state.heli_vel;

            if input_up {
                state.heli_vel.y += 0.0001;
            } else {
                state.heli_vel.y -= 0.0001;
            }

            move_tube(&mut state);

            state.collided = is_collided(&state);
        }

        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

fn move_tube(state: &mut AppState) {
    let rng = &mut state.rng;
    let tube = &mut state.tube;

    for (p, _) in tube.iter_mut() {
        p.x -= 0.001;
    }

    while tube.get(1).filter(|(p, _)| p.x < 0.0).is_some() {
        tube.pop_front();
    }

    while tube.back().filter(|(p, _)| p.x >= 1.0).is_none() {
        let new_x = tube.back().map_or(0.0, |(p, _)| p.x + 1.0 / 5.0);
        tube.push_back((
            V2::new(new_x, rng.gen_range(0.2, 0.8)),
            rng.gen_range(0.1, 0.2),
        ));
    }
}

fn segment_point_distance((seg_start, seg_end): (V2<f64>, V2<f64>), point: V2<f64>) -> f64 {
    (seg_end - seg_start)
        .turn_left()
        .normalized()
        .dot(point - seg_start)
}

fn is_collided(state: &AppState) -> bool {
    const HELI_RADIUS: f64 = 0.03;
    fn between((start, end): (f64, f64), x: f64) -> bool {
        x > start && x < end
    }
    fn max(x: f64, y: f64) -> f64 {
        if x > y {
            x
        } else {
            y
        }
    }
    fn min(x: f64, y: f64) -> f64 {
        if x < y {
            x
        } else {
            y
        }
    }

    // TODO: Do proper circle-polygon intersection
    let pos = state.heli_pos;
    let hit_ground = state
        .ground()
        .zip(state.ground().skip(1))
        .any(|(start, end)| {
            between((start.x, end.x), pos.x)
                && pos.y - HELI_RADIUS < max(start.y, end.y)
                && segment_point_distance((start, end), pos) < HELI_RADIUS
        });
    let hit_ceiling = state
        .ceiling()
        .zip(state.ceiling().skip(1))
        .any(|(start, end)| {
            between((start.x, end.x), pos.x)
                && pos.y + HELI_RADIUS > min(start.y, end.y)
                && segment_point_distance((end, start), pos) < HELI_RADIUS
        });

    hit_ground || hit_ceiling
}
