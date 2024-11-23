use macroquad::prelude::*;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
}

#[macroquad::main("My Game!")]
async fn main() {
    const MAXSPEED: f32 = 200.0;
    let mut squares = vec![];
    let mut circle = Shape {
        size: 16.0,
        speed: MAXSPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
    };
    loop {
        clear_background(DARKPURPLE);
        let delta = get_frame_time();

        if rand::gen_range(0, 99) >= 95 {
            let size = rand::gen_range(16.0, 64.0);
            squares.push(Shape {
                size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size / 2.0, screen_width() / 2.0 - size / 2.0),
                y: -size,
            })
        }

        for square in &mut squares {
            square.y = square.speed * delta;
        }
        squares.retain(|square| square.y < screen_height() + square.size);

        for square in &squares {
            draw_rectangle(square.x, square.y, square.size, square.size, GREEN);
        }

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

        draw_circle(circle.x, circle.y, 16.0, YELLOW);

        next_frame().await
    }
}