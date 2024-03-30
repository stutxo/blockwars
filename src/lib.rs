#![no_std]
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const PLAYER_SPEED: usize = 2;
const ENEMY_SIZE: usize = 5;
const PLAYER_SIZE: usize = 10;
const WALL_SIZE: usize = 3;
const SEED: u32 = 0x1331;
const ENEMIES_PER_WAVE: u32 = 5;
const MAX_ENEMIES: usize = 1000;
const ENEMIES_NONE: core::option::Option<Enemy> = None;
const MAX_WALL: usize = 30000;
const WALL_NONE: core::option::Option<Wall> = None;

static mut PLAYER: [Option<Player>; 1] = [None; 1];
static mut ENEMIES: [Option<Enemy>; MAX_ENEMIES] = [ENEMIES_NONE; MAX_ENEMIES];
static mut WALL: [Option<Wall>; MAX_WALL] = [WALL_NONE; MAX_WALL];

static GAME_OVER: AtomicBool = AtomicBool::new(false);
static FRAME: AtomicU32 = AtomicU32::new(0);
static KEY_STATE: AtomicUsize = AtomicUsize::new(0);

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

struct Wall {
    x: usize,
    y: usize,
}

impl Wall {
    fn new(x: usize, y: usize) -> Self {
        Wall { x, y }
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
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[no_mangle]
pub unsafe extern "C" fn key_pressed(value: usize) {
    KEY_STATE.store(value, Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn game_loop() -> u32 {
    if !GAME_OVER.load(Ordering::Relaxed) {
        frame_safe(&mut BUFFER, &mut ENEMIES, &mut PLAYER, &mut WALL);
        1
    } else {
        0
    }
}

fn frame_safe(
    buffer: &mut [u32; WIDTH * HEIGHT],
    enemies: &mut [Option<Enemy>; MAX_ENEMIES],
    player: &mut [Option<Player>; 1],
    wall: &mut [Option<Wall>; MAX_WALL],
) {
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    let mut rng = Rng::new(SEED + f);
    if player[0].is_none() {
        spawn_player(player);
    }
    if f % 3 == 0 && f < 1337 {
        spawn_enemy(enemies, &mut rng);
    }
    update_player_pos(player, wall);
    update_enemy_pos(enemies, player, wall, &mut rng);
    render_frame(buffer, enemies, player, wall);
}

fn spawn_player(player: &mut [Option<Player>; 1]) {
    for slot in player.iter_mut() {
        if slot.is_none() {
            *slot = Some(Player::new());
        }
    }
}

fn spawn_enemy(enemies: &mut [Option<Enemy>; MAX_ENEMIES], rng: &mut Rng) {
    for _ in 0..ENEMIES_PER_WAVE {
        for slot in enemies.iter_mut() {
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

fn update_player_pos(player: &mut [Option<Player>; 1], wall: &mut [Option<Wall>; MAX_WALL]) {
    let key_press = KEY_STATE.load(Ordering::Relaxed);
    let key = match key_press {
        1 => Key::Left,
        2 => Key::Right,
        3 => Key::Up,
        4 => Key::Down,
        _ => return,
    };

    for player in player.iter_mut() {
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

            for slot in wall.iter_mut() {
                if slot.is_none() {
                    *slot = Some(Wall::new(
                        player.x + PLAYER_SIZE / 2,
                        player.y + PLAYER_SIZE / 2,
                    ));
                    break;
                }
            }
        }
    }
}

fn update_enemy_pos(
    enemies: &mut [Option<Enemy>; MAX_ENEMIES],
    player: &mut [Option<Player>; 1],
    wall: &mut [Option<Wall>; MAX_WALL],
    rng: &mut Rng,
) {
    for enemy_entity in enemies.iter_mut() {
        for player_entity in player.iter() {
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

                        if (enemy.x < player.x + PLAYER_SIZE)
                            && (enemy.x + ENEMY_SIZE > player.x)
                            && (enemy.y < player.y + PLAYER_SIZE)
                            && (enemy.y + ENEMY_SIZE > player.y)
                        {
                            GAME_OVER.store(true, Ordering::Relaxed);
                        }

                        enemy.frame_counter = rng.rand_in_range(1, 8) as usize;
                    }
                }
            }
        }
    }

    for enemy_entity in enemies.iter_mut() {
        if let Some(enemy) = enemy_entity {
            let mut enemy_hit = false;
            for wall_entity in wall.iter_mut() {
                if let Some(wall) = wall_entity {
                    if enemy.x < wall.x + WALL_SIZE
                        && enemy.x + ENEMY_SIZE > wall.x
                        && enemy.y < wall.y + WALL_SIZE
                        && enemy.y + ENEMY_SIZE > wall.y
                    {
                        *wall_entity = None;
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

fn render_frame(
    buffer: &mut [u32; WIDTH * HEIGHT],
    enemies: &[Option<Enemy>; MAX_ENEMIES],
    player: &[Option<Player>; 1],
    wall: &[Option<Wall>; MAX_WALL],
) {
    for i in 0..(WIDTH * HEIGHT) {
        buffer[i] = 0xFF_00_00_00;
    }

    for player_entity in player.iter() {
        if let Some(player) = player_entity {
            for y in player.y..(player.y + PLAYER_SIZE) {
                for x in player.x..(player.x + PLAYER_SIZE) {
                    buffer[y * WIDTH + x] = 0xFFFFFF;
                }
            }
        }
    }

    for wall_entity in wall.iter() {
        if let Some(wall) = wall_entity {
            for y in wall.y..(wall.y + WALL_SIZE) {
                for x in wall.x..(wall.x + WALL_SIZE) {
                    buffer[y * WIDTH + x] = 0xFFFFFF;
                }
            }
        }
    }

    for enemy_entity in enemies.iter() {
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

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
