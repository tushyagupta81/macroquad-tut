use macroquad::audio::{load_sound, play_sound, play_sound_once, PlaySoundParams, Sound};
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::experimental::collections::storage;
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, Skin};
use macroquad_particles::{self as particles, AtlasConfig, Emitter, EmitterConfig};
use std::fs;

enum GameState {
    GameOver,
    MainMenu,
    Playing,
    Paused,
}

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
}

impl Shape {
    fn collide(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            h: self.size,
            w: self.size,
        }
    }
}

struct Resources {
    ship_texture: Texture2D,
    bullet_texture: Texture2D,
    explosion_texture: Texture2D,
    enemy_small_texture: Texture2D,
    enemy_medium_texture: Texture2D,
    enemy_big_texture: Texture2D,
    theme_music: Sound,
    sound_explosion: Sound,
    sound_laser: Sound,
    ui_skin: Skin,
}

impl Resources {
    async fn new() -> Result<Resources, macroquad::Error> {
        let ship_texture: Texture2D = load_texture("ship.png").await?;
        ship_texture.set_filter(FilterMode::Nearest);
        let bullet_texture: Texture2D = load_texture("laser-bolts.png").await?;
        bullet_texture.set_filter(FilterMode::Nearest);
        let explosion_texture: Texture2D = load_texture("explosion.png").await?;
        explosion_texture.set_filter(FilterMode::Nearest);
        let enemy_small_texture: Texture2D = load_texture("enemy-small.png").await?;
        enemy_small_texture.set_filter(FilterMode::Nearest);
        let enemy_medium_texture: Texture2D = load_texture("enemy-medium.png").await?;
        enemy_medium_texture.set_filter(FilterMode::Nearest);
        let enemy_big_texture: Texture2D = load_texture("enemy-big.png").await?;
        enemy_big_texture.set_filter(FilterMode::Nearest);
        build_textures_atlas();

        let theme_music = load_sound("8bit-spaceshooter.ogg").await?;
        let sound_explosion = load_sound("explosion.wav").await?;
        let sound_laser = load_sound("laser.wav").await?;

        let window_background = load_image("window_background.png").await?;
        let button_background = load_image("button_background.png").await?;
        let button_clicked_background = load_image("button_clicked_background.png").await?;
        let font = load_file("atari_games.ttf").await?;

        let window_style = root_ui()
            .style_builder()
            .background(window_background)
            .background_margin(RectOffset::new(32.0, 76.0, 44.0, 20.0))
            .margin(RectOffset::new(0.0, -40.0, 0.0, 0.0))
            .build();
        let button_style = root_ui()
            .style_builder()
            .background(button_background)
            .background_clicked(button_clicked_background)
            .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
            .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
            .font(&font)
            .unwrap()
            .text_color(WHITE)
            .font_size(64)
            .build();
        let label_style = root_ui()
            .style_builder()
            .font(&font)
            .unwrap()
            .text_color(WHITE)
            .font_size(28)
            .build();

        let ui_skin = Skin {
            window_style,
            button_style,
            label_style,
            ..root_ui().default_skin()
        };

        Ok(Resources {
            ship_texture,
            bullet_texture,
            explosion_texture,
            enemy_small_texture,
            enemy_medium_texture,
            enemy_big_texture,
            theme_music,
            sound_explosion,
            sound_laser,
            ui_skin,
        })
    }
    pub async fn load() -> Result<(), macroquad::Error> {
        let resources_loading = start_coroutine(async move {
            let resources = Resources::new().await.unwrap();
            storage::store(resources);
        });

        while !resources_loading.is_done() {
            clear_background(BLACK);
            let text = format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.) as usize) % 4)
            );
            draw_text(
                &text,
                screen_width() / 2. - 160.,
                screen_height() / 2.,
                40.,
                WHITE,
            );
            next_frame().await;
        }

        Ok(())
    }
}

fn particle_explosion() -> particles::EmitterConfig {
    particles::EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        explosiveness: 0.65,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 400.0,
        initial_velocity_randomness: 0.8,
        size: 16.0,
        size_randomness: 0.3,
        atlas: Some(AtlasConfig::new(5, 1, 0..)),
        ..Default::default()
    }
}

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

#[macroquad::main("My Game!")]
async fn main() -> Result<(), macroquad::Error> {
    const MAXSPEED: f32 = 200.0;

    let mut explosions: Vec<(Emitter, Vec2)> = vec![];
    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution".to_owned().as_str(), UniformType::Float2),
                UniformDesc::new(
                    "direction_modifier".to_owned().as_str(),
                    UniformType::Float1,
                ),
            ],
            ..Default::default()
        },
    )?;

    rand::srand(miniquad::date::now() as u64);
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut game_state = GameState::MainMenu;
    let mut circle = Shape {
        size: 16.0,
        speed: MAXSPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
    };

    let mut prev_time = get_time();
    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let mut old_high_score: u32 = high_score.clone();

    set_pc_assets_folder("assets");
    Resources::load().await?;
    let resources = storage::get::<Resources>();

    root_ui().push_skin(&resources.ui_skin);
    let window_size = vec2(370.0, 320.0);

    let mut enemy_small_sprite = AnimatedSprite::new(
        17,
        16,
        &[Animation {
            name: "enemy_small".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    let mut enemy_medium_sprite = AnimatedSprite::new(
        32,
        16,
        &[Animation {
            name: "enemy_medium".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    let mut enemy_big_sprite = AnimatedSprite::new(
        32,
        30,
        &[Animation {
            name: "enemy_big".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "bolt".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    bullet_sprite.set_animation(1);

    let mut ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );

    play_sound(
        &resources.theme_music,
        PlaySoundParams {
            looped: true,
            volume: 1.,
        },
    );

    loop {
        match game_state {
            GameState::MainMenu => {
                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width() / 2.0 - window_size.x / 2.0,
                        screen_height() / 2.0 - window_size.y / 2.0,
                    ),
                    window_size,
                    |ui| {
                        ui.label(vec2(80.0, -34.0), "Main Menu");
                        if ui.button(vec2(65.0, 25.0), "Play") {
                            squares.clear();
                            bullets.clear();
                            prev_time = get_time();
                            explosions.clear();
                            circle.x = screen_width() / 2.0;
                            circle.y = screen_height() / 2.0;
                            score = 0;
                            old_high_score = high_score;
                            game_state = GameState::Playing;
                        }
                        if ui.button(vec2(65.0, 125.0), "Quit") {
                            std::process::exit(0);
                        }
                    },
                );
            }
            GameState::GameOver => {
                let mut offset = 0.0;
                if old_high_score < high_score {
                    offset = 50.0;
                }
                let text = format!("New High Score {} !", high_score);
                let text_dimensions = measure_text(text.as_str(), None, 35, 1.0);

                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width() / 2.0 - 500.0 / 2.0,
                        screen_height() / 2.0 - (320.0 + offset) / 2.0,
                    ),
                    vec2(500.0, 320.0 + offset),
                    |ui| {
                        ui.label(vec2(155., -34.0), "Game Over!");
                        if ui.button(vec2(55.0, 25.0), "Main Menu") {
                            game_state = GameState::MainMenu;
                        }
                        if ui.button(vec2(135.0, 125.0), "Quit") {
                            std::process::exit(0);
                        }
                        if offset > 0.0 {
                            ui.label(
                                vec2(500.0 / 2.0 - text_dimensions.width / 2.0, 225.0),
                                text.as_str(),
                            );
                        }
                    },
                );
            }
            GameState::Paused => {
                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width() / 2.0 - window_size.x / 2.0,
                        screen_height() / 2.0 - window_size.y / 2.0,
                    ),
                    window_size,
                    |ui| {
                        ui.label(vec2(110.0, -34.0), "Paused");
                        if ui.button(vec2(35.0, 25.0), "Resume") {
                            game_state = GameState::Playing;
                        }
                        if ui.button(vec2(65.0, 125.0), "Quit") {
                            std::process::exit(0);
                        }
                    },
                );
            }
            GameState::Playing => {
                clear_background(BLACK);

                material.set_uniform("iResolution", (screen_width(), screen_height()));
                material.set_uniform("direction_modifier", direction_modifier);
                gl_use_material(&material);
                draw_texture_ex(
                    &render_target.texture,
                    0.,
                    0.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
                gl_use_default_material();

                let delta = get_frame_time();

                if is_key_down(KeyCode::Left) {
                    circle.x -= circle.speed * delta;
                    direction_modifier += 0.05 * delta;
                    ship_sprite.set_animation(1);
                }
                if is_key_down(KeyCode::Right) {
                    circle.x += circle.speed * delta;
                    direction_modifier -= 0.05 * delta;
                    ship_sprite.set_animation(2);
                }
                if is_key_down(KeyCode::Up) {
                    circle.y -= circle.speed * delta;
                }
                if is_key_down(KeyCode::Down) {
                    circle.y += circle.speed * delta;
                }
                if is_key_down(KeyCode::Space) {
                    if get_time() - prev_time > 0.2 {
                        prev_time = get_time();
                        bullets.push(Shape {
                            size: 32.0,
                            speed: circle.speed * 2.0,
                            x: circle.x,
                            y: circle.y - 24.0,
                            collided: false,
                        });
                        play_sound_once(&resources.sound_laser);
                    }
                }
                if is_key_down(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }
                circle.x = clamp(circle.x, 0.0, screen_width());
                circle.y = clamp(circle.y, 0.0, screen_height());

                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    squares.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        collided: false,
                    })
                }
                for square in &mut squares {
                    square.y += square.speed * delta;
                }
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta;
                }
                ship_sprite.update();
                bullet_sprite.update();
                enemy_small_sprite.update();
                enemy_medium_sprite.update();
                enemy_big_sprite.update();

                squares.retain(|square| square.y < screen_height() + square.size);
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
                bullets.retain(|bullet| !bullet.collided);
                explosions.retain(|(explosion, _)| explosion.config.emitting);

                if squares.iter().any(|square| circle.collide(square)) {
                    if score == high_score {
                        fs::write("highscore.dat", high_score.to_string()).ok();
                    }
                    game_state = GameState::GameOver;
                }

                for square in squares.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collide(square) {
                            bullet.collided = true;
                            square.collided = true;
                            score += square.size.round() as u32;
                            high_score = high_score.max(score);
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 3,
                                    texture: Some(resources.explosion_texture.clone()),
                                    ..particle_explosion()
                                }),
                                vec2(square.x, square.y),
                            ));
                            play_sound_once(&resources.sound_explosion);
                        }
                    }
                }

                let ship_frame = ship_sprite.frame();
                draw_texture_ex(
                    &resources.ship_texture,
                    circle.x - ship_frame.dest_size.x,
                    circle.y - ship_frame.dest_size.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ship_frame.dest_size * 2.0),
                        source: Some(ship_frame.source_rect),
                        ..Default::default()
                    },
                );

                let enemy_small_frame = enemy_small_sprite.frame();
                let enemy_medium_frame = enemy_medium_sprite.frame();
                let enemy_big_frame = enemy_big_sprite.frame();
                for square in &squares {
                    if square.size < 32.0 {
                        draw_texture_ex(
                            &resources.enemy_small_texture,
                            square.x - square.size / 2.0,
                            square.y - square.size / 2.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(square.size, square.size)),
                                source: Some(enemy_small_frame.source_rect),
                                ..Default::default()
                            },
                        );
                    } else if square.size > 48.0 {
                        draw_texture_ex(
                            &resources.enemy_big_texture,
                            square.x - square.size / 2.0,
                            square.y - square.size / 2.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(square.size, square.size)),
                                source: Some(enemy_big_frame.source_rect),
                                ..Default::default()
                            },
                        );
                    } else {
                        draw_texture_ex(
                            &resources.enemy_medium_texture,
                            square.x - square.size / 2.0,
                            square.y - square.size / 2.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(square.size, square.size)),
                                source: Some(enemy_medium_frame.source_rect),
                                ..Default::default()
                            },
                        );
                    }
                }
                for (explosion, coords) in explosions.iter_mut() {
                    explosion.draw(*coords);
                }

                let bullet_frame = bullet_sprite.frame();
                for bullet in &bullets {
                    draw_texture_ex(
                        &resources.bullet_texture,
                        bullet.x - bullet.size / 2.0,
                        bullet.y - bullet.size / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(bullet.size, bullet.size)),
                            source: Some(bullet_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }

                draw_text(
                    format!("Score: {}", score).as_str(),
                    10.0,
                    35.0,
                    25.0,
                    WHITE,
                );

                let highscore_text = format!("High score: {}", high_score);
                let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
                draw_text(
                    highscore_text.as_str(),
                    screen_width() - text_dimensions.width - 10.0,
                    35.0,
                    25.0,
                    WHITE,
                );
            }
        }
        next_frame().await
    }
}
