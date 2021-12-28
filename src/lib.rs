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
use tiled::error::TiledError;

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
    /// Not encapsulating `tiled::tileset::Tileset` in order to preserve the ability to
    /// load it from different sources.
    /// TODO: encapsulate into a number of constructors, with `Reader`, PathBuf, &str and what else.
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

#[derive(Debug)]
pub struct Map {
    // pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,

    pub map: tiled::map::Map,
}

impl Map {

    pub async fn new_async(map_path: &Path) -> Result<Self, TiledError> {
        let map = tiled::map::Map::parse_file(map_path)?;

        let mut tilesets = HashMap::new();

        for tileset in map.tilesets.iter() {
            // FIXME: Probably better to save a reference than clone(), but
            // then Map/Tileset will be sprawling with lifetimes. Try it later.
            let mqts = TileSet::new_async(tileset.clone(), map_path)
                .await
                .map_err(|e| TiledError::Other(format!("FileError: {:?}", e)) )?;
            tilesets.insert(tileset.name.clone(), mqts);
        }

        Ok( Self {
            tilesets,
            map
        })
    }

    fn get_tileset(&self, tileset: &str) -> &TileSet {
        self.tilesets.get(tileset)
            .expect(
                &format!("No such tileset: {}, tilesets available: {:?}",
                         tileset,
                         self.tilesets.keys()
                ))
    }

    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        let tileset = self.get_tileset(tileset);
        tileset.spr(sprite, dest);
    }

    pub fn spr_ex(&self, tileset: &str, source: Rect, dest: Rect) {
        let tileset = self.get_tileset(tileset);

        tileset.spr_ex(source, dest);
    }

    // pub fn contains_layer(&self, layer: &str) -> bool {
    //     self.map.layers.contains_key(layer)
    // }

    /// Arguments:
    /// * `layer`: the Layer to draw.
    // * `source`: the source Rect inside the entire Map, in pixels. `None` for the entire map.
    /// * `source`: the source Rect inside the entire Map, in TILES. `None` for the entire map.
    /// * `dest`: the Rect to draw into.
    ///
    /// Panics:
    /// * If `source` is `None` on infinite map;
    /// * If `layer` does not exist.
    pub fn draw_tiles(&self, layer: usize, dest: Rect, source: impl Into<Option<Rect>>) {
        assert!(self.map.layers.len() > layer, "No such layer: {}", layer);

        let source = source.into();
        assert!(!self.map.infinite || source.is_some() , "On infinite maps, you must specify a `source` rect");

        let source = source.unwrap_or(Rect::new(
            0.,
            0.,
            self.map.width as f32,
            self.map.height as f32,
        ));

        let layer = &self.map.layers[layer];

        let spr_width = dest.w / source.w;
        let spr_height = dest.h / source.h;

        // todo: support map.renderorder

        for y in source.y as i32..source.y as i32 + source.h as i32 {
            for x in source.x as i32..source.x as i32 + source.w as i32 {
                let pos = vec2(
                    (x - source.x as i32) as f32 / source.w * dest.w + dest.x,
                    (y - source.y as i32) as f32 / source.h * dest.h + dest.y,
                );

                if let Some(tile) = layer.get_tile(x, y) {
                    if let Some(tileset) = self.map.tileset_by_gid(tile.gid) {

                        // FIXME: Account for flipped/rotated flags (add to spr_ex signature)
                        self.spr(
                            &tileset.name,
                            tile.gid - tileset.first_gid,
                            Rect::new(pos.x, pos.y, spr_width, spr_height),
                        );
                    }
                }
            }
        }
    }
}
