use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use macroquad::color::LIGHTGRAY;
use macroquad::input::{is_key_down, KeyCode};
use macroquad::math::Rect;
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use tiled::tileset::Tileset;

use macroquad_tiled_redux::TileSet;


#[macroquad::main("Texture")]
async fn main() {
    let path = Path::new("assets/horse.tsx");
    let file = File::open(&path).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let tileset = Tileset::parse(reader, 1).unwrap();
    println!("{:?}", tileset);

    let mqts = TileSet::new_async(tileset, path)
        .await
        .expect("Couldn't load Tileset");

    let sprite_id = 0;

    let w = mqts.tileset.tile_width as f32;
    let h = mqts.tileset.tile_height as f32;

    let mut ani_state = mqts.make_animated(sprite_id, false);
    let animation = mqts.animations
        .get(&sprite_id);

    ani_state.playing = true;

    loop {
        clear_background(LIGHTGRAY);

        if let Some(animation) = animation {

            let dest = Rect::new(
                screen_width() / 2.0 - w / 2.0,
                screen_height() / 2.0 - h / 2.0,
                w,
                h);

            ani_state.update(&animation);
            mqts.ani_spr(&mut ani_state, dest);
        }

        if is_key_down(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
