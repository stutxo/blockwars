#![no_std]
use core::sync::atomic::{AtomicU32, Ordering};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

static FRAME: AtomicU32 = AtomicU32::new(0);

static SQUARE_POS_X: AtomicU32 = AtomicU32::new(WIDTH as u32 / 2 - 5);
static SQUARE_POS_Y: AtomicU32 = AtomicU32::new(HEIGHT as u32 / 2 - 5);

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const SPEED: u32 = 2;

const PERIMETER: usize = 2 * (WIDTH + HEIGHT) - 4;

extern crate alloc;
use alloc::vec::Vec;

use spin::Once;

use talc::TalckWasm;

static ENEMIES: Once<spin::Mutex<Vec<Enemy>>> = Once::new();

#[global_allocator]
static ALLOCATOR: TalckWasm = unsafe { TalckWasm::new_global() };

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[no_mangle]
pub unsafe extern "C" fn go() {
    update_enemy_positions_towards_square();
    render_frame_safe(&mut BUFFER)
}

struct Enemy {
    x: usize,
    y: usize,
}

pub enum Key {
    Left,
    Right,
    Up,
    Down,
}

#[no_mangle]
pub extern "C" fn main() {
    for _ in 0..100 {
        spawn_enemy();
    }
}

fn spawn_enemy() {
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    let mut small_rng = SmallRng::seed_from_u64(1232 + f as u64);

    let point = small_rng.gen_range(0..PERIMETER);

    let (x, y) = if point < WIDTH {
        (point, 0)
    } else if point < WIDTH + HEIGHT - 1 {
        (WIDTH - 1, point - WIDTH + 1)
    } else if point < 2 * WIDTH + HEIGHT - 2 {
        (2 * WIDTH + HEIGHT - 2 - point - 1, HEIGHT - 1)
    } else {
        (0, PERIMETER - point)
    };

    let enemy = Enemy { x, y };
    ENEMIES
        .call_once(|| spin::Mutex::new(Vec::new()))
        .lock()
        .push(enemy);
}

fn render_frame_safe(buffer: &mut [u32; WIDTH * HEIGHT]) {
    for i in 0..(WIDTH * HEIGHT) {
        buffer[i] = 0xFF_00_00_00;
    }

    let start_x = SQUARE_POS_X.load(Ordering::Relaxed) as usize;
    let start_y = SQUARE_POS_Y.load(Ordering::Relaxed) as usize;

    for y in start_y..(start_y + 10) {
        for x in start_x..(start_x + 10) {
            buffer[y * WIDTH + x] = 0xFFFFFF;
        }
    }

    let enemies = ENEMIES.call_once(|| spin::Mutex::new(Vec::new())).lock();
    for enemy in enemies.iter() {
        for y in enemy.y..(enemy.y + 10) {
            for x in enemy.x..(enemy.x + 10) {
                if x < WIDTH && y < HEIGHT {
                    buffer[y * WIDTH + x] = 0xFFFFBF00;
                }
            }
        }
    }
}

fn update_enemy_positions_towards_square() {
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    let mut rng = SmallRng::seed_from_u64(1232 + f as u64);
    let square_x = SQUARE_POS_X.load(Ordering::Relaxed) as usize;
    let square_y = SQUARE_POS_Y.load(Ordering::Relaxed) as usize;

    if rng.gen_bool(0.1) {
        spawn_enemy();
    }

    let mut enemies = ENEMIES.call_once(|| spin::Mutex::new(Vec::new())).lock();
    for enemy in enemies.iter_mut() {
        if rng.gen_bool(0.7) {
            continue;
        }

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
                    if x >= SPEED {
                        Some(x - SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Right => {
            SQUARE_POS_X
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    if x + 10 + SPEED <= WIDTH as u32 {
                        Some(x + SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Up => {
            SQUARE_POS_Y
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |y| {
                    if y >= SPEED {
                        Some(y - SPEED)
                    } else {
                        None
                    }
                })
                .ok();
        }
        Key::Down => {
            SQUARE_POS_Y
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |y| {
                    if y + 10 + SPEED <= HEIGHT as u32 {
                        Some(y + SPEED)
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
