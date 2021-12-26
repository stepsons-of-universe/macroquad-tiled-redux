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

    // This should better sit inside of TileSet::new(), but alas, load_texture
    // has to be async.
    let image_source = &tileset
        .image
        .as_ref()
        .expect("Only spritesheet-type tilesets are now supported")
        .source;

    let image_path = path
        .parent()
        .unwrap()
        .join(image_source);
    let texture: Texture2D = load_texture(image_path.to_str().unwrap())
        .await
        .expect(&format!("Couldn't load the texture: {:?}", image_path));

    let mqts = TileSet::new(tileset, texture);

    loop {
        clear_background(LIGHTGRAY);

        let tile_count = mqts.tileset.tilecount.unwrap_or(mqts.tileset.tiles.len() as u32);
        for i in 0..tile_count {
            let w = mqts.tileset.tile_width as f32;
            let h = mqts.tileset.tile_height as f32;
            let dest = Rect::new(
                (i / mqts.tileset.columns) as f32 * w + 1.0,
                (i % mqts.tileset.columns) as f32 * h + 1.0,
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
