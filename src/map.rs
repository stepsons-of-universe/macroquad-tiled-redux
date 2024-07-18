use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Deref;
use std::path::Path;

use macroquad::math::{ivec2, vec2, IVec2, Rect, Vec2};
use macroquad::texture::DrawTextureParams;
use macroquad::Error as MqError;

use tiled::Error as TiledError;
use tiled::{LayerType, Loader};

use crate::layer_order::LayersOrder;
use crate::tileset::TileSet;

#[derive(Debug)]
pub struct Map {
    // pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,
    pub layer_order: LayersOrder,
    pub map: tiled::Map,
}

impl Map {
    pub async fn new_async(map_path: &Path) -> Result<Self, TiledError> {
        let map = Loader::new().load_tmx_map(map_path)?;
        Self::new_async_map(map).await
    }

    pub async fn new_async_map(map: tiled::Map) -> Result<Self, TiledError> {
        let mut tilesets = HashMap::new();

        for tileset in map.tilesets().iter() {
            // FIXME: Probably better to save a reference than clone(), but
            // then Map/Tileset will be sprawling with lifetimes. Try it later.
            let mqts = TileSet::new_async(tileset.deref().clone())
                .await
                .map_err(file_error_to_tiled)?;
            tilesets.insert(tileset.name.clone(), mqts);
        }

        let layer_order = LayersOrder::new(map.layers());

        Ok(Self {
            tilesets,
            layer_order,
            map,
        })
    }

    fn get_tileset(&self, tileset: &str) -> &TileSet {
        self.tilesets.get(tileset).unwrap_or_else(|| {
            panic!(
                "No such tileset: {}, tilesets available: {:?}",
                tileset,
                self.tilesets.keys()
            )
        })
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

    // FIXME: Introduce different (new?)types for world pixel, world tile, screen pixel and maybe
    // screen tile types.
    /// Arguments:
    /// * `layer`: the Layer to draw.
    /// * `source`: the source Rect inside the entire Map, in world pixels. `None` for the entire layer.
    /// * `dest`: the Rect to draw into.
    /// * `callback(pos: Vec2) -> bool`: draw if callback return `true`.
    ///
    /// Panics:
    /// * If `source` is `None` on infinite map;
    /// * If `layer` does not exist.
    pub fn draw_tiles_callback<F>(
        &self,
        layer: usize,
        dest: Rect,
        source_px: impl Into<Option<Rect>>,
        callback: Option<F>,
    ) where
        F: Fn(IVec2) -> bool,
    {
        assert!(self.map.layers().len() > layer, "No such layer: {}", layer);

        let source = source_px.into();
        assert!(
            !self.map.infinite() || source.is_some(),
            "On infinite maps, you must specify a `source` rect"
        );

        let source = source.unwrap_or_else(|| {
            Rect::new(
                0.,
                0.,
                (self.map.width * self.map.tile_width) as f32,
                (self.map.height * self.map.tile_height) as f32,
            )
        });

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
        let spr_size = world_tile_size * dest.size() / source.size();

        let source_tiles = Rect::new(
            (source.x as i32 / self.map.tile_width as i32) as f32,
            (source.y as i32 / self.map.tile_height as i32) as f32,
            (source.w as i32 / self.map.tile_width as i32) as f32,
            (source.h as i32 / self.map.tile_height as i32) as f32,
        );

        // todo: support map.renderorder

        for y in (source_tiles.y as i32 - 1)..=(source_tiles.y as i32 + source_tiles.h as i32) + 1 {
            for x in
                (source_tiles.x as i32 - 1)..=(source_tiles.x as i32 + source_tiles.w as i32) + 1
            {
                // maybe use layer. instead of map
                if x < 0 || x as u32 >= self.map.width || y < 0 || y as u32 >= self.map.height {
                    continue;
                }

                if let Some(cb) = callback.as_ref() {
                    if !cb(ivec2(x, y)) {
                        continue;
                    }
                }

                let pos =
                    world_px_to_screen(vec2(x as f32, y as f32) * world_tile_size, source, dest);

                if let Some(tile) = layer.get_tile(x, y) {
                    let tileset = tile.get_tileset();

                    // TODO (performance): Move out of loop, or cache tilesets.
                    let mq_tile_set = self
                        .tilesets
                        .get(&tileset.name)
                        .unwrap_or_else(|| panic!("Tileset {} not found", tileset.name));
                    let spr_rect = mq_tile_set.sprite_rect(tile.id()); //  - tileset.first_gid

                    // 90: 101, 180: 110, 270: 011 - HVD
                    let (h, v, r) = match (tile.flip_h, tile.flip_v, tile.flip_d) {
                        (h, v, false) => (h, v, 0.0),
                        (true, false, true) => (false, false, PI / 2.0),
                        // (true, true, false) => (false, false, PI), - covered by above
                        (false, true, true) => (false, false, PI * 3.0 / 2.0),
                        // tiled didn't produce other combinations for me, so

                        // not sure about these two.
                        (true, true, true) => (false, false, PI / 2.0),
                        (false, false, true) => (true, true, 0.0),
                    };

                    let params = DrawTextureParams {
                        dest_size: Some(spr_size),
                        source: Some(spr_rect),
                        rotation: r,
                        flip_x: h,
                        flip_y: v,
                        pivot: None,
                    };

                    self.spr_ex(mq_tile_set, params, pos);
                }
            }
        }
    }

    pub fn draw_tiles(&self, layer: usize, dest: Rect, source_px: impl Into<Option<Rect>>) {
        let no_callback: Option<fn(IVec2) -> bool> = None;
        self.draw_tiles_callback(layer, dest, source_px, no_callback)
    }
}

/// Translate world pixel coordinates into screen pixels.
/// `world_px`: position in world pixels
/// `source`: source rectangle in world pixels
/// `dest`: dest rectangle in screen pixels
#[inline]
pub fn world_px_to_screen(world_px: Vec2, source_px: Rect, dest: Rect) -> Vec2 {
    (world_px - source_px.point()) / source_px.size() * dest.size() + dest.point()
}

fn file_error_to_tiled(e: MqError) -> tiled::Error {
    match e {
        MqError::FontError(message) => TiledError::MalformedAttributes(message.to_string()),
        MqError::FileError { kind, path } => TiledError::ResourceLoadingError {
            path: path.clone().into(),
            err: Box::new(MqError::FileError { kind, path }),
        },
        MqError::ShaderError(e) => TiledError::MalformedAttributes(e.to_string()),
        MqError::ImageError(e) => TiledError::MalformedAttributes(e.to_string()),
        MqError::UnknownError(e) => TiledError::MalformedAttributes(e.to_string()),
    }
}
