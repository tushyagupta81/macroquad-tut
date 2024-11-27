use macroquad::prelude::*;
use std::fs;

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

#[macroquad::main("My Game!")]
async fn main() {
    const MAXSPEED: f32 = 200.0;

    rand::srand(miniquad::date::now() as u64);
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut circle = Shape {
        size: 16.0,
        speed: MAXSPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
    };

    let mut gameover = false;
    let mut prev_time = get_time();
    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let old_high_score: u32 = high_score.clone();

    loop {
        if !gameover {
            clear_background(DARKPURPLE);
            let delta = get_frame_time();

            if is_key_down(KeyCode::Left) {
                circle.x -= circle.speed * delta;
            }
            if is_key_down(KeyCode::Right) {
                circle.x += circle.speed * delta;
            }
            if is_key_down(KeyCode::Up) {
                circle.y -= circle.speed * delta;
            }
            if is_key_down(KeyCode::Down) {
                circle.y += circle.speed * delta;
            }
            if is_key_down(KeyCode::Space) {
                if get_time() - prev_time > 0.5 {
                    prev_time = get_time();
                    bullets.push(Shape {
                        size: 5.0,
                        speed: circle.speed * 2.0,
                        x: circle.x,
                        y: circle.y,
                        collided: false,
                    })
                }
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
            squares.retain(|square| square.y < screen_height() + square.size);
            squares.retain(|square| !square.collided);
            bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
            bullets.retain(|bullet| !bullet.collided);
        }
        if squares.iter().any(|square| circle.collide(square)) {
            if score == high_score {
                fs::write("highscore.dat", high_score.to_string()).ok();
            }
            gameover = true;
        }
        for square in squares.iter_mut() {
            for bullet in bullets.iter_mut() {
                if bullet.collide(square) {
                    bullet.collided = true;
                    square.collided = true;
                    score += square.size.round() as u32;
                    high_score = high_score.max(score);
                }
            }
        }
        draw_circle(circle.x, circle.y, 16.0, YELLOW);
        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                GREEN,
            );
        }
        for bullet in &bullets {
            draw_circle(bullet.x, bullet.y, bullet.size / 2.0, RED);
        }
        if gameover {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(
                text,
                screen_width() / 2.0 - text_dimensions.width / 2.0,
                screen_height() / 2.0,
                50.0,
                RED,
            );
            if old_high_score < high_score {
                let text = format!("New high Score: {}", high_score);
                let text_dimensions = measure_text(text.as_str(), None, 36, 1.0);
                draw_text(
                    text.as_str(),
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 1.7,
                    36.0,
                    RED,
                );
            }
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
        if gameover && is_key_pressed(KeyCode::Space) {
            squares.clear();
            bullets.clear();
            prev_time = get_time();
            score = 0;
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            gameover = false;
        }
        next_frame().await
    }
}
