mod animation;

use std::collections::HashMap;
use std::ops::Add;
use std::path::Path;
use coarsetime::Duration;

use macroquad::color::WHITE;
use macroquad::math::{Rect, vec2};
use macroquad::file::FileError;
use macroquad::texture::{draw_texture_ex, DrawTextureParams, load_texture, Texture2D};

use tiled;

use crate::animation::{AnimatedTile, AnimatedSpriteState, Animation, AnimationFrame};


#[derive(Debug)]
pub struct TileSet {
    texture: Texture2D,
    pub tileset: tiled::tileset::Tileset,

    // todo: hide behind get_animation?
    /// Animations: map tile_id -> AnimatedSprite
    pub animations: HashMap<u32, AnimatedTile>,
}

impl TileSet {
    pub fn new(
        tileset: tiled::tileset::Tileset,
        texture: Texture2D,
        animations: HashMap<u32, AnimatedTile>
    ) -> Self
    {
        Self {
            texture,
            tileset,
            animations,
        }
    }

    /// Future: loading Tileset can be wrapped into another async Future that
    /// loads it in another thread. Then the entire function could be Macroquad-async.
    pub async fn new_async(
        tileset: tiled::tileset::Tileset,
        tileset_path: &Path,
    )
        -> Result<Self, FileError>
    {
        let image_source = &tileset
            .image
            .as_ref()
            .expect("Only spritesheet-type tilesets are now supported")
            .source;

        let image_path = tileset_path
            .parent()
            .expect("Tileset path has no parent")
            .join(image_source);

        let texture: Texture2D = load_texture(image_path.to_str().unwrap())
            .await
            .expect(&format!("Couldn't load the texture: {:?}", image_path));

        let mut animations = HashMap::new();

        for tile in tileset.tiles.iter() {
            if let Some(tiled_animation) = &tile.animation {

                let frames: Vec<AnimationFrame> = tiled_animation
                    .iter()
                    .map(AnimationFrame::from)
                    .collect();

                // two passes, sure, but I expect them all not to exceed 10-20 frames.
                let total_duration = frames.iter()
                    .fold(
                        Duration::from_ticks(0),
                        |sum, val| sum.add(val.duration) );

                let animation = AnimatedTile::new(
                    Animation {
                        frames,
                        duration: total_duration
                    }
                );
                animations.insert(tile.id, animation);
            }
        }

        Ok(Self::new(tileset, texture, animations))
    }

    fn sprite_rect(&self, ix: u32) -> Rect {
        let sw = self.tileset.tile_width as f32;
        let sh = self.tileset.tile_height as f32;
        let sx = (ix % self.tileset.columns) as f32 * (sw + self.tileset.spacing as f32) + self.tileset.margin as f32;
        let sy = (ix / self.tileset.columns) as f32 * (sh + self.tileset.spacing as f32) + self.tileset.margin as f32;

        // TODO: configure tiles margin
        Rect::new(sx, sy, sw, sh)
    }

    pub fn spr(&self, sprite: u32, dest: Rect) {
        let spr_rect = self.sprite_rect(sprite);

        draw_texture_ex(
            self.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(Rect::new(
                    spr_rect.x,
                    spr_rect.y,
                    spr_rect.w,
                    spr_rect.h,
                )),
                ..Default::default()
            },
        );
    }

    pub fn spr_ex(&self, source: Rect, dest: Rect) {
        draw_texture_ex(
            self.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(source),
                ..Default::default()
            },
        );
    }
}

impl TileSet {
    /// Create a per-object animation state for the given animation.
    /// Later, use it to render it with `Self::ani_spr()`
    pub fn make_animated(&self, animation_id: u32, playing: bool) -> AnimatedSpriteState {
        AnimatedSpriteState::new(animation_id, playing)
    }

    pub fn ani_spr(&self, state: &mut AnimatedSpriteState, dest: Rect) {
        let ani_tile = self.animations
            .get(
                &state.current_animation())
            .expect(&format!("Animation {} not found", state.current_animation()));
        let tile = ani_tile.animation.frames[state.frame as usize].tile_id;
        self.spr(tile, dest);
    }
}

// #[derive(Debug)]
// pub struct Map {
//     // pub layers: HashMap<String, Layer>,
//     // pub tilesets: HashMap<String, TileSet>,
//
//     pub map: tiled::Map,
// }
//
// impl Map {
//     pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
//         if self.tilesets.contains_key(tileset) == false {
//             panic!(
//                 "No such tileset: {}, tilesets available: {:?}",
//                 tileset,
//                 self.tilesets.keys()
//             )
//         }
//         let tileset = &self.tilesets[tileset];
//
//         tileset.spr(sprite, dest);
//     }
//
//     pub fn spr_ex(&self, tileset: &str, source: Rect, dest: Rect) {
//         let tileset = &self.tilesets[tileset];
//
//         tileset.spr_ex(source, dest);
//     }
//
//     pub fn contains_layer(&self, layer: &str) -> bool {
//         self.layers.contains_key(layer)
//     }
//
//     pub fn draw_tiles(&self, layer: &str, dest: Rect, source: impl Into<Option<Rect>>) {
//         assert!(self.layers.contains_key(layer), "No such layer: {}", layer);
//
//         let source = source.into().unwrap_or(Rect::new(
//             0.,
//             0.,
//             self.raw_tiled_map.width as f32,
//             self.raw_tiled_map.height as f32,
//         ));
//         let layer = &self.layers[layer];
//
//         let spr_width = dest.w / source.w;
//         let spr_height = dest.h / source.h;
//
//         for y in source.y as u32..source.y as u32 + source.h as u32 {
//             for x in source.x as u32..source.x as u32 + source.w as u32 {
//                 let pos = vec2(
//                     (x - source.x as u32) as f32 / source.w * dest.w + dest.x,
//                     (y - source.y as u32) as f32 / source.h * dest.h + dest.y,
//                 );
//
//                 if let Some(tile) = &layer.data[(y * layer.width + x)] {
//                     self.spr(
//                         &tile.tileset,
//                         tile.id,
//                         Rect::new(pos.x, pos.y, spr_width, spr_height),
//                     );
//                 }
//             }
//         }
//     }
// }
