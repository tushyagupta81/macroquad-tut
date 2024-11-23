use macroquad::prelude::*;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
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
    let mut circle = Shape {
        size: 16.0,
        speed: MAXSPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
    };

    let mut gameover = false;

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
            circle.x = clamp(circle.x, 0.0, screen_width());
            circle.y = clamp(circle.y, 0.0, screen_height());

            if rand::gen_range(0, 99) >= 95 {
                let size = rand::gen_range(16.0, 64.0);
                squares.push(Shape {
                    size,
                    speed: rand::gen_range(50.0, 150.0),
                    x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                    y: -size,
                })
            }
            for square in &mut squares {
                square.y += square.speed * delta;
            }
            squares.retain(|square| square.y < screen_height() + square.size);
        }
        if squares.iter().any(|square| circle.collide(square)) {
            gameover = true;
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
        }
        if gameover && is_key_pressed(KeyCode::Space) {
            squares.clear();
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            gameover = false;
        }
        next_frame().await
    }
}
