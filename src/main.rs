extern crate opengl_graphics;
extern crate graphics;
extern crate glutin_window;
extern crate piston;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

pub struct Game {
    gl: GlGraphics,
    rows: u32,
    cols: u32,
    rake: Rake,
    just_eaten: bool,
    square_width: u32,
    food: Food,
    score: u32,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(BLACK, gl);
        });

        self.rake.render(args);
        self.food.render(&mut self.gl, args, self.square_width);
    }

    fn update(&mut self, _args: &UpdateArgs) -> bool {
        if !self.rake.update(self.just_eaten, self.cols, self.rows) {
            return false;
        }

        if self.just_eaten {
            self.score += 1;
            self.just_eaten = false;
        }

        self.just_eaten = self.food.update(&self.rake);
        if self.just_eaten {
            use rand::Rng;
            use rand::thread_rng;
            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0.. self.cols);
                let new_y = r.gen_range(0.. self.rows);
                if !self.rake.is_collide(new_x, new_y) {
                    self.food = Food { x: new_x, y: new_y };
                    break;
                }
            }
        }

        true
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.rake.d.clone();
        self.rake.d = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::DOWN => Direction::UP,
            &Button::Keyboard(Key::Down) if last_direction != Direction::UP => Direction::DOWN,
            &Button::Keyboard(Key::Left) if last_direction != Direction::RIGHT => Direction::LEFT,
            &Button::Keyboard(Key::Right) if last_direction != Direction::LEFT => Direction::RIGHT,
            _ => last_direction,
        };
    }
}

/// Rake direction
#[derive(Clone, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct Rake {
    gl: GlGraphics,
    rake_parts: LinkedList<Rake_Piece>,
    width: u32,
    d: Direction,
}

#[derive(Clone)]
pub struct Rake_Piece(u32, u32);

impl Rake {
    pub fn render(&mut self, args: &RenderArgs) {

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.rake_parts
            .iter()
            .map(|p| Rake_Piece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(GREEN, square, transform, gl));
        })
    }

    /// Move the rake if valid, otherwise returns false.
    pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_front: Rake_Piece =
            (*self.rake_parts.front().expect("No front of rake found.")).clone();

        if (self.d == Direction::UP && new_front.1 == 0)
            || (self.d == Direction::LEFT && new_front.0 == 0)
            || (self.d == Direction::DOWN && new_front.1 == rows - 1)
            || (self.d == Direction::RIGHT && new_front.0 == cols - 1)
        {
            return false;
        }

        match self.d {
            Direction::UP => new_front.1 -= 1,
            Direction::DOWN => new_front.1 += 1,
            Direction::LEFT => new_front.0 -= 1,
            Direction::RIGHT => new_front.0 += 1,
        }

        if !just_eaten {
            self.rake_parts.pop_back();
        }

        // Checks self collision.
        if self.is_collide(new_front.0, new_front.1) {
            return false;
        }

        self.rake_parts.push_front(new_front);
        true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {
        self.rake_parts.iter().any(|p| x == p.0 && y == p.1)
    }
}

pub struct Food {
    x: u32,
    y: u32,
}

impl Food {
    // Return true if rake ate food this update
    fn update(&mut self, s: &Rake) -> bool {
        let front = s.rake_parts.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let x = self.x * width;
        let y = self.y * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(RED, square, transform, gl)
        });
    }
}

fn main() {
    // Change to OpenGL::V2_1 if this fails.
    let opengl = OpenGL::V3_2;

    const COLS: u32 = 30;
    const ROWS: u32 = 20;
    const SQUARE_WIDTH: u32 = 20;

    let width = COLS * SQUARE_WIDTH;
    let height = ROWS * SQUARE_WIDTH;

    let mut window: GlutinWindow = WindowSettings::new("Rake Game", [width, height])
   //     .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLS,
        square_width: SQUARE_WIDTH,
        just_eaten: false,
        food: Food { x: 1, y: 1 },
        score: 0,
        rake: Rake {
            gl: GlGraphics::new(opengl),
            rake_parts: LinkedList::from_iter((vec![Rake_Piece(COLS / 2, ROWS / 2)]).into_iter()),
            width: SQUARE_WIDTH,
            d: Direction::DOWN,
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
    println!("Congratulations, your score was: {}", game.score);
}
