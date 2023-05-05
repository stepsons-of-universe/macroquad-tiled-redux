pub mod animation;
pub mod animation_controller;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::ops::{Add, Deref};
use std::path::Path;
use coarsetime::{Duration, Instant};

use macroquad::color::WHITE;
use macroquad::math::{Rect, vec2, Vec2};
use macroquad::file::FileError;
use macroquad::miniquad::fs::Error;
use macroquad::texture::{draw_texture_ex, DrawTextureParams, FilterMode, load_texture, Texture2D};

use tiled::{LayerType, Loader};
use tiled::Error as TiledError;

use crate::animation::{AnimatedTile, AnimatedSpriteState, Animation, AnimationFrame};


#[derive(Debug)]
pub struct TileSet {
    texture: Texture2D,
    pub tileset: tiled::Tileset,

    // todo: hide behind get_animation?
    /// Animations: map tile_id -> AnimatedSprite
    pub animations: HashMap<u32, AnimatedTile>,
}

impl TileSet {
    /// Not encapsulating `tiled::tileset::Tileset` in order to preserve the ability to
    /// load it from different sources.
    /// TODO: encapsulate into a number of constructors, with `Reader`, PathBuf, &str and what else.
    pub fn new(
        tileset: tiled::Tileset,
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
        tileset: tiled::Tileset,
    )
        -> Result<Self, FileError>
    {
        let image_source = &tileset
            .image
            .as_ref()
            .expect("Only spritesheet-type tilesets are now supported")
            .source;

        let texture: Texture2D = load_texture(image_source.to_str().unwrap())
            .await
            .unwrap_or_else(|e| panic!("Couldn't load the texture: {:?}: {}", image_source, e));

        // For a pixel-perfect rendering.
        // https://gamedev.stackexchange.com/questions/22712/how-can-i-draw-crisp-per-pixel-images-with-opengl-es-on-android
        texture.set_filter(FilterMode::Nearest);

        let mut animations = HashMap::new();

        for (tile_id, tile) in tileset.tiles() {
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
                    tile_id,
                    Animation {
                        frames,
                        duration: total_duration
                    }
                );
                animations.insert(tile_id, animation);
            }
        }

        Ok(Self::new(tileset, texture, animations))
    }

    // Duplicate of get_tile_rectangle_by_id from
    // https://github.com/mapeditor/rs-tiled/pull/87
    // Remove once that is merged.
    pub fn sprite_rect(&self, ix: u32) -> Rect {
        let sw = self.tileset.tile_width as f32;
        let sh = self.tileset.tile_height as f32;
        let sx = (ix % self.tileset.columns) as f32 * (sw + self.tileset.spacing as f32) + self.tileset.margin as f32;
        let sy = (ix / self.tileset.columns) as f32 * (sh + self.tileset.spacing as f32) + self.tileset.margin as f32;

        // TODO: configure tiles margin
        Rect::new(sx, sy, sw, sh)
        // Rect::new(sx + 1.1, sy + 1.1, sw - 2.2, sh - 2.2)
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

    pub fn spr_ex(&self, params: DrawTextureParams, dest: Vec2) {
        draw_texture_ex(
            self.texture,
            dest[0],
            dest[1],
            WHITE,
            params,
        );
    }
}

impl TileSet {
    /// Create a per-object animation state for the given animation.
    /// Later, use it to render it with `Self::ani_spr()`
    pub fn make_animated(&self, animation_id: u32, now: Instant, playing: bool) -> AnimatedSpriteState {
        AnimatedSpriteState::new(animation_id, now, playing)
    }

    pub fn ani_spr(&self, state: &mut AnimatedSpriteState, dest: Rect) {
        let ani_tile = self.animations
            .get(&state.current_animation())
            .unwrap_or_else(|| panic!("Animation {} not found", state.current_animation()));
        let tile = ani_tile.animation.frames[state.frame as usize].tile_id;
        self.spr(tile, dest);
    }
}

fn file_error_to_tiled(e: FileError) -> tiled::Error {

    let err = match e.kind {
        // TODO: handle properly for mobile platforms.
        Error::IOError(e) => e,
        Error::DownloadFailed => std::io::Error::from(ErrorKind::ConnectionReset),
        Error::IOSAssetNoSuchFile => std::io::Error::from(ErrorKind::NotFound),
        Error::IOSAssetNoData => std::io::Error::from(ErrorKind::NotFound),
        Error::AndroidAssetLoadingError => std::io::Error::from(ErrorKind::NotFound),
    };

    TiledError::ResourceLoadingError {
        path: e.path.into(),
        err: Box::new(err)
    }
}

#[derive(Debug)]
pub struct Map {
    // pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,

    pub map: tiled::Map,
}

impl Map {

    pub async fn new_async(map_path: &Path) -> Result<Self, TiledError> {
        let map = Loader::new()
            .load_tmx_map(map_path)?;

        let mut tilesets = HashMap::new();

        for tileset in map.tilesets().iter() {
            // FIXME: Probably better to save a reference than clone(), but
            // then Map/Tileset will be sprawling with lifetimes. Try it later.
            let mqts = TileSet::new_async(tileset.deref().clone())
                .await
                .map_err(file_error_to_tiled)?;
            tilesets.insert(tileset.name.clone(), mqts);
        }

        Ok( Self {
            tilesets,
            map
        })
    }

    fn get_tileset(&self, tileset: &str) -> &TileSet {
        self.tilesets.get(tileset)
            .unwrap_or_else(|| panic!("No such tileset: {}, tilesets available: {:?}",
                            tileset,
                            self.tilesets.keys()))
    }

    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        let tileset = self.get_tileset(tileset);
        tileset.spr(sprite, dest);
    }

    pub fn spr_ex(&self, tileset: &TileSet, params: DrawTextureParams, dest: Vec2) {
        tileset.spr_ex(params, dest);
    }

    // pub fn contains_layer(&self, layer: &str) -> bool {
    //     self.map.layers.contains_key(layer)
    // }

    /// Translate world pixel coordinates into screen pixels.
    /// `world_px`: position in world pixels
    /// `source`: source rectangle in world pixels
    /// `dest`: dest rectangle in screen pixels
    #[inline]
    pub fn world_px_to_screen(&self, world_px: Vec2, source_px: Rect, dest: Rect) -> Vec2 {
        (world_px - source_px.point()) / source_px.size() * dest.size() + dest.point()
    }

    // FIXME: Introduce different (new?)types for world pixel, world tile, screen pixel and maybe
    // screen tile types.
    /// Arguments:
    /// * `layer`: the Layer to draw.
    /// * `source`: the source Rect inside the entire Map, in world pixels. `None` for the entire layer.
    /// * `dest`: the Rect to draw into.
    ///
    /// Panics:
    /// * If `source` is `None` on infinite map;
    /// * If `layer` does not exist.
    pub fn draw_tiles(&self, layer: usize, dest: Rect, source_px: impl Into<Option<Rect>>) {
        assert!(self.map.layers().len() > layer, "No such layer: {}", layer);

        let source = source_px.into();
        assert!(!self.map.infinite() || source.is_some() , "On infinite maps, you must specify a `source` rect");

        let source = source.unwrap_or_else(|| Rect::new(
            0.,
            0.,
            (self.map.width * self.map.tile_width) as f32,
            (self.map.height * self.map.tile_height) as f32,
        ));

        let layer = match self.map.get_layer(layer) {
            Some(layer) => layer,
            None => return,
        };

        let layer = match layer.layer_type() {
            LayerType::Tiles(layer) => layer,
            _ => return,
            // TODO: Implement
            // LayerType::ObjectLayer(_) => {}
            // LayerType::ImageLayer(_) => {}
            // LayerType::GroupLayer(_) => {}
        };

        let world_tile_size = vec2(self.map.tile_width as f32, self.map.tile_height as f32);
        let spr_size= world_tile_size * dest.size() / source.size();

        let source_tiles = Rect::new(
            (source.x as i32 / self.map.tile_width as i32) as f32,
            (source.y as i32 / self.map.tile_height as i32) as f32,
            (source.w as i32 / self.map.tile_width as i32) as f32,
            (source.h as i32 / self.map.tile_height as i32) as f32,
        );

        // todo: support map.renderorder

        for y in (source_tiles.y as i32 - 1)..=(source_tiles.y as i32 + source_tiles.h as i32) + 1 {
            for x in (source_tiles.x as i32 - 1)..=(source_tiles.x as i32 + source_tiles.w as i32) + 1 {

                let pos = self.world_px_to_screen(vec2(x as f32, y as f32) * world_tile_size, source, dest);

                if let Some(tile) = layer.get_tile(x, y) {
                    let tileset = tile.get_tileset();
                    //if let Some(tileset) = self.map.tileset_by_gid(tile.id) {

                        // TODO (performance): Move out of loop, or cache tilesets.
                        let mq_tile_set = self.tilesets.get(&tileset.name)
                            .unwrap_or_else(|| panic!("Tileset {} not found", tileset.name));
                        let spr_rect = mq_tile_set.sprite_rect(tile.id()); //  - tileset.first_gid

                        let params = DrawTextureParams {
                            dest_size: Some(spr_size),
                            source: Some(spr_rect),
                            rotation: 0.0,
                            flip_x: tile.flip_v ^ tile.flip_d,
                            flip_y: tile.flip_h ^ tile.flip_d,
                            pivot: None
                        };

                        self.spr_ex(
                            mq_tile_set,
                            params,
                            pos,
                        );
                    //}
                }
            }
        }
    }
}
