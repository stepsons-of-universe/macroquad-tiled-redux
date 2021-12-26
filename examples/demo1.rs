use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use macroquad::color::{LIGHTGRAY, WHITE};
use macroquad::texture::{draw_texture, load_texture, Texture2D};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};
use tiled::parse;

#[macroquad::main("Texture")]
async fn main() {
    let file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let map = parse(reader).unwrap();
    println!("{:?}", map);
    println!("{:?}", map.get_tileset_by_gid(22));

    let texture: Texture2D = load_texture("assets/tilesheet.png").await.unwrap();

    loop {
        clear_background(LIGHTGRAY);
        draw_texture(
            texture,
            screen_width() / 2. - texture.width() / 2.,
            screen_height() / 2. - texture.height() / 2.,
            WHITE,
        );
        next_frame().await
    }
}
