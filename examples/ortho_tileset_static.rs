use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, KeyCode};
use macroquad::math::Rect;
use macroquad::window::{clear_background, next_frame};

use tiled::tileset::Tileset;

use macroquad_tiled_redux::TileSet;


#[macroquad::main("Texture")]
async fn main() {
    let path = Path::new("assets/tiled_base64_zlib.tmx");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let tileset = Tileset::parse(reader, 1).unwrap();
    println!("{:?}", tileset);

    let mqts = TileSet::new_async(tileset, path)
        .await
        .expect("Couldn't load Tileset");

    let margin = 0.0;
    let mut zoom = 3.0;

    loop {
        clear_background(LIGHTGRAY);

        let tile_count = mqts.tileset.tilecount.unwrap_or(mqts.tileset.tiles.len() as u32);

        for i in 0..tile_count {
            let w = mqts.tileset.tile_width as f32;
            let h = mqts.tileset.tile_height as f32;
            let x = (i % mqts.tileset.columns) as f32 * (w + margin);
            let y = (i / mqts.tileset.columns) as f32 * (h + margin);
            let dest = Rect::new(x * zoom, y * zoom, w * zoom, h * zoom);
            mqts.spr(i, dest);
        };

        if is_key_down(KeyCode::Q) {
            break;
        }
        if is_key_down(KeyCode::KpAdd) || is_key_down(KeyCode::KpMultiply) {
            zoom += 1.0;
        }
        if (is_key_down(KeyCode::Minus) || is_key_down(KeyCode::KpSubtract)) && zoom >= 2.0 {
            zoom -= 1.0;
        }
        if is_key_down(KeyCode::Key0) || is_key_down(KeyCode::Kp0) {
            zoom = 1.0;
        }

        next_frame().await
    }
}
