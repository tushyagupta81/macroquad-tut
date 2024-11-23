use macroquad::prelude::*;

#[macroquad::main("My Game!")]
async fn main() {
    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;
    loop {
        clear_background(DARKPURPLE);

        if is_key_down(KeyCode::Left) {
            x -= 0.1;
        }
        if is_key_down(KeyCode::Right) {
            x += 0.1;
        }
        if is_key_down(KeyCode::Up) {
            y -= 0.1;
        }
        if is_key_down(KeyCode::Down) {
            y += 0.1;
        }
        draw_circle(x, y, 16.0, YELLOW);

        next_frame().await
    }
}
