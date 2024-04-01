#![no_std]
use core::ptr;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const SEED: u32 = 0x1337;

static mut PLAYER: (u8, u8, u8) = (WIDTH / 2 - 5, (HEIGHT / 4) * 3, 1);

const PLAYER_SIZE: u8 = 5;
const PLAYER_SPEED: u32 = 3;

static mut ENEMIES: [(u8, u8, u8); MAX_ENEMIES] = [(0, 0, 0); MAX_ENEMIES];
const ENEMY_SIZE: u8 = 4;
const ENEMIES_PER_WAVE: u8 = 1;
const MAX_ENEMIES: usize = 255 * 255 / 25;
const ENEMY_SPAWN_RATE: u32 = 10;
const ENEMY_SPEED: u32 = 2;

static mut KEY_STATE: u8 = 0;
static mut FRAME: u32 = 0;

//https://blog.orhun.dev/zero-deps-random-in-rust/
#[inline]
fn rng(frame: u32) -> impl Iterator<Item = u32> {
    let mut random = SEED + frame;
    core::iter::repeat_with(move || {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        random
    })
}

//accessed by javascript
#[no_mangle]
static mut BUFFER: [u32; 255 * 255] = [0; 255 * 255];

#[inline]
#[no_mangle]
unsafe extern "C" fn key_pressed(value: u8) {
    KEY_STATE = value;
}

#[inline]
#[no_mangle]
unsafe extern "C" fn game_loop() -> u32 {
    FRAME += 1;
    frame_safe(
        &mut *ptr::addr_of_mut!(BUFFER),
        &mut *ptr::addr_of_mut!(ENEMIES),
        &mut *ptr::addr_of_mut!(PLAYER),
        FRAME,
        KEY_STATE,
    );
    1
}

//no unsafe code below this point
#[inline]
fn frame_safe(
    buffer: &mut [u32; 255 * 255],
    enemies: &mut [(u8, u8, u8); MAX_ENEMIES],
    player: &mut (u8, u8, u8),
    frame: u32,
    key_state: u8,
) {
    let mut rng = rng(frame);

    if frame % ENEMY_SPAWN_RATE == 0 {
        spawn_enemy(enemies, &mut rng);
    }
    if frame % PLAYER_SPEED == 0 {
        update_player_pos(player, key_state);
    }
    if frame % ENEMY_SPEED == 0 {
        update_enemy_pos(enemies, player);
    }
    render_frame(buffer, enemies, *player);
}

fn spawn_enemy(enemies: &mut [(u8, u8, u8); MAX_ENEMIES], rng: &mut impl Iterator<Item = u32>) {
    let spawn_range_start = 64;
    let spawn_range_end = 191;

    let spawn_range_width = (spawn_range_end - spawn_range_start) as u32;

    for _ in 0..ENEMIES_PER_WAVE {
        if let Some(slot) = enemies.iter_mut().find(|e| e.2 == 0) {
            let position_x =
                (rng.next().unwrap() % spawn_range_width + spawn_range_start as u32) as u8;
            let position = (position_x, 0);

            *slot = new_enemy(position.0, position.1);
        }
    }
}

#[inline]
fn new_enemy(x: u8, y: u8) -> (u8, u8, u8) {
    (x, y, 1)
}

enum Key {
    Left,
    Right,
}

#[inline]
fn update_player_pos(player: &mut (u8, u8, u8), key_state: u8) {
    let key = match key_state {
        1 => Some(Key::Left),
        2 => Some(Key::Right),
        _ => None,
    };

    if let Some(key) = key {
        match key {
            Key::Left => player.0 = player.0.wrapping_sub(1),
            Key::Right => player.0 = player.0.wrapping_add(1),
        }
    }
}

#[inline]
fn update_enemy_pos(enemies: &mut [(u8, u8, u8); MAX_ENEMIES], player: &mut (u8, u8, u8)) {
    for enemy in enemies.iter_mut() {
        if enemy.1 == HEIGHT {
            enemy.2 = 0;
        }

        if enemy.2 == 0 {
            continue;
        }

        enemy.1 += 1;

        if (enemy.0 < player.0 + PLAYER_SIZE)
            && (enemy.0 + ENEMY_SIZE > player.0)
            && (enemy.1 < player.1 + PLAYER_SIZE)
            && (enemy.1 + ENEMY_SIZE > player.1)
        {
            player.2 = 0;
        }
    }
}

#[inline]
fn render_frame(
    buffer: &mut [u32; 255 * 255],
    enemies: &[(u8, u8, u8); MAX_ENEMIES],
    player: (u8, u8, u8),
) {
    if player.2 == 0 {
        return;
    }

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
