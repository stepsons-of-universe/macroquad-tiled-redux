use macroquad::texture::{draw_texture_ex, DrawTextureParams, Texture2D};
use macroquad::math::{Rect, vec2};
use tiled;
use macroquad;
use macroquad::color::WHITE;

#[derive(Debug)]
pub struct TileSet {
    texture: Texture2D,
    pub tileset: tiled::tileset::Tileset,
}

impl TileSet {
    pub fn new(tileset: tiled::tileset::Tileset, texture: Texture2D) -> Self {
        Self {
            texture,
            tileset,
        }
    }

    // pub async fn new_async(tileset: tiled::tileset::Tileset) -> Self {
    //     Self {
    //         texture,
    //         tileset,
    //     }
    // }

    fn sprite_rect(&self, ix: u32) -> Rect {
        let sw = self.tileset.tile_width as f32;
        let sh = self.tileset.tile_height as f32;
        let sx = (ix % self.tileset.columns) as f32 * (sw + self.tileset.spacing as f32) + self.tileset.margin as f32;
        let sy = (ix / self.tileset.columns) as f32 * (sh + self.tileset.spacing as f32) + self.tileset.margin as f32;

        // TODO: configure tiles margin
        Rect::new(sx + 1.1, sy + 1.1, sw - 2.2, sh - 2.2)
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
                    spr_rect.x - 1.0,
                    spr_rect.y - 1.0,
                    spr_rect.w + 2.0,
                    spr_rect.h + 2.0,
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
//                 if let Some(tile) = &layer.data[(y * layer.width + x) as usize] {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
