use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{Rect};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use macroquad_tiled_redux::{Map};


#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(Path::new("assets/grass/map1.tmx"))
        .await
        .expect("Error loading map");

    println!("{:?}", tilemap);

    let mut camera = (
        tilemap.map.width * tilemap.map.tile_width,
        tilemap.map.height * tilemap.map.tile_height,
    );

    let mut zoom = 3.0;

    loop {
        clear_background(LIGHTGRAY);

        let dest = Rect::new(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
        );

        // dest.move_to(vec2(camera.0 - screen_width()/2, camera.1 - screen_height()/2));

        tilemap.draw_tiles(0, dest, None);

        if is_key_down(KeyCode::Q) {
            break;
        }
        if is_key_pressed(KeyCode::KpAdd) || is_key_pressed(KeyCode::KpMultiply) {
            zoom += 1.0;
            // dest.scale(1.25, 1.25);
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract)) && zoom >= 2.0 {
            zoom += 1.0;
            // dest.scale(0.8, 0.8);
        }
        if is_key_pressed(KeyCode::Key0) || is_key_pressed(KeyCode::Kp0) {
            zoom = 1.0;
        }
        if is_key_pressed(KeyCode::Left) {
            camera = (camera.0 - 2, camera.1);
        }
        if is_key_pressed(KeyCode::Right) {
            camera = (camera.0 + 2, camera.1);
        }
        if is_key_pressed(KeyCode::Up) {
            camera = (camera.0, camera.1 - 2);
        }
        if is_key_pressed(KeyCode::Down) {
            camera = (camera.0, camera.1 + 2);
        }

        next_frame().await
    }
}
