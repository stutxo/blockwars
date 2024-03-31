#![no_std]
use core::{
    ptr,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering},
};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const SEED: u32 = 0x1331;

static mut PLAYER: (u8, u8) = (WIDTH / 2 - 5, HEIGHT / 2 - 5);
const PLAYER_SIZE: u8 = 5;
const PLAYER_SPEED: u8 = 1;

static mut ENEMIES: [(u8, u8, u8); MAX_ENEMIES] = [(0, 0, 0); MAX_ENEMIES];
const ENEMY_SIZE: u8 = 5;
const ENEMIES_PER_WAVE: u8 = 1;
const MAX_ENEMIES: usize = 255 * 255 / 25;

static GAME_OVER: AtomicBool = AtomicBool::new(false);
static FRAME: AtomicU32 = AtomicU32::new(0);
static KEY_STATE: AtomicU8 = AtomicU8::new(0);

//https://blog.orhun.dev/zero-deps-random-in-rust/
#[inline]
fn rng() -> impl Iterator<Item = u32> {
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    let mut random = SEED + f;
    core::iter::repeat_with(move || {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        random
    })
}

enum Key {
    Left,
    Right,
    Up,
    Down,
}

#[no_mangle]
static mut BUFFER: [u32; 255 * 255] = [0; 255 * 255];

#[inline]
#[no_mangle]
unsafe extern "C" fn key_pressed(value: u8) {
    KEY_STATE.store(value, Ordering::Relaxed);
}

#[inline]
#[no_mangle]
unsafe extern "C" fn game_loop() -> u32 {
    if !GAME_OVER.load(Ordering::Relaxed) {
        frame_safe(
            &mut *ptr::addr_of_mut!(BUFFER),
            &mut *ptr::addr_of_mut!(ENEMIES),
            &mut *ptr::addr_of_mut!(PLAYER),
        );
        1
    } else {
        0
    }
}

#[inline]
fn frame_safe(
    buffer: &mut [u32; 255 * 255],
    enemies: &mut [(u8, u8, u8); MAX_ENEMIES],
    player: &mut (u8, u8),
) {
    let mut rng = rng();

    spawn_enemy(enemies, &mut rng);
    update_player_pos(player);
    update_enemy_pos(enemies, player);
    render_frame(buffer, enemies, *player);
}

#[inline]
fn spawn_enemy(enemies: &mut [(u8, u8, u8); MAX_ENEMIES], rng: &mut impl Iterator<Item = u32>) {
    let width_limit = (WIDTH - ENEMY_SIZE) as u32;
    let height_limit = (HEIGHT - ENEMY_SIZE) as u32;

    for _ in 0..ENEMIES_PER_WAVE {
        if let Some(slot) = enemies.iter_mut().find(|e| e.2 == 0) {
            let edge = rng.next().unwrap() % 4;

            let position = match edge {
                0 => ((rng.next().unwrap() % width_limit) as u8, 0),
                1 => (
                    WIDTH - ENEMY_SIZE,
                    (rng.next().unwrap() % height_limit) as u8,
                ),
                2 => (
                    (rng.next().unwrap() % width_limit) as u8,
                    HEIGHT - ENEMY_SIZE,
                ),
                _ => (0, (rng.next().unwrap() % height_limit) as u8),
            };

            *slot = new_enemy(position.0, position.1);
        }
    }
}

#[inline]
fn new_enemy(x: u8, y: u8) -> (u8, u8, u8) {
    (x, y, 1)
}

#[inline]
fn update_player_pos(player: &mut (u8, u8)) {
    let key = match KEY_STATE.load(Ordering::Relaxed) {
        1 => Some(Key::Left),
        2 => Some(Key::Right),
        3 => Some(Key::Up),
        4 => Some(Key::Down),
        _ => None,
    };

    if let Some(key) = key {
        match key {
            Key::Left => player.0 = player.0.wrapping_sub(PLAYER_SPEED),
            Key::Right => player.0 = player.0.wrapping_add(PLAYER_SPEED),
            Key::Up => player.1 = player.1.wrapping_sub(PLAYER_SPEED),
            Key::Down => player.1 = player.1.wrapping_add(PLAYER_SPEED),
        }
    }
}

#[inline]
fn update_enemy_pos(enemies: &mut [(u8, u8, u8); MAX_ENEMIES], player: &mut (u8, u8)) {
    for enemy in enemies.iter_mut() {
        if enemy.2 == 0 {
            continue;
        }

        if enemy.0 > player.0 {
            enemy.0 -= 1;
        } else if enemy.0 < player.0 {
            enemy.0 += 1;
        }

        if enemy.1 > player.1 {
            enemy.1 -= 1;
        } else if enemy.1 < player.1 {
            enemy.1 += 1;
        }

        if (enemy.0 < player.0 + PLAYER_SIZE)
            && (enemy.0 + ENEMY_SIZE > player.0)
            && (enemy.1 < player.1 + PLAYER_SIZE)
            && (enemy.1 + ENEMY_SIZE > player.1)
        {
            // GAME_OVER.store(true, Ordering::Relaxed);
            enemy.2 = 0;
        }
    }
}

#[inline]
fn render_frame(
    buffer: &mut [u32; 255 * 255],
    enemies: &[(u8, u8, u8); MAX_ENEMIES],
    player: (u8, u8),
) {
    buffer.fill(0xFF_00_00_00);

    let mut draw_rect = |x: u8, y: u8, width: u8, height: u8, color: u32| {
        for dy in 0..height {
            for dx in 0..width {
                let index = usize::from(y + dy) * usize::from(WIDTH) + usize::from(x + dx);
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }
    };

    for enemy in enemies.iter() {
        if enemy.2 == 0 {
            continue;
        }
        draw_rect(enemy.0, enemy.1, ENEMY_SIZE, ENEMY_SIZE, 0xFFFFBF00);
    }

    draw_rect(player.0, player.1, PLAYER_SIZE, PLAYER_SIZE, 0xFFFFFF);
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
