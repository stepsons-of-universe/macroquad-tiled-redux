use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, KeyCode};
use macroquad::math::Rect;
use macroquad::texture::{load_texture, Texture2D};
use macroquad::window::{clear_background, next_frame};

use tiled::tileset::Tileset;

use macroquad_tiled_redux::TileSet;


#[macroquad::main("Texture")]
async fn main() {
    let path = Path::new("assets/tiled_base64_zlib.tmx");
    let file = File::open(&path).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let tileset = Tileset::parse(reader, 1).unwrap();
    println!("{:?}", tileset);

    let mqts = TileSet::new_async(tileset, path)
        .await
        .expect("Couldn't load Tileset");

    loop {
        clear_background(LIGHTGRAY);

        let tile_count = mqts.tileset.tilecount.unwrap_or(mqts.tileset.tiles.len() as u32);
        for i in 0..tile_count {
            let w = mqts.tileset.tile_width as f32;
            let h = mqts.tileset.tile_height as f32;
            let dest = Rect::new(
                (i / mqts.tileset.columns) as f32 * (w + 5.0),
                (i % mqts.tileset.columns) as f32 * (h + 5.0),
                    w,
                    h);
            mqts.spr(i, dest);
        };

        if is_key_down(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
