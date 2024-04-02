#![no_std]
use core::{iter::repeat_with, ptr};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

static mut TELEPORT: [(u8, u8, u8); MAX_TELEPORT] = [(0, 0, 0); MAX_TELEPORT];
const MAX_TELEPORT: usize = 12;
const TELEPORT_SIZE: u8 = 15;
const TELEPORT_SPEED: u8 = 5;

const GRID_WIDTH: usize = (WIDTH as usize) / TELEPORT_SIZE as usize;
const GRID_HEIGHT: usize = (HEIGHT as usize) / TELEPORT_SIZE as usize;

static mut PLAYER_MOVE: bool = false;

//https://blog.orhun.dev/zero-deps-random-in-rust/
#[inline]
fn rng(seed: u32, frame: u32) -> impl Iterator<Item = u32> {
    let mut random = seed + frame;
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
unsafe extern "C" fn game_loop(seed: u32, key_pressed: bool, frame: u32) {
    if key_pressed {
        PLAYER_MOVE = true;
    }

    if frame == 1 {
        TELEPORT.iter_mut().for_each(|t| *t = (0, 0, 0));
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
    teleport: &mut [(u8, u8, u8); MAX_TELEPORT],
    key_state: &mut bool,
) {
    let mut rng = rng(seed, frame);

    if frame == 1 {
        spawn_tele(teleport, &mut rng);
    }

    if *key_state {
        update_tele_pos(teleport, key_state);
    }
    render_frame(buffer, teleport);
}

#[inline]
fn spawn_tele(teleport: &mut [(u8, u8, u8); MAX_TELEPORT], rng: &mut impl Iterator<Item = u32>) {
    let teleporter_size = TELEPORT_SIZE as usize;
    let max_index_x = GRID_WIDTH - 1;
    let max_index_y = GRID_HEIGHT - 1;

    let num_teleporters =
        ((rng.next().unwrap() % (MAX_TELEPORT as u32 - 2)) as usize + 2).min(teleport.len());

    for i in 0..num_teleporters {
        let mut x = (rng.next().unwrap() as usize % max_index_x) * teleporter_size;
        let mut y = (rng.next().unwrap() as usize % max_index_y) * teleporter_size;

        x = x.min((WIDTH as usize) - teleporter_size);
        y = y.min((HEIGHT as usize) - teleporter_size);

        teleport[i] = (
            x as u8,
            y as u8,
            if i == 0 {
                1
            } else if i == 1 {
                2
            } else {
                3
            },
        );
    }

    for i in num_teleporters..teleport.len() {
        teleport[i].2 = 0;
    }
}

#[inline]
fn update_tele_pos(teleporters: &mut [(u8, u8, u8); MAX_TELEPORT], key_state: &mut bool) {
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
                teleporters[current].2 = 4;
                teleporters[target].2 = 1;
                if target + 1 < teleporters.len() && teleporters[target + 1].2 != 0 {
                    teleporters[target + 1].2 = 2;
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
        if tele.2 == 1 {
            current = Some(i);
            break;
        }
    }

    if let Some(curr) = current {
        for tele in teleporters.iter().skip(curr + 1) {
            if tele.2 != 0 && tele.2 != 4 {
                next_target = Some(teleporters.iter().position(|&t| t == *tele).unwrap());
                break;
            }
        }
    }

    if current.is_some() && next_target.is_none() {
        next_target = teleporters
            .iter()
            .position(|&t| t.2 != 0 && t.2 != 4 && t.2 != 1);
    }

    match (current, next_target) {
        (Some(c), Some(t)) => Some((c, t)),
        _ => None,
    }
}

#[inline]
fn render_frame(buffer: &mut [u32; 255 * 255], teleporters: &mut [(u8, u8, u8); MAX_TELEPORT]) {
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

    for state in (0..=4).rev() {
        for teleport in teleporters.iter() {
            if teleport.2 == state {
                draw_rect(
                    teleport.0,
                    teleport.1,
                    TELEPORT_SIZE,
                    TELEPORT_SIZE,
                    state.into(),
                );
            }
        }
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
