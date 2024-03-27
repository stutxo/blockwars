#![no_std]
use core::sync::atomic::{AtomicU32, Ordering};
static FRAME: AtomicU32 = AtomicU32::new(0);

static SQUARE_POS_X: AtomicU32 = AtomicU32::new(WIDTH as u32 / 2 - 5);
static SQUARE_POS_Y: AtomicU32 = AtomicU32::new(HEIGHT as u32 / 2 - 5);

const MAX_ENEMIES: usize = 600;

const ARRAY_REPEAT_VALUE: core::option::Option<Enemy> = None;
static mut ENEMIES: [Option<Enemy>; MAX_ENEMIES] = [ARRAY_REPEAT_VALUE; MAX_ENEMIES];

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const PLAYER_SPEED: u32 = 2;
const ENEMY_SIZE: usize = 15;
const PLAYER_SIZE: usize = 10;

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[no_mangle]
pub unsafe extern "C" fn game_loop() -> u32 {
    update_enemy_pos();
    render_frame_safe(&mut BUFFER);
    1
}

struct Enemy {
    x: usize,
    y: usize,
    frame_counter: usize,
}

pub enum Key {
    Left,
    Right,
    Up,
    Down,
}

struct Rng {
    seed: u32,
}

impl Rng {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;
    const M: u32 = 2_u32.pow(31);

    fn new(seed: u32) -> Self {
        Rng { seed }
    }

    fn rand(&mut self) -> u32 {
        self.seed = (Self::A.wrapping_mul(self.seed) + Self::C) % Self::M;
        self.seed
    }

    fn rand_in_range(&mut self, start: u32, end: u32) -> u32 {
        start + (self.rand() % (end - start + 1))
    }
}

#[no_mangle]
pub extern "C" fn spawn_enemies() {
    for _ in 0..100 {
        spawn_enemy();
    }
}

fn spawn_enemy() {
    unsafe {
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        let mut rng = Rng::new(123 + f);

        for slot in ENEMIES.iter_mut() {
            if slot.is_none() {
                match rng.rand() % 4 {
                    0 => {
                        let x = rng.rand() % (WIDTH as u32 - ENEMY_SIZE as u32);
                        *slot = Some(Enemy {
                            x: x as usize,
                            y: 0,
                            frame_counter: 0,
                        });
                    }
                    1 => {
                        let y = rng.rand() % (HEIGHT as u32 - ENEMY_SIZE as u32);
                        *slot = Some(Enemy {
                            x: WIDTH - ENEMY_SIZE as usize,
                            y: y as usize,
                            frame_counter: 0,
                        });
                    }
                    2 => {
                        let x = rng.rand() % (WIDTH as u32 - ENEMY_SIZE as u32);
                        *slot = Some(Enemy {
                            x: x as usize,
                            y: HEIGHT - ENEMY_SIZE as usize,
                            frame_counter: 0,
                        });
                    }
                    3 => {
                        let y = rng.rand() % (HEIGHT as u32 - ENEMY_SIZE as u32);
                        *slot = Some(Enemy {
                            x: 0,
                            y: y as usize,
                            frame_counter: 0,
                        });
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}

fn render_frame_safe(buffer: &mut [u32; WIDTH * HEIGHT]) {
    for i in 0..(WIDTH * HEIGHT) {
        buffer[i] = 0xFF_00_00_00;
    }

    let start_x = SQUARE_POS_X.load(Ordering::Relaxed) as usize;
    let start_y = SQUARE_POS_Y.load(Ordering::Relaxed) as usize;

    for y in start_y..(start_y + PLAYER_SIZE) {
        for x in start_x..(start_x + PLAYER_SIZE) {
            buffer[y * WIDTH + x] = 0xFFFFFF;
        }
    }

    unsafe {
        for enemy_option in ENEMIES.iter() {
            if let Some(enemy) = enemy_option {
                for y in enemy.y..(enemy.y + ENEMY_SIZE) {
                    for x in enemy.x..(enemy.x + ENEMY_SIZE) {
                        if x < WIDTH && y < HEIGHT {
                            buffer[y * WIDTH + x] = 0xFFFFBF00;
                        }
                    }
                }
            }
        }
    }
}

fn update_enemy_pos() {
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    let mut rng = Rng::new(123 + f);

    let square_x = SQUARE_POS_X.load(Ordering::Relaxed) as usize;
    let square_y = SQUARE_POS_Y.load(Ordering::Relaxed) as usize;

    if f % 600 == 0 && f < 1337 {
        spawn_enemies();
    }

    unsafe {
        for slot in ENEMIES.iter_mut() {
            if let Some(enemy) = slot {
                if enemy.frame_counter > 0 {
                    enemy.frame_counter -= 1;
                }

                if enemy.frame_counter == 0 {
                    if enemy.x > square_x {
                        enemy.x = enemy.x.saturating_sub(1);
                    } else if enemy.x < square_x {
                        enemy.x += 1;
                    }

                    if enemy.y > square_y {
                        enemy.y = enemy.y.saturating_sub(1);
                    } else if enemy.y < square_y {
                        enemy.y += 1;
                    }

                    if (enemy.x >= square_x - PLAYER_SIZE && enemy.x <= square_x + PLAYER_SIZE)
                        && (enemy.y >= square_y - PLAYER_SIZE && enemy.y <= square_y + PLAYER_SIZE)
                    {
                        *slot = None;
                        continue;
                    }

                    enemy.frame_counter = rng.rand_in_range(1, 8) as usize;
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn key_pressed(value: usize) {
    let key = match value {
        1 => Key::Left,
        2 => Key::Right,
        3 => Key::Up,
        4 => Key::Down,
        _ => return,
    };

    match key {
        Key::Left => {
            SQUARE_POS_X
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    if x >= PLAYER_SPEED {
                        Some(x - PLAYER_SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Right => {
            SQUARE_POS_X
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    if x + 10 + PLAYER_SPEED <= WIDTH as u32 {
                        Some(x + PLAYER_SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Up => {
            SQUARE_POS_Y
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |y| {
                    if y >= PLAYER_SPEED {
                        Some(y - PLAYER_SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Down => {
            SQUARE_POS_Y
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |y| {
                    if y + 10 + PLAYER_SPEED <= HEIGHT as u32 {
                        Some(y + PLAYER_SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
