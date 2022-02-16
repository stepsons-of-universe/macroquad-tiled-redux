use std::fs::File;
use std::io::BufReader;
use std::path::{Path};
use coarsetime::Instant;

use macroquad::color::LIGHTGRAY;
use macroquad::file::FileError;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use macroquad::math::{IVec2, ivec2, Rect, vec2, Vec2};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};

use tiled::{FilesystemResourceCache, Tileset};

use macroquad_tiled_redux::{Map, TileSet};
use macroquad_tiled_redux::animation_controller::{AnimationController, AnimationRegistry};

#[derive(Debug)]
#[derive(Copy, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}

struct GameState {
    // In world tiles.
    pub position: IVec2,
    pub char_animation: AnimationController,
    pub facing: Direction,
    // In world pixels.
    pub camera: Vec2,
    pub zoom: f32,
    tile_size: IVec2,
}

struct Resources {
    pub map: Map,
    // temporary, till animations kick in.
    pub char_tileset: TileSet,
    pub char_animations: AnimationRegistry,
}

impl Resources {
    fn direction_animation(&self, dir: Direction) -> Option<u32> {
        match dir {
            Direction::North => self.char_animations.get_animation_id("walk-n"),
            Direction::East => self.char_animations.get_animation_id("walk-e"),
            Direction::South => self.char_animations.get_animation_id("walk-s"),
            Direction::West => self.char_animations.get_animation_id("walk-w"),
        }
    }
}

#[inline]
fn ivec2_to_vec2(v: IVec2) -> Vec2 {
    vec2(v.x as f32, v.y as f32)
}

#[inline]
#[allow(dead_code)]
fn vec2_to_ivec2(v: Vec2) -> IVec2 {
    ivec2(v.x as i32, v.y as i32)
}


// I see three ways to animate things:
// - parts of a Map get animated as a part of Map redraw;
// - Entities with one looping animation just get a `TiAnimationState`.
// - Entities with changing animations (like characters) each get
// an `AnimationController`.

impl GameState {

    pub fn handle_input(&mut self, resources: &Resources) {
        if is_key_pressed(KeyCode::KpAdd) || is_key_pressed(KeyCode::Key9) {
            self.zoom *= 2.0;
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::Key8)) && self.zoom >= 2.0 {
            self.zoom *= 0.5;
        }
        if is_key_down(KeyCode::Key0) {
            self.zoom = 1.0;
        }

        let mut direction_name: Option<char> = None;
        let mut direction_offset = ivec2(0, 0);

        // TODO: Check if the terrain is walkable.
        if (is_key_pressed(KeyCode::Left) || (self.char_animation.get_frame(Instant::now()).is_none() && is_key_down(KeyCode::Left))) && self.position.x >= 1 {
            self.facing = Direction::West;
            direction_name = Some('w');
            direction_offset = ivec2(-1, 0);
        }
        if (is_key_pressed(KeyCode::Right) || (self.char_animation.get_frame(Instant::now()).is_none() && is_key_down(KeyCode::Right))) && self.position.x < resources.map.map.width as i32 {
            self.facing = Direction::East;
            direction_name = Some('e');
            direction_offset = ivec2(1, 0);
        }
        if (is_key_pressed(KeyCode::Up) || (self.char_animation.get_frame(Instant::now()).is_none() && is_key_down(KeyCode::Up))) && self.position.y >= 1 {
            self.facing = Direction::North;
            direction_name = Some('n');
            direction_offset = ivec2(0, -1);
        }
        if (is_key_pressed(KeyCode::Down) || (self.char_animation.get_frame(Instant::now()).is_none() && is_key_down(KeyCode::Down))) && self.position.x < resources.map.map.height as i32 {
            self.facing = Direction::South;
            direction_name = Some('s');
            direction_offset = ivec2(0, 1);
        }

        if let Some(direction) = direction_name {
            let walk_name = format!("walk-{}", direction);
            let idle_name = format!("cast-{}", direction);
            if let Some(animation) = resources.char_animations.get_template(&walk_name) {
                let origin = (
                    self.position.x as f32 * resources.map.map.tile_width as f32,
                    self.position.y as f32 * resources.map.map.tile_height as f32,
                );

                let movement = (
                    direction_offset.x as f32 * resources.map.map.tile_width as f32,
                    direction_offset.y as f32 * resources.map.map.tile_height as f32);

                self.char_animation.add_animation(Instant::now(), animation, movement, origin);
            }

            if let Some(animation) = resources.char_animations.get_template(&idle_name) {
                self.char_animation.set_idle_animation(animation, 3);
            }

            self.position += direction_offset;
        }
    }

    fn draw(&self, resources: &Resources) {
        clear_background(LIGHTGRAY);

        let tile_size = vec2(
            resources.map.map.tile_width as f32,
            resources.map.map.tile_height as f32);

        let screen = Rect::new(
            0.0, 0.0,
            screen_width(),
            screen_height());

        let screen_size_world_px = screen.size() / self.zoom;

        let source_topleft_world_px = self.camera + tile_size / 2.0 - screen_size_world_px / 2.0;
        let source = Rect::new(
            source_topleft_world_px.x,
            source_topleft_world_px.y,
            screen_size_world_px.x,
            screen_size_world_px.y,
        );

        let dest = screen;

        let char_frame = self.char_animation.get_frame(Instant::recent());

        for (i, _layer) in resources.map.map.layers().enumerate() {

            // Change the API from index to Layer?
            resources.map.draw_tiles(i, dest, Some(source));

            // Draw the character.
            if i == 0 {

                let char_screen_pos = resources.map.world_px_to_screen(
                    self.camera,
                    source,
                    dest);

                let char_dest = Rect::new(
                    char_screen_pos.x,
                    char_screen_pos.y,
                    // scale to map's tile size.
                    tile_size.x * self.zoom,
                    tile_size.y * self.zoom,
                );

                match &char_frame {
                    // animated
                    Some(frame) => {
                        // let char_dest = char_dest.offset(Vec2::from(movement) * self.zoom);
                        resources.char_tileset.spr(frame.tile_id, char_dest);
                    }

                    // static
                    None => {
                        let direction_sprite = resources.direction_animation(self.facing);

                        if let Some(gid) = direction_sprite {
                            resources.char_tileset.spr(gid, char_dest);
                        } else {
                            println!("error: no sprite for {:?}", self.facing);
                        }
                    }

                }
            }
        }
    }
}


async fn load_character() -> Result<TileSet, FileError> {
    let path = Path::new("assets/uLPC-drake.tsx");
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let tiled_tileset = Tileset::parse_with_path(reader, path).unwrap();
    TileSet::new_async(tiled_tileset)
        .await
}

#[macroquad::main("Texture")]
async fn main() {

    let tilemap = Map::new_async(
        Path::new("assets/grass/map1.tmx"),
        &mut FilesystemResourceCache::new())
        .await
        .expect("Error loading map");

    let char_tileset = load_character()
        .await
        .expect("Error loading char tileset");
    let char_animations = AnimationRegistry::load(&char_tileset.tileset);

    let resources = Resources {
        map: tilemap,
        char_tileset,
        char_animations,
    };

    let position = ivec2(10, 10);
    let tile_size = ivec2(resources.map.map.tile_width as i32, resources.map.map.tile_height as i32);

    let mut state = GameState {
        position,
        char_animation: AnimationController::new(),
        facing: Direction::South,
        camera: ivec2_to_vec2(position * tile_size),
        zoom: 2.0,
        tile_size,
    };

    loop {
        state.char_animation.update(Instant::now());
        let frame = state.char_animation.get_frame(Instant::recent());

        if let Some(frame) = frame {
            state.camera = Vec2::from(frame.position);
        } else {
            // no input if animations from the previous turn are playing.
            state.camera = ivec2_to_vec2(state.position * state.tile_size);
        }
        state.handle_input(&resources);

        state.draw(&resources);

        if is_key_down(KeyCode::Q) {
            break;
        }

        next_frame().await
    }
}
