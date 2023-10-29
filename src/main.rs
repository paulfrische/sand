use std::{collections::HashSet, error::Error};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 427;
const HEIGHT: u32 = 240;
const SCALE: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Particle {
    Empty,
    Static,
    Sand,
}

impl Particle {
    fn color(&self) -> [u8; 3] {
        match self {
            Particle::Empty => [92u8, 208u8, 224u8],
            Particle::Static => [99u8, 78u8, 28u8],
            Particle::Sand => [234u8, 195u8, 103u8],
        }
    }
}

struct World {
    grid: [Particle; (WIDTH * HEIGHT) as usize],
    movable: HashSet<(u32, u32)>,
    radius: u32,
}

impl World {
    fn new() -> Self {
        Self {
            grid: [Particle::Empty; (WIDTH * HEIGHT) as usize],
            movable: HashSet::new(),
            radius: 10,
        }
    }

    fn get(&self, x: u32, y: u32) -> Particle {
        return self.grid[(y * WIDTH + x) as usize];
    }

    fn set(&mut self, x: u32, y: u32, particle: Particle) {
        self.grid[(y * WIDTH + x) as usize] = particle;
    }

    fn update(&mut self) {
        for (x, y) in self.movable.clone().iter() {
            if 0 == *x || *x >= WIDTH - 1 || *y >= HEIGHT - 1 {
                continue;
            }

            let mut to_go = None;
            if self.get(*x, *y + 1) == Particle::Empty {
                to_go = Some((*x, *y + 1))
            } else if self.get(*x - 1, *y + 1) == Particle::Empty {
                to_go = Some((*x - 1, *y + 1))
            } else if self.get(*x + 1, *y + 1) == Particle::Empty {
                to_go = Some((*x + 1, *y + 1))
            }

            if let Some(p) = to_go {
                self.set(*x, *y, Particle::Empty);
                self.set(p.0, p.1, Particle::Sand);
                self.movable.remove(&(*x, *y));
                self.movable.insert(p);
            }
        }
    }

    fn input(&mut self, input: &mut WinitInputHelper) {
        if let Some((mx, my)) = input.mouse() {
            let (x, y) = Self::px_to_grid(mx, my);
            if input.mouse_held(0) {
                self.add(x, y, Particle::Sand);
            } else if input.mouse_held(1) {
                self.add(x, y, Particle::Static);
            } else if input.mouse_held(2) {
                self.add(x, y, Particle::Empty);
            }
        }

        self.radius = f32::max(self.radius as f32 + input.scroll_diff(), 0.0) as u32;
    }

    fn add(&mut self, mx: u32, my: u32, particle: Particle) {
        let lower_x = if mx < self.radius {
            0
        } else {
            mx - self.radius
        };
        let upper_x = if mx + self.radius >= WIDTH {
            WIDTH - 1
        } else {
            mx + self.radius
        };

        let lower_y = if my < self.radius {
            0
        } else {
            my - self.radius
        };
        let upper_y = if my + self.radius >= HEIGHT {
            HEIGHT - 1
        } else {
            my + self.radius
        };

        for x in lower_x..upper_x {
            for y in lower_y..upper_y {
                self.set(x, y, particle);
                if particle == Particle::Sand {
                    self.movable.insert((x, y));
                }
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, p) in self.grid.iter().enumerate() {
            let color = p.color();
            frame[i * 4 + 0] = color[0];
            frame[i * 4 + 1] = color[1];
            frame[i * 4 + 2] = color[2];
            frame[i * 4 + 3] = 255u8;
        }
    }

    fn px_to_grid(x: f32, y: f32) -> (u32, u32) {
        (x as u32 / SCALE, y as u32 / SCALE)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new((WIDTH * SCALE) as f64, (HEIGHT * SCALE) as f64);
        WindowBuilder::new()
            .with_title("sand")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(e) = pixels.render() {
                eprintln!("{:?}", e);
                *control_flow = ControlFlow::Exit;
                return;
            }
            world.update();
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            world.input(&mut input)
        }

        window.request_redraw();
    });
}
