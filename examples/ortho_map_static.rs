use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{Rect, vec2};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use macroquad_tiled_redux::{Map};


#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(Path::new("assets/grass/map1.tmx"))
        .await
        .expect("Error loading map");

    println!("{:?}", tilemap);

    let map_size = Rect::new(
        0.0,
        0.0,
        (tilemap.map.width * tilemap.map.tile_width) as f32,
        (tilemap.map.height * tilemap.map.tile_height) as f32);

    // in world pixels. Starting in the middle of the map.
    let mut camera = (map_size.w / 2.0, map_size.h / 2.0);

    let mut zoom = 2.0;

    loop {
        clear_background(LIGHTGRAY);

        let screen = Rect::new(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
        );

        let mut source = screen;
        let mut dest = screen;

        source.move_to(vec2(
            camera.0 - screen_width() / zoom / 2.0,
            camera.1 - screen_height() / zoom / 2.0));

        let mut source_in_tiles = Rect::new(
            source.x / tilemap.map.tile_width as f32,
            source.y / tilemap.map.tile_height as f32,
            source.w / tilemap.map.tile_width as f32,
            source.h / tilemap.map.tile_height as f32,
        );

        dest.scale(zoom, zoom);
        for i in 0..tilemap.map.layers.len() {
            tilemap.draw_tiles(i, dest, Some(source_in_tiles));
        }

        if is_key_down(KeyCode::Q) {
            break;
        }
        if is_key_pressed(KeyCode::KpAdd) || is_key_down(KeyCode::Key9) {
            zoom *= 2.0;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_down(KeyCode::Key8)) && zoom >= 2.0 {
            zoom *= 0.5;
        }
        if is_key_down(KeyCode::Key0) || is_key_down(KeyCode::Kp0) {
            zoom = 1.0;
            camera = (map_size.w / 2.0, map_size.h / 2.0);
        }
        if is_key_down(KeyCode::Left) {
            camera = (camera.0 - 2.0, camera.1);
        }
        if is_key_down(KeyCode::Right) {
            camera = (camera.0 + 2.0, camera.1);
        }
        if is_key_down(KeyCode::Up) {
            camera = (camera.0, camera.1 - 2.0);
        }
        if is_key_down(KeyCode::Down) {
            camera = (camera.0, camera.1 + 2.0);
        }

        next_frame().await
    }
}
