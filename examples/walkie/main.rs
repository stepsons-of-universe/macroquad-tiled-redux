mod animation_controller;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path};

use macroquad::color::LIGHTGRAY;
use macroquad::file::FileError;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{Rect, vec2, Vec2};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use tiled::tileset::Tileset;

use macroquad_tiled_redux::{Map, TileSet};

enum Direction {
    North,
    East,
    South,
    West,
}

struct GameState {
    pub position: Vec2,
    pub facing: Direction,
    pub zoom: f32,
}


// I see three ways to animate things:
// - parts of a Map get animated as a part of Map redraw;
// - Entities with one looping animation just get a `TiAnimationState`.
// - Entities with changing animations (like characters) each get
// an `AnimationController`.

impl GameState {

    pub fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::KpAdd) || is_key_down(KeyCode::Key9) {
            self.zoom *= 2.0;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_down(KeyCode::Key8)) && self.zoom >= 2.0 {
            self.zoom *= 0.5;
        }
        if is_key_down(KeyCode::Key0) || is_key_down(KeyCode::Kp0) {
            self.zoom = 1.0;
            // camera = (map_size.w / 2.0, map_size.h / 2.0);
        }
        if is_key_down(KeyCode::Left) {
            // camera = (camera.0 - 2.0, camera.1);
        }
        if is_key_down(KeyCode::Right) {
            // camera = (camera.0 + 2.0, camera.1);
        }
        if is_key_down(KeyCode::Up) {
            // camera = (camera.0, camera.1 - 2.0);
        }
        if is_key_down(KeyCode::Down) {
            // camera = (camera.0, camera.1 + 2.0);
        }
    }

    fn draw_map(&self, tilemap: &Map) {
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
            self.position.x * tilemap.map.tile_width as f32  - screen_width() / self.zoom / 2.0,
            self.position.y as f32 * tilemap.map.tile_height as f32 - screen_height() / self.zoom / 2.0));

        let source_in_tiles = Rect::new(
            source.x / tilemap.map.tile_width as f32,
            source.y / tilemap.map.tile_height as f32,
            source.w / tilemap.map.tile_width as f32,
            source.h / tilemap.map.tile_height as f32,
        );

        dest.scale(self.zoom, self.zoom);
        for i in 0..tilemap.map.layers.len() {
            tilemap.draw_tiles(i, dest, Some(source_in_tiles));
        }
    }

}


async fn load_character() -> Result<TileSet, FileError> {
    let path = Path::new("assets/horse.tsx");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let tiled_tileset = Tileset::parse_with_path(reader, 1, path).unwrap();
    TileSet::new_async(tiled_tileset)
        .await
}

#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(Path::new("assets/grass/map1.tmx"))
        .await
        .expect("Error loading map");

    let map_size = Rect::new(
        0.0,
        0.0,
        (tilemap.map.width * tilemap.map.tile_width) as f32,
        (tilemap.map.height * tilemap.map.tile_height) as f32);

    let char_tileset = load_character()
        .await
        .expect("Error loading char tileset");
    let char_sprite_id = 1;
    let mut char_ani_state = char_tileset.make_animated(char_sprite_id, false);

    let mut state = GameState {
        position: vec2(10.0, 10.0),
        facing: Direction::South,
        zoom: 2.0,
    };

    loop {
        state.draw_map(&tilemap);

        state.handle_input();
        if is_key_down(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
