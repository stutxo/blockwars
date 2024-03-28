#![no_std]
use core::sync::atomic::{AtomicU32, Ordering};
static FRAME: AtomicU32 = AtomicU32::new(0);

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const PLAYER_SPEED: usize = 2;
const ENEMY_SIZE: usize = 15;
const PLAYER_SIZE: usize = 10;
const BULLET_SIZE: usize = 3;

static mut GAME_OVER: bool = false;

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[no_mangle]
pub unsafe extern "C" fn game_loop() -> u32 {
    if !GAME_OVER {
        update_enemy_pos();
        render_frame_safe(&mut BUFFER);
        1
    } else {
        0
    }
}

const MAX_ENEMIES: usize = 600;
const ENEMIES_NONE: core::option::Option<Enemy> = None;
static mut ENEMIES: [Option<Enemy>; MAX_ENEMIES] = [ENEMIES_NONE; MAX_ENEMIES];
struct Enemy {
    x: usize,
    y: usize,
    frame_counter: usize,
}

impl Enemy {
    fn new(x: usize, y: usize) -> Self {
        Enemy {
            x,
            y,
            frame_counter: 0,
        }
    }
}

static mut PLAYER: [Option<Player>; 1] = [None; 1];
struct Player {
    x: usize,
    y: usize,
}

impl Player {
    fn new() -> Self {
        Player {
            x: (WIDTH as u32 / 2 - 5) as usize,
            y: (HEIGHT as u32 / 2 - 5) as usize,
        }
    }
}

const MAX_BULLETS: usize = 500;
const BULLETS_NONE: core::option::Option<Bullet> = None;
static mut BULLETS: [Option<Bullet>; MAX_BULLETS] = [BULLETS_NONE; MAX_BULLETS];
struct Bullet {
    x: usize,
    y: usize,
}

impl Bullet {
    fn new(x: usize, y: usize) -> Self {
        Bullet { x, y }
    }
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
pub extern "C" fn spawn() {
    for _ in 0..21 {
        spawn_enemy();
    }

    spawn_player();
}

fn spawn_player() {
    unsafe {
        for slot in PLAYER.iter_mut() {
            if slot.is_none() {
                *slot = Some(Player::new());
            }
        }
    }
}

fn spawn_bullet(x: usize, y: usize) {
    unsafe {
        for slot in BULLETS.iter_mut() {
            if slot.is_none() {
                *slot = Some(Bullet::new(x, y));
                break;
            }
        }
    }
}

fn spawn_enemy() {
    unsafe {
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        let mut rng = Rng::new(123 + f);

        for slot in ENEMIES.iter_mut() {
            if slot.is_none() {
                let position = match rng.rand() % 4 {
                    0 => (rng.rand() % (WIDTH as u32 - ENEMY_SIZE as u32), 0),
                    1 => (
                        WIDTH as u32 - ENEMY_SIZE as u32,
                        rng.rand() % (HEIGHT as u32 - ENEMY_SIZE as u32),
                    ),
                    2 => (
                        rng.rand() % (WIDTH as u32 - ENEMY_SIZE as u32),
                        HEIGHT as u32 - ENEMY_SIZE as u32,
                    ),
                    3 => (0, rng.rand() % (HEIGHT as u32 - ENEMY_SIZE as u32)),
                    _ => continue,
                };

                *slot = Some(Enemy::new(position.0 as usize, position.1 as usize));
                break;
            }
        }
    }
}

fn render_frame_safe(buffer: &mut [u32; WIDTH * HEIGHT]) {
    for i in 0..(WIDTH * HEIGHT) {
        buffer[i] = 0xFF_00_00_00;
    }

    unsafe {
        for player_entity in PLAYER.iter() {
            if let Some(player) = player_entity {
                for y in player.y..(player.y + PLAYER_SIZE) {
                    for x in player.x..(player.x + PLAYER_SIZE) {
                        buffer[y * WIDTH + x] = 0xFFFFFF;
                    }
                }
            }
        }
    }

    unsafe {
        for bullet_entity in BULLETS.iter() {
            if let Some(bullet) = bullet_entity {
                for y in bullet.y..(bullet.y + BULLET_SIZE) {
                    for x in bullet.x..(bullet.x + BULLET_SIZE) {
                        buffer[y * WIDTH + x] = 0xFFFFFF;
                    }
                }
            }
        }
    }

    unsafe {
        for enemy_entity in ENEMIES.iter() {
            if let Some(enemy) = enemy_entity {
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

    if f % 600 == 0 && f < 1337 {
        spawn();
    }

    unsafe {
        for enemy_entity in ENEMIES.iter_mut() {
            for player_entity in PLAYER.iter() {
                if let Some(player) = player_entity {
                    if let Some(enemy) = enemy_entity {
                        if enemy.frame_counter > 0 {
                            enemy.frame_counter -= 1;
                        }

                        if enemy.frame_counter == 0 {
                            if enemy.x > player.x {
                                enemy.x = enemy.x.saturating_sub(1);
                            } else if enemy.x < player.x {
                                enemy.x += 1;
                            }

                            if enemy.y > player.y {
                                enemy.y = enemy.y.saturating_sub(1);
                            } else if enemy.y < player.y {
                                enemy.y += 1;
                            }

                            if (enemy.x >= player.x - PLAYER_SIZE
                                && enemy.x <= player.x + PLAYER_SIZE)
                                && (enemy.y >= player.y - PLAYER_SIZE
                                    && enemy.y <= player.y + PLAYER_SIZE)
                            {
                                GAME_OVER = true;
                            }

                            enemy.frame_counter = rng.rand_in_range(1, 8) as usize;
                        }
                    }
                }
            }
        }

        for enemy_entity in ENEMIES.iter_mut() {
            if let Some(enemy) = enemy_entity {
                // Iterate through all bullets to check for collisions.
                let mut enemy_hit = false;
                for bullet_entity in BULLETS.iter_mut() {
                    if let Some(bullet) = bullet_entity {
                        // Check for overlap in both the X and Y directions.
                        // Considering the size of the enemy and the bullet for collision detection.

                        if enemy.x < bullet.x + BULLET_SIZE
                            && enemy.x + ENEMY_SIZE > bullet.x
                            && enemy.y < bullet.y + BULLET_SIZE
                            && enemy.y + ENEMY_SIZE > bullet.y
                        {
                            // Collision detected, remove the bullet by setting it to None.
                            *bullet_entity = None;
                            enemy_hit = true;
                        }
                    }
                }
                if enemy_hit {
                    *enemy_entity = None;
                    break;
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

    for player in PLAYER.iter_mut() {
        if let Some(player) = player {
            match key {
                Key::Left => {
                    if player.x >= PLAYER_SPEED {
                        player.x -= PLAYER_SPEED;
                    }
                }
                Key::Right => {
                    if player.x + 10 + PLAYER_SPEED <= WIDTH {
                        player.x += PLAYER_SPEED;
                    }
                }
                Key::Up => {
                    if player.y >= PLAYER_SPEED {
                        player.y -= PLAYER_SPEED;
                    }
                }
                Key::Down => {
                    if player.y + 10 + PLAYER_SPEED <= HEIGHT {
                        player.y += PLAYER_SPEED;
                    }
                }
            }
            spawn_bullet(player.x + 5, player.y + 5);
        }
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
