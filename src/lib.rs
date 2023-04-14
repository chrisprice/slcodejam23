#![no_std]
use core::fmt::Debug;
use defmt::info;
use rand::{Rng, RngCore};
use smart_leds::RGB8;

const WIDTH: usize = 6;
const HEIGHT: usize = 6;

const CELL_COUNT: usize = WIDTH * HEIGHT;
const LED_COUNT: usize = CELL_COUNT * 2 + 5;
const MAX_SNAKE_LENGTH: usize = 10;

const SPARE_COLOR: RGB8 = smart_leds::colors::BLACK;
const PLAYFIELD_COLOR: RGB8 = smart_leds::colors::BLACK;
const SNAKE_TAIL_COLOR: RGB8 = smart_leds::colors::RED;
const SNAKE_HEAD_COLOR: RGB8 = smart_leds::colors::GREEN;
const FOOD_COLOR: RGB8 = smart_leds::colors::BLUE;

/// 0, 0 is bottom left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate(usize, usize);

impl Coordinate {
    fn rand(rng: &mut impl RngCore, exclusions: &[Option<Coordinate>]) -> Self {
        loop {
            let possible = Self(
                rng.gen_range(0..WIDTH) as usize,
                rng.gen_range(0..HEIGHT) as usize,
            );
            if !exclusions.contains(&Some(possible)) {
                return possible;
            }
        }
    }
    fn apply(&self, velocity: Velocity) -> Result<Self, ()> {
        match velocity {
            Velocity::Up => {
                if self.1 + 1 < HEIGHT {
                    Ok(Self(self.0, self.1 + 1))
                } else {
                    Err(())
                }
            }
            Velocity::Down => {
                if self.1 >= 1 {
                    Ok(Self(self.0, self.1 - 1))
                } else {
                    Err(())
                }
            }
            Velocity::Left => {
                if self.0 >= 1 {
                    Ok(Self(self.0 - 1, self.1))
                } else {
                    Err(())
                }
            }
            Velocity::Right => {
                if self.0 + 1 < WIDTH {
                    Ok(Self(self.0 + 1, self.1))
                } else {
                    Err(())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Player {
    P1,
    P2,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    CW,
    CCW,
}

#[derive(Debug, Clone, Copy)]
pub enum Velocity {
    Up,
    Down,
    Left,
    Right,
}
impl Velocity {
    fn apply(&self, direction: Direction) -> Velocity {
        match self {
            Self::Up => match direction {
                Direction::CW => Self::Right,
                Direction::CCW => Self::Left,
            },
            Self::Down => match direction {
                Direction::CW => Self::Left,
                Direction::CCW => Self::Right,
            },
            Self::Left => match direction {
                Direction::CW => Self::Up,
                Direction::CCW => Self::Down,
            },
            Self::Right => match direction {
                Direction::CW => Self::Down,
                Direction::CCW => Self::Up,
            },
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    next_direction: Option<Direction>,
    velocity: Velocity,
    snake: [Option<Coordinate>; MAX_SNAKE_LENGTH],
    food: Coordinate,
    pub player: Player,
    level: u32,
}

impl GameState {
    pub fn new(rng: &mut impl RngCore) -> Self {
        let velocity = Velocity::Right;
        let mut snake = [None; MAX_SNAKE_LENGTH];
        snake[0] = Some(Coordinate(1, 3));
        let food = Coordinate::rand(rng, snake.as_slice());
        Self {
            next_direction: None,
            velocity,
            snake,
            food,
            player: Player::P2,
            level: 0,
        }
    }

    fn is_driver(&self, player: Player) -> bool {
        match self.player {
            Player::P1 => match player {
                Player::P1 => true,
                Player::P2 => false,
            },
            Player::P2 => match player {
                Player::P1 => false,
                Player::P2 => true,
            },
        }
    }

    fn playfield(&self, player: Player) -> [[RGB8; WIDTH]; HEIGHT] {
        let mut playfield = [[PLAYFIELD_COLOR; WIDTH]; HEIGHT];
        let snake_head = self.snake[0].expect("snake to have a head");
        playfield[snake_head.1][snake_head.0] = SNAKE_HEAD_COLOR;
        if self.is_driver(player) {
            playfield[self.food.1][self.food.0] = FOOD_COLOR;
        } else {
            for snake_tail in self.snake.iter().skip(1) {
                if let Some(snake_tail) = snake_tail {
                    playfield[snake_tail.1][snake_tail.0] = SNAKE_TAIL_COLOR;
                }
            }
            let x = playfield[3].map(|item| item.r + item.g + item.b);
            info!("{}, {}, {}, {}, {}, {}", x[0], x[1], x[2], x[3], x[4], x[5]);
            for i in 0..playfield.len() {
                playfield[i].reverse();
            }
            let x = playfield[3].map(|item| item.r + item.g + item.b);
            info!("{}, {}, {}, {}, {}, {}", x[0], x[1], x[2], x[3], x[4], x[5]);
        }
        return playfield;
    }

    pub fn leds(&self) -> [RGB8; LED_COUNT] {
        fn copy_row(
            row: &mut [RGB8; WIDTH],
            target: &mut [RGB8; LED_COUNT],
            offset: usize,
            reverse: bool,
        ) -> usize {
            if reverse {
                row.reverse();
            }
            target[offset..offset + row.len()].copy_from_slice(row);
            return offset + row.len();
        }
        let mut leds = [PLAYFIELD_COLOR; LED_COUNT];
        let mut playfield_p1 = self.playfield(Player::P1);
        let mut playfield_p2 = self.playfield(Player::P2);
        let mut offset = 0;
        for i in 0..=2 {
            offset = copy_row(&mut playfield_p1[i * 2 + 0], &mut leds, offset, false);
            offset = copy_row(&mut playfield_p2[i * 2 + 0], &mut leds, offset, false);
            leds[offset] = SPARE_COLOR;
            offset += 1;
            offset = copy_row(&mut playfield_p2[i * 2 + 1], &mut leds, offset, true);
            offset = copy_row(&mut playfield_p1[i * 2 + 1], &mut leds, offset, true);
            if offset < LED_COUNT {
                leds[offset] = SPARE_COLOR;
                offset += 1;
            }
        }
        leds
    }

    pub fn button_push(&mut self, player: Player, direction: Direction) {
        if !self.is_driver(player) {
            return;
        }
        self.next_direction = Some(direction);
    }

    fn snake(&self) -> impl Iterator<Item = &'_ Coordinate> + '_ {
        self.snake.iter().filter_map(|item| item.as_ref())
    }

    pub fn tick(&mut self, rng: &mut impl RngCore) -> u32 {
        let snake_head = self.snake[0].expect("snake to have a head");
        if let Some(next_direction) = self.next_direction {
            self.velocity = self.velocity.apply(next_direction);
            self.next_direction = None;
        }
        let new_snake_head = snake_head.apply(self.velocity);
        if let Ok(new_snake_head) = new_snake_head {
            if !self.snake.contains(&Some(new_snake_head)) {
                self.snake.rotate_right(1);
                self.snake[0] = Some(new_snake_head);
                if new_snake_head == self.food {
                    if self.snake().count() >= MAX_SNAKE_LENGTH {
                        self.snake
                            .iter_mut()
                            .filter(|item| item.is_some())
                            .skip(1)
                            .for_each(|item| {
                                item.take();
                            });
                        self.level += 1;
                    }
                    self.food = Coordinate::rand(rng, self.snake.as_slice());
                    self.player = match self.player {
                        Player::P1 => Player::P2,
                        Player::P2 => Player::P1,
                    };
                } else {
                    self.snake
                        .iter_mut()
                        .filter(|item| item.is_some())
                        .last()
                        .expect("snake to be at least head + one tail segment")
                        .take();
                }
            } else {
                // RESET
                *self = GameState::new(rng);
            }
        } else {
            // RESET
            *self = GameState::new(rng);
        }

        2000 / 2_u32.pow(self.level)
    }
}
