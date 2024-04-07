#![no_std]
use core::ptr;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

const MAX_TELEPORT: usize = 5;
const TELEPORT_SIZE: u8 = 10;
const TELEPORT_SPEED: f32 = 10.;

const TELEPORT_NONE: core::option::Option<(f32, f32, f32)> = None;
static mut TELEPORT: [Option<(f32, f32, f32)>; MAX_TELEPORT] = [TELEPORT_NONE; MAX_TELEPORT];

const MAX_ENEMY: usize = 15;
const ENEMY_HEIGHT: u8 = 5;
const ENEMY_WIDTH: u8 = 10;

static mut ENEMY: [(f32, f32, f32, f32); MAX_ENEMY] = [(0., 0., 0., 0.); MAX_ENEMY];

#[no_mangle]
static mut INPUT: [u8; 1] = [0; 1];

#[no_mangle]
static mut RESET: [u8; 1] = [0; 1];

#[no_mangle]
static mut DRAW: [u32; 255 * 255] = [0; 255 * 255];

#[no_mangle]
static mut SEED: [u32; 32] = [0; 32];

#[inline]
#[no_mangle]
unsafe extern "C" fn blockwars() {
    if RESET[0] == 1 {
        RESET[0] = 0;
        INPUT[0] = 0;
        DRAW.iter_mut().for_each(|b| *b = 0);
        spawn_tele(&mut *ptr::addr_of_mut!(TELEPORT), SEED);
        spawn_enemy(&mut *ptr::addr_of_mut!(ENEMY), SEED);
    } else if RESET[0] == 0 {
        DRAW.iter_mut().for_each(|b| *b = 0);
        frame_safe(
            &mut *ptr::addr_of_mut!(DRAW),
            &mut *ptr::addr_of_mut!(TELEPORT),
            &mut *ptr::addr_of_mut!(INPUT),
            &mut *ptr::addr_of_mut!(ENEMY),
            &mut *ptr::addr_of_mut!(RESET),
        );
    }
}

//no unsafe code below this point
#[inline]
fn frame_safe(
    draw: &mut [u32; 255 * 255],
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    input: &mut [u8; 1],
    enemies: &mut [(f32, f32, f32, f32); MAX_ENEMY],
    reset: &mut [u8; 1],
) {
    let gg = teleporters
        .iter()
        .rev()
        .find_map(|t| t.as_ref())
        .map_or(false, |teleporter| teleporter.2 == 1.0);

    if input[0] == 1 {
        move_player(teleporters, input, reset, enemies);
    }
    move_enemy(enemies);
    render_frame(draw, teleporters, enemies, gg);
}

#[inline]
fn spawn_tele(teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT], rng: [u32; 32]) {
    let max_index_x = WIDTH as usize - ENEMY_WIDTH as usize;
    let max_index_y = HEIGHT as usize - ENEMY_HEIGHT as usize;

    let num_teleporters = 4;

    for i in 0..num_teleporters {
        let random_value_x = rng[i] ^ rng[(31 - i) % rng.len()];

        let x = random_value_x % max_index_x as u32;

        let random_value_y = rng[i] ^ rng[(30 - i) % rng.len()];
        let y = random_value_y % max_index_y as u32;

        teleporters[i] = Some((
            x as f32,
            y as f32,
            match i {
                0 => 1.0,
                1 => 2.0,
                _ => 3.0,
            },
        ));
    }

    for i in num_teleporters..teleporters.len() {
        teleporters[i] = None;
    }
}

#[inline]
fn spawn_enemy(enemies: &mut [(f32, f32, f32, f32); MAX_ENEMY], rng: [u32; 32]) {
    let max_index_x = WIDTH as usize - ENEMY_WIDTH as usize;
    let max_index_y = HEIGHT as usize - ENEMY_HEIGHT as usize;

    for i in 0..MAX_ENEMY {
        let random_value_x = rng[i] ^ rng[(31 - i) % rng.len()];

        let x = random_value_x % max_index_x as u32 + 20;

        let random_value_y = rng[i] ^ rng[(30 - i) % rng.len()];

        let y = random_value_y % max_index_y as u32 + 20;

        let raw_random_value = rng[i] ^ rng[31];
        let scaled_random_value = raw_random_value % 4 + 1;

        enemies[i] = (x as f32, y as f32, scaled_random_value as f32, 10.);
    }
}

#[inline]
fn move_enemy(enemies: &mut [(f32, f32, f32, f32); MAX_ENEMY]) {
    for enemy in enemies.iter_mut() {
        let (x, y, state, count) = enemy;
        let movement_count = 64.0;

        if *count >= movement_count {
            *state = match *state {
                1.0 => 2.0,
                2.0 => 3.0,
                3.0 => 4.0,
                4.0 => 1.0,
                _ => 1.0,
            };
            *count = 1.0;
        } else {
            *count += 1.0;
        }

        match *state {
            1.0 => *x += 1.0,
            2.0 => *y -= 1.0,
            3.0 => *x -= 1.0,
            4.0 => *y += 1.0,
            _ => (),
        }
    }
}

#[inline]
fn move_player(
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    input: &mut [u8; 1],
    reset: &mut [u8; 1],
    enemies: &mut [(f32, f32, f32, f32); MAX_ENEMY],
) {
    if input[0] == 1 {
        if let Some((current, target)) = find_teleporter_targets(teleporters) {
            if let (Some(current_pos), Some(target_pos)) =
                (teleporters[current], teleporters[target])
            {
                let dx = target_pos.0 - current_pos.0;
                let dy = target_pos.1 - current_pos.1;

                let distance = sqrt_approx(dx * dx + dy * dy);

                if distance <= TELEPORT_SPEED {
                    teleporters[current] = Some((target_pos.0, target_pos.1, 4.0));
                    teleporters[target] = Some((target_pos.0, target_pos.1, 1.0));

                    let next_index = (target + 1) % MAX_TELEPORT;
                    if let Some(next_teleporter) = teleporters[next_index] {
                        teleporters[next_index] = Some((next_teleporter.0, next_teleporter.1, 2.0));
                    }

                    input[0] = 0;
                } else {
                    let dir_x = dx / distance;
                    let dir_y = dy / distance;

                    for enemy in enemies.iter() {
                        let (enemy_x, enemy_y, _, _) = enemy;
                        let enemy_dx = *enemy_x - current_pos.0;
                        let enemy_dy = *enemy_y - current_pos.1;
                        let enemy_distance = sqrt_approx(enemy_dx * enemy_dx + enemy_dy * enemy_dy);

                        if enemy_distance <= 10.0 {
                            *reset = [2];
                        }
                    }

                    teleporters[current] = Some((
                        current_pos.0 + dir_x * TELEPORT_SPEED,
                        current_pos.1 + dir_y * TELEPORT_SPEED,
                        current_pos.2,
                    ));
                }
            }
        }
    }
}

#[inline]
fn sqrt_approx(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    if value == 1.0 {
        return 1.0;
    }

    let mut x = value;
    let mut y = (x / 2.0) + 1.0;
    while y < x {
        x = y;
        y = (x + value / x) / 2.0;
    }
    x
}

#[inline]
fn find_teleporter_targets(
    teleporters: &[Option<(f32, f32, f32)>; MAX_TELEPORT],
) -> Option<(usize, usize)> {
    let current_index = teleporters.iter().position(|&tele| {
        if let Some((_, _, state)) = tele {
            state == 1.0
        } else {
            false
        }
    })?;

    let next_target_index = teleporters.iter().position(|&tele| {
        if let Some((_, _, state)) = tele {
            state == 2.0
        } else {
            false
        }
    })?;

    Some((current_index, next_target_index))
}

#[inline]
fn render_frame(
    draw: &mut [u32; 255 * 255],
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    enemies: &mut [(f32, f32, f32, f32); MAX_ENEMY],
    gg: bool,
) {
    let mut draw_rect = |x: f32, y: f32, width: u8, height: u8, state: u32| {
        for dy in 0..height {
            for dx in 0..width {
                let index =
                    (y + f32::from(dy)) as usize * WIDTH as usize + (x + f32::from(dx)) as usize;
                if index < draw.len() {
                    draw[index] = 0;
                    draw[index] = state;
                }
            }
        }
    };

    for state in (0..=4).rev() {
        for teleport in teleporters.iter() {
            if let Some((x, y, tele_state)) = teleport {
                if *tele_state == state as f32 {
                    draw_rect(*x, *y, TELEPORT_SIZE, TELEPORT_SIZE, state);
                }
            }
        }
        for enemy in enemies.iter() {
            let (x, y, enemy_state, _) = enemy;
            if *enemy_state == state as f32 {
                if gg {
                    draw_rect(*x, *y, ENEMY_WIDTH, ENEMY_HEIGHT, 1);
                } else {
                    draw_rect(*x, *y, ENEMY_WIDTH, ENEMY_HEIGHT, 5);
                }
            }
        }
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
