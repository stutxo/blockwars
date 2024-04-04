#![no_std]
use core::{iter::repeat_with, ptr};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

const TELEPORT_NONE: core::option::Option<(f32, f32, f32)> = None;
static mut TELEPORT: [Option<(f32, f32, f32)>; MAX_TELEPORT] = [TELEPORT_NONE; MAX_TELEPORT];

const MAX_TELEPORT: usize = 12;
const TELEPORT_SIZE: u8 = 10;
const TELEPORT_SPEED: f32 = 10.;

const GRID_WIDTH: usize = (WIDTH as usize) / TELEPORT_SIZE as usize;
const GRID_HEIGHT: usize = (HEIGHT as usize) / TELEPORT_SIZE as usize;

static mut PLAYER_MOVE: bool = false;

//https://blog.orhun.dev/zero-deps-random-in-rust/
#[inline]
fn rng(seed: u32, frame: u32) -> impl Iterator<Item = u32> {
    let mut random = seed ^ frame;
    repeat_with(move || {
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
unsafe extern "C" fn blockwars(seed: u32, key_pressed: bool, frame: u32) {
    if key_pressed {
        PLAYER_MOVE = true;
    }

    if frame == 1 {
        TELEPORT.iter_mut().for_each(|t| *t = None);
        BUFFER.iter_mut().for_each(|b| *b = 0);
    }

    frame_safe(
        &mut *ptr::addr_of_mut!(BUFFER),
        frame,
        seed,
        &mut *ptr::addr_of_mut!(TELEPORT),
        &mut *ptr::addr_of_mut!(PLAYER_MOVE),
    );
}

//no unsafe code below this point
#[inline]
fn frame_safe(
    buffer: &mut [u32; 255 * 255],
    frame: u32,
    seed: u32,
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    key_state: &mut bool,
) {
    let mut rng = rng(seed, frame);

    if frame == 1 {
        spawn_tele(teleporters, &mut rng);
    }

    if *key_state {
        update_tele_pos(teleporters, key_state);
    }
    render_frame(buffer, teleporters);
}

#[inline]
fn spawn_tele(
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    rng: &mut impl Iterator<Item = u32>,
) {
    let teleporter_size = TELEPORT_SIZE as usize;
    let max_index_x = GRID_WIDTH - 1;
    let max_index_y = GRID_HEIGHT - 1;

    let num_teleporters = match rng.next() {
        Some(random_value) => {
            ((random_value % (MAX_TELEPORT as u32 - 2)) as usize + 2).min(teleporters.len())
        }
        None => 2,
    };

    for i in 0..num_teleporters {
        let x = match rng.next() {
            Some(random_value) => {
                let mut x = (random_value as usize % max_index_x) * teleporter_size;
                x = x.min((WIDTH as usize) - teleporter_size);
                x as f32
            }
            None => 0.0,
        };

        let y = match rng.next() {
            Some(random_value) => {
                let mut y = (random_value as usize % max_index_y) * teleporter_size;
                y = y.min((HEIGHT as usize) - teleporter_size);
                y as f32
            }
            None => 0.0,
        };

        teleporters[i] = Some((
            x,
            y,
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
fn update_tele_pos(
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
    key_state: &mut bool,
) {
    if *key_state {
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

                    *key_state = false;
                } else {
                    let dir_x = dx / distance;
                    let dir_y = dy / distance;

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
    buffer: &mut [u32; 255 * 255],
    teleporters: &mut [Option<(f32, f32, f32)>; MAX_TELEPORT],
) {
    let mut draw_rect = |x: f32, y: f32, width: u8, height: u8, state: u32| {
        for dy in 0..height {
            for dx in 0..width {
                let index =
                    (y + f32::from(dy)) as usize * WIDTH as usize + (x + f32::from(dx)) as usize;
                if index < buffer.len() {
                    buffer[index] = state;
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
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
