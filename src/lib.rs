#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};

static SQUARE_POS_X: AtomicU32 = AtomicU32::new(WIDTH as u32 / 2 - 5);
static SQUARE_POS_Y: AtomicU32 = AtomicU32::new(HEIGHT as u32 / 2 - 5);

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const SPEED: u32 = 10;

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[no_mangle]
pub unsafe extern "C" fn go() {
    render_frame_safe(&mut BUFFER)
}

pub enum Key {
    Left,
    Right,
    Up,
    Down,
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
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
