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

    let mut camera_center = (map_size.w / 2.0, map_size.h / 2.0);

    let mut zoom = 2.0;

    loop {
        clear_background(LIGHTGRAY);

        let screen = Rect::new(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
        );

        let mut source = match map_size.intersect(screen) {
            None => continue,
            Some(source) => source,
        };
        let mut dest = screen.intersect(source).expect("should always intersect");

        source.move_to(vec2(camera.0 - screen_width()/2, camera.1 - screen_height()/2));

        let mut source_in_tiles = source;
        source_in_tiles.scale(1.0 / tilemap.map.tile_width as f32, 1.0 / tilemap.map.tile_height as f32);

        dest.scale(zoom, zoom);
        tilemap.draw_tiles(0, dest, Some(source_in_tiles));

        if is_key_down(KeyCode::Q) {
            break;
        }
        if is_key_pressed(KeyCode::KpAdd) || is_key_pressed(KeyCode::KpMultiply) {
            zoom *= 1.25;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract)) && zoom >= 2.0 {
            zoom *= 0.8;
        }
        if is_key_pressed(KeyCode::Key0) || is_key_pressed(KeyCode::Kp0) {
            zoom = 1.0;
            camera_center = (map_size.w / 2.0, map_size.h / 2.0);
        }
        if is_key_pressed(KeyCode::Left) {
            camera_center = (camera_center.0 - 2.0, camera_center.1);
        }
        if is_key_pressed(KeyCode::Right) {
            camera_center = (camera_center.0 + 2.0, camera_center.1);
        }
        if is_key_pressed(KeyCode::Up) {
            camera_center = (camera_center.0, camera_center.1 - 2.0);
        }
        if is_key_pressed(KeyCode::Down) {
            camera_center = (camera_center.0, camera_center.1 + 2.0);
        }

        next_frame().await
    }
}
