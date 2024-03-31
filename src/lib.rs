#![no_std]
use core::{
    ptr,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering},
};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PLAYER_SPEED: u8 = 1;
const ENEMY_SIZE: u8 = 5;
const PLAYER_SIZE: u8 = 5;
const WALL_SIZE: u8 = 2;
const SEED: u32 = 0x1331;
const ENEMIES_PER_WAVE: u8 = 255;
const MAX_ENEMIES: usize = 1000000;
const ENEMIES_NONE: core::option::Option<(u8, u8, u8)> = None;
const MAX_WALL: usize = 10000;
const WALL_NONE: core::option::Option<(u8, u8, u8)> = None;

static mut PLAYER: [Option<(u8, u8)>; 1] = [None; 1];
static mut ENEMIES: [Option<(u8, u8, u8)>; MAX_ENEMIES] = [ENEMIES_NONE; MAX_ENEMIES];
static mut WALL: [Option<(u8, u8, u8)>; MAX_WALL] = [WALL_NONE; MAX_WALL];

static GAME_OVER: AtomicBool = AtomicBool::new(false);
static FRAME: AtomicU32 = AtomicU32::new(0);
static KEY_STATE: AtomicU8 = AtomicU8::new(0);

#[inline]
fn new_enemy(x: u8, y: u8) -> (u8, u8, u8) {
    (x, y, 1)
}

#[inline]
fn new_player() -> (u8, u8) {
    (WIDTH / 2 - 5, HEIGHT / 2 - 5)
}

#[inline]
fn new_wall(x: u8, y: u8) -> (u8, u8, u8) {
    (x, y, 0)
}

enum Key {
    Left,
    Right,
    Up,
    Down,
}

//https://blog.orhun.dev/zero-deps-random-in-rust/
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
            &mut *ptr::addr_of_mut!(WALL),
        );
        1
    } else {
        0
    }
}

#[inline]
fn frame_safe(
    buffer: &mut [u32; 255 * 255],
    enemies: &mut [Option<(u8, u8, u8)>; MAX_ENEMIES],
    player: &mut [Option<(u8, u8)>; 1],
    wall: &mut [Option<(u8, u8, u8)>; MAX_WALL],
) {
    let mut rng = rng();
    if player[0].is_none() {
        spawn_player(player);
    }

    spawn_enemy(enemies, &mut rng);
    update_player_pos(player, wall);
    update_enemy_pos(enemies, player);
    check_wall_collision(wall, enemies);
    render_frame(buffer, enemies, player, wall);
}

#[inline]
fn spawn_player(player: &mut [Option<(u8, u8)>; 1]) {
    for slot in player.iter_mut() {
        if slot.is_none() {
            *slot = Some(new_player());
        }
    }
}

#[inline]
fn spawn_enemy(
    enemies: &mut [Option<(u8, u8, u8)>; MAX_ENEMIES],
    rng: &mut impl Iterator<Item = u32>,
) {
    let width_limit = (WIDTH - ENEMY_SIZE) as u32;
    let height_limit = (HEIGHT - ENEMY_SIZE) as u32;

    for _ in 0..ENEMIES_PER_WAVE {
        if let Some(slot) = enemies.iter_mut().find(|e| e.is_none()) {
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

            *slot = Some(new_enemy(position.0, position.1));
        }
    }
}

#[inline]
fn update_player_pos(
    player: &mut [Option<(u8, u8)>; 1],
    wall: &mut [Option<(u8, u8, u8)>; MAX_WALL],
) {
    if let Some(player) = &mut player[0] {
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

            attempt_spawn_wall(player, wall);
        }
    }
}

#[inline]
fn attempt_spawn_wall(player: &(u8, u8), wall: &mut [Option<(u8, u8, u8)>; MAX_WALL]) {
    let player_center_x = player.0 + PLAYER_SIZE / 2;
    let player_center_y = player.1 + PLAYER_SIZE / 2;

    if !wall.iter().any(|w| matches!(w, Some(wall) if wall.0 == player_center_x && wall.1 == player_center_y && wall.2 == 0)) {
        if let Some(slot) = wall
            .iter_mut()
            .find(|wall| wall.is_none() || wall.as_ref().map_or(false, |wall| wall.2 == 1))
        {
            *slot = Some(new_wall(player_center_x, player_center_y));
        }
    }
}

#[inline]
fn update_enemy_pos(
    enemies: &mut [Option<(u8, u8, u8)>; MAX_ENEMIES],
    player: &mut [Option<(u8, u8)>; 1],
) {
    for enemy_entity in enemies.iter_mut() {
        for player_entity in player.iter() {
            if let Some(player) = player_entity {
                if let Some(enemy) = enemy_entity {
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
                    }
                }
            }
        }
    }
}

#[inline]
fn check_wall_collision(
    wall: &mut [Option<(u8, u8, u8)>; MAX_WALL],
    enemies: &mut [Option<(u8, u8, u8)>; MAX_ENEMIES],
) {
    for wall_entity in wall.iter_mut() {
        if let Some(wall) = wall_entity {
            if wall.2 == 1 {
                continue;
            }

            for enemy_entity in enemies.iter_mut() {
                if let Some(enemy) = enemy_entity {
                    if enemy.2 == 0 {
                        continue;
                    }
                    if (enemy.0 < wall.0 + WALL_SIZE)
                        && (enemy.0 + ENEMY_SIZE > wall.0)
                        && (enemy.1 < wall.1 + WALL_SIZE)
                        && (enemy.1 + ENEMY_SIZE > wall.1)
                    {
                        enemy.2 = 0;
                        wall.2 = 1;
                    }
                }
            }
        }
    }
}

#[inline]
fn render_frame(
    buffer: &mut [u32; 255 * 255],
    enemies: &[Option<(u8, u8, u8)>; MAX_ENEMIES],
    player: &[Option<(u8, u8)>; 1],
    wall: &[Option<(u8, u8, u8)>; MAX_WALL],
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

    if let Some(player) = player.iter().flatten().next() {
        draw_rect(player.0, player.1, PLAYER_SIZE, PLAYER_SIZE, 0xFFFFFF);
    }

    for wall in wall.iter().flatten().filter(|wall| wall.2 == 0) {
        draw_rect(wall.0, wall.1, WALL_SIZE, WALL_SIZE, 0xFFFFFF);
    }

    for enemy in enemies.iter().flatten().filter(|e| e.2 != 0) {
        draw_rect(enemy.0, enemy.1, ENEMY_SIZE, ENEMY_SIZE, 0xFFFFBF00);
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
