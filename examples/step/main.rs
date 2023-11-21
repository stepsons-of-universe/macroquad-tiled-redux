use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{Rect, vec2};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};
use tiled::TileId;

use macroquad_tiled_redux::{Map};


trait WangWalls {
    fn is_wall_s(&self, tile_id: TileId) -> bool;
}

impl WangWalls for Map {

    fn is_wall_s(&self, tile_id: TileId) -> bool {
        // A lot of optimization potential here, if I convert it to hashmap first.
        for tileset in self.map.tilesets() {
            for wangset in tileset.wang_sets.iter() {
                if let Some(wt) = wangset.wang_tiles.get(&tile_id) {
                    // FIXME: Read the doc about which index is which. I know S is 4.
                    if wt.wang_id.0[4] != 0 {
                        return true
                    }
                }
            }
        }

        false
    }
}

// Now, what other info do we need from map?
// * Doors?
// * Containers?
// * Transparency for vision, bullets, lasers?
// * Object/wall hit points?
// * Obstacle shape? (probably the only)
// * Terrain, like water?
// * Lighting properties - like height, shadow shape?
// How do we implement:
// * Smoke
// * Fire
// * Vacuum
// * Radiation?

// Can we make entire layers non-walkable? Perhaps this will reduce the amount
// of work for map designer?
// Or more generally, set properties for entire layers?

// Awakening cave: додати нанесеної землі на підлогу.


// Later!
// trait DestructibleMap {
// }


#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(Path::new("assets/step/01-awakening-cave.tmx"))
        .await
        .expect("Error loading map");

    // println!("{:?}", tilemap);

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

        dest.scale(zoom, zoom);
        for (i, _layer) in tilemap.map.layers().enumerate() {
            tilemap.draw_tiles(i, dest, Some(source));
        }

        // let layer0 = tilemap.map.get_layer(0).unwrap().as_tile_layer().unwrap();
        // let tile = layer0.get_tile(10, 10).unwrap();
        // let i = tile.tileset_index();

        if is_key_down(KeyCode::Q) {
            break;
        }
        if is_key_pressed(KeyCode::KpAdd) || is_key_pressed(KeyCode::Key9) {
            zoom *= 2.0;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::Key8)) && zoom >= 2.0 {
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
