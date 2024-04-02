#![no_std]
use core::ptr;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

static mut TELEPORT: [(u8, u8, u8); MAX_TELEPORT] = [(0, 0, 0); MAX_TELEPORT];
const MAX_TELEPORT: usize = 8;
const TELEPORT_SIZE: u8 = 15;
const TELEPORT_SPEED: u8 = 5;

static mut GRID: [bool; GRID_WIDTH * GRID_HEIGHT] = [false; GRID_WIDTH * GRID_HEIGHT];
const GRID_WIDTH: usize = (WIDTH as usize) / TELEPORT_SIZE as usize;
const GRID_HEIGHT: usize = (HEIGHT as usize) / TELEPORT_SIZE as usize;

static mut KEY_STATE: bool = false;
static mut FRAME: u32 = 0;
static mut SEED: u32 = 0;

//https://blog.orhun.dev/zero-deps-random-in-rust/
#[inline]
fn rng(seed: u32, frame: u32) -> impl Iterator<Item = u32> {
    let mut random = seed + frame;
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
unsafe extern "C" fn key_pressed() {
    KEY_STATE = true;
}

#[inline]
#[no_mangle]
unsafe extern "C" fn seed(value: u32) {
    SEED = value;
}

#[inline]
#[no_mangle]
unsafe extern "C" fn game_loop() {
    FRAME += 1;
    frame_safe(
        &mut *ptr::addr_of_mut!(BUFFER),
        FRAME,
        &mut *ptr::addr_of_mut!(KEY_STATE),
        SEED,
        &mut *ptr::addr_of_mut!(TELEPORT),
        &mut *ptr::addr_of_mut!(GRID),
    );
}

//no unsafe code below this point
#[inline]
fn frame_safe(
    buffer: &mut [u32; 255 * 255],
    frame: u32,
    key_state: &mut bool,
    seed: u32,
    teleport: &mut [(u8, u8, u8); MAX_TELEPORT],
    grid: &mut [bool; GRID_WIDTH * GRID_HEIGHT],
) {
    let mut rng = rng(seed, frame);

    if frame == 1 {
        spawn_teleporters_on_grid(teleport, &mut rng, grid);
    }

    update_teleporter_pos(teleport, key_state);
    render_frame(buffer, teleport);
}

#[inline]
fn place_teleporter(
    last_position: (usize, usize),
    rng: &mut impl Iterator<Item = u32>,
    grid: &mut [bool; GRID_WIDTH * GRID_HEIGHT],
) -> Option<(usize, usize)> {
    let mut attempts = 0;

    while attempts < 100 {
        let same_column = rng.next().unwrap() % 2 == 0;
        let (new_x, new_y);

        if same_column {
            new_x = last_position.0;
            let direction = rng.next().unwrap() % 7 + 5; // Generates a number between 5 and 11
            new_y = (last_position.1 + direction as usize) % GRID_HEIGHT;
        } else {
            new_y = last_position.1;
            let direction = rng.next().unwrap() % 7 + 5; // Generates a number between 5 and 11
            new_x = (last_position.0 + direction as usize) % GRID_WIDTH;
        }

        let is_valid_position = match same_column {
            true => (new_y != last_position.1) && ((new_y + 1) % GRID_HEIGHT != last_position.1),
            false => (new_x != last_position.0) && ((new_x + 1) % GRID_WIDTH != last_position.0),
        };

        let cell_index = new_y * GRID_WIDTH + new_x;

        if !grid[cell_index] && is_valid_position {
            grid[cell_index] = true;
            return Some((new_x, new_y));
        }

        attempts += 1;
    }

    None
}

#[inline]
fn spawn_teleporters_on_grid(
    teleport: &mut [(u8, u8, u8); MAX_TELEPORT],
    rng: &mut impl Iterator<Item = u32>,
    grid: &mut [bool; GRID_WIDTH * GRID_HEIGHT],
) {
    let min_teleporters = 2;
    let range = MAX_TELEPORT.saturating_sub(min_teleporters) + 1;

    let num_teleporters = if range > 0 {
        (rng.next().unwrap() % range as u32) as usize + min_teleporters
    } else {
        min_teleporters
    };

    let mut last_position = (
        rng.next().unwrap() as usize % GRID_WIDTH,
        rng.next().unwrap() as usize % GRID_HEIGHT,
    );

    let initial_cell_index = last_position.1 * GRID_WIDTH + last_position.0;

    grid[initial_cell_index] = true;

    for i in 0..num_teleporters {
        if i < teleport.len() {
            let pos = if i == 0 {
                last_position
            } else if let Some(next_position) = place_teleporter(last_position, rng, grid) {
                last_position = next_position;
                next_position
            } else {
                continue;
            };

            teleport[i] = (
                pos.0 as u8 * TELEPORT_SIZE as u8,
                pos.1 as u8 * TELEPORT_SIZE as u8,
                1,
            );

            if i == 0 {
                teleport[i].2 = 2;
            } else if i == 1 {
                teleport[i].2 = 3;
            }
        }
    }

    for i in num_teleporters..teleport.len() {
        teleport[i].2 = 0;
    }
}

#[inline]
fn update_teleporter_pos(teleporters: &mut [(u8, u8, u8); MAX_TELEPORT], key_state: &mut bool) {
    if *key_state {
        if let Some((current, target)) = find_teleporter_targets(teleporters) {
            let target_pos = (teleporters[target].0, teleporters[target].1);
            let current_pos = &mut teleporters[current];

            let close_enough_x =
                (current_pos.0 as i16 - target_pos.0 as i16).abs() <= TELEPORT_SPEED as i16;
            let close_enough_y =
                (current_pos.1 as i16 - target_pos.1 as i16).abs() <= TELEPORT_SPEED as i16;

            if close_enough_x && close_enough_y {
                current_pos.0 = target_pos.0;
                current_pos.1 = target_pos.1;
                teleporters[current].2 = 0;
                teleporters[target].2 = 2;
                if target + 1 < teleporters.len() && teleporters[target + 1].2 != 0 {
                    teleporters[target + 1].2 = 3;
                }
                *key_state = false;
            } else {
                if current_pos.0 < target_pos.0 {
                    current_pos.0 += TELEPORT_SPEED;
                } else if current_pos.0 > target_pos.0 {
                    current_pos.0 -= TELEPORT_SPEED;
                }

                if current_pos.1 < target_pos.1 {
                    current_pos.1 += TELEPORT_SPEED;
                } else if current_pos.1 > target_pos.1 {
                    current_pos.1 -= TELEPORT_SPEED;
                }
            }
        }
    }
}

fn find_teleporter_targets(teleporters: &[(u8, u8, u8); MAX_TELEPORT]) -> Option<(usize, usize)> {
    let mut current = None;
    let mut next_target = None;

    for (i, &tele) in teleporters.iter().enumerate() {
        if tele.2 == 2 {
            current = Some(i);
            break;
        }
    }

    if let Some(curr) = current {
        for tele in teleporters.iter().skip(curr + 1) {
            if tele.2 != 0 {
                next_target = Some(teleporters.iter().position(|&t| t == *tele).unwrap());
                break;
            }
        }
    }

    if current.is_some() && next_target.is_none() {
        next_target = teleporters.iter().position(|&t| t.2 != 0 && t.2 != 2);
    }

    match (current, next_target) {
        (Some(c), Some(t)) => Some((c, t)),
        _ => None,
    }
}

#[inline]
fn render_frame(buffer: &mut [u32; 255 * 255], teleporters: &mut [(u8, u8, u8); MAX_TELEPORT]) {
    buffer.fill(0);

    let mut draw_rect = |x: u8, y: u8, width: u8, height: u8, state: u32| {
        for dy in 0..height {
            for dx in 0..width {
                let index = usize::from(y + dy) * WIDTH as usize + usize::from(x + dx);
                if index < buffer.len() {
                    buffer[index] = state;
                }
            }
        }
    };

    for teleport in teleporters.iter() {
        let state = match teleport.2 {
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 0,
        };

        draw_rect(teleport.0, teleport.1, TELEPORT_SIZE, TELEPORT_SIZE, state);
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
