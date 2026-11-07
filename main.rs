use macroquad::prelude::*;

const ARENA: f32 = 25.0;
const GAME_TIME: f32 = 60.0;
const MOVE_SPEED: f32 = 8.0;
const LOOK_SPEED: f32 = 0.1;
const GRAVITY: f32 = 25.0;
const JUMP_SPEED: f32 = 9.0;
const COIN_COUNT: usize = 12;

fn window_conf() -> Conf {
    Conf {
        window_title: "Coin Rush 3D".to_owned(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

struct Game {
    pos: Vec3,   // x, z = ground position, y = height above floor
    vel_y: f32,
    yaw: f32,
    pitch: f32,
    coins: Vec<Vec3>,
    pillars: Vec<Vec3>,
    score: u32,
    time_left: f32,
    spin: f32,
}

impl Game {
    fn new() -> Self {
        let coins = (0..COIN_COUNT).map(|_| random_coin()).collect();
        let pillars = (0..10)
            .map(|_| {
                vec3(
                    rand::gen_range(-ARENA + 3.0, ARENA - 3.0),
                    rand::gen_range(1.5, 4.0),
                    rand::gen_range(-ARENA + 3.0, ARENA - 3.0),
                )
            })
            .collect();
        Self {
            pos: vec3(0.0, 0.0, 0.0),
            vel_y: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            coins,
            pillars,
            score: 0,
            time_left: GAME_TIME,
            spin: 0.0,
        }
    }
}

fn random_coin() -> Vec3 {
    vec3(
        rand::gen_range(-ARENA + 2.0, ARENA - 2.0),
        1.0,
        rand::gen_range(-ARENA + 2.0, ARENA - 2.0),
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);

    let mut game = Game::new();
    let mut last_mouse = Vec2::from(mouse_position());
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        let dt = get_frame_time();
        let playing = game.time_left > 0.0;

        // ---------- Mouse look ----------
        let mouse = Vec2::from(mouse_position());
        let delta = mouse - last_mouse;
        last_mouse = mouse;

        if playing {
            game.yaw += delta.x * dt * LOOK_SPEED;
            game.pitch -= delta.y * dt * LOOK_SPEED;
            game.pitch = game.pitch.clamp(-1.3, 1.3);

            // ---------- Movement (WASD) ----------
            let forward = vec3(game.yaw.cos(), 0.0, game.yaw.sin());
            let right = vec3(-game.yaw.sin(), 0.0, game.yaw.cos());
            let mut dir = Vec3::ZERO;
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                dir += forward;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                dir -= forward;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                dir += right;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                dir -= right;
            }
            if dir.length() > 0.0 {
                game.pos += dir.normalize() * MOVE_SPEED * dt;
            }
            game.pos.x = game.pos.x.clamp(-ARENA + 0.5, ARENA - 0.5);
            game.pos.z = game.pos.z.clamp(-ARENA + 0.5, ARENA - 0.5);

            // ---------- Jump and gravity ----------
            let on_ground = game.pos.y <= 0.0;
            if on_ground && is_key_pressed(KeyCode::Space) {
                game.vel_y = JUMP_SPEED;
            }
            game.vel_y -= GRAVITY * dt;
            game.pos.y += game.vel_y * dt;
            if game.pos.y < 0.0 {
                game.pos.y = 0.0;
                game.vel_y = 0.0;
            }

            // ---------- Collect coins ----------
            let player_center = vec3(game.pos.x, game.pos.y + 1.0, game.pos.z);
            let mut collected = 0;
            for coin in game.coins.iter_mut() {
                if coin.distance(player_center) < 1.5 {
                    *coin = random_coin();
                    collected += 1;
                }
            }
            game.score += collected;

            game.time_left -= dt;
            game.spin += dt * 2.0;
        } else if is_key_pressed(KeyCode::R) {
            game = Game::new();
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // ---------- Render 3D ----------
        clear_background(Color::new(0.45, 0.65, 0.9, 1.0));

        let eye = vec3(game.pos.x, game.pos.y + 1.7, game.pos.z);
        let front = vec3(
            game.yaw.cos() * game.pitch.cos(),
            game.pitch.sin(),
            game.yaw.sin() * game.pitch.cos(),
        )
        .normalize();

        set_camera(&Camera3D {
            position: eye,
            target: eye + front,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 70.0_f32.to_radians(),
            ..Default::default()
        });

        // Floor and grid
        draw_plane(Vec3::ZERO, vec2(ARENA, ARENA), None, Color::new(0.25, 0.55, 0.3, 1.0));
        draw_grid(
            (ARENA * 2.0) as u32,
            1.0,
            Color::new(0.0, 0.0, 0.0, 0.3),
            Color::new(0.0, 0.0, 0.0, 0.15),
        );

        // Pillars (decoration)
        for p in &game.pillars {
            draw_cube(vec3(p.x, p.y / 2.0, p.z), vec3(1.5, p.y, 1.5), None, GRAY);
            draw_cube_wires(vec3(p.x, p.y / 2.0, p.z), vec3(1.5, p.y, 1.5), DARKGRAY);
        }

        // Spinning, bobbing coins
        for (i, c) in game.coins.iter().enumerate() {
            let bob = (game.spin + i as f32).sin() * 0.2;
            let pos = vec3(c.x, c.y + bob, c.z);
            draw_sphere(pos, 0.4, None, GOLD);
            draw_cube(pos, vec3(0.15, 1.0, 0.15), None, ORANGE);
        }

        // ---------- HUD ----------
        set_default_camera();
        draw_text(&format!("Score: {}", game.score), 20.0, 40.0, 40.0, WHITE);
        draw_text(
            &format!("Time: {:.0}", game.time_left.max(0.0)),
            20.0,
            80.0,
            40.0,
            WHITE,
        );
        // Crosshair
        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 3.0, WHITE);

        if !playing {
            draw_text(
                &format!("TIME'S UP!  Final score: {}", game.score),
                screen_width() / 2.0 - 260.0,
                screen_height() / 2.0 - 20.0,
                50.0,
                YELLOW,
            );
            draw_text(
                "Press R to restart, Esc to quit",
                screen_width() / 2.0 - 200.0,
                screen_height() / 2.0 + 30.0,
                32.0,
                WHITE,
            );
        } else {
            draw_text(
                "WASD move | Mouse look | Space jump | Esc quit",
                20.0,
                screen_height() - 20.0,
                24.0,
                WHITE,
            );
        }

        next_frame().await;
    }
}
