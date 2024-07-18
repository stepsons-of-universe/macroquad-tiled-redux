use coarsetime::{Duration, Instant};
use std::collections::HashMap;
use std::ops::Add;

use macroquad::color::WHITE;
use macroquad::math::{vec2, Rect, Vec2};
use macroquad::texture::{draw_texture_ex, load_texture, DrawTextureParams, FilterMode, Texture2D};
use macroquad::Error as MqError;
use tiled::{PropertyValue, TileId};

use crate::animation::{AnimatedSpriteState, AnimatedTile, Animation, AnimationFrame};

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
        animations: HashMap<u32, AnimatedTile>,
    ) -> Self {
        Self {
            texture,
            tileset,
            animations,
        }
    }

    /// Future: loading Tileset can be wrapped into another async Future that
    /// loads it in another thread. Then the entire function could be Macroquad-async.
    pub async fn new_async(tileset: tiled::Tileset) -> Result<Self, MqError> {
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
                let frames: Vec<AnimationFrame> =
                    tiled_animation.iter().map(AnimationFrame::from).collect();

                // two passes, sure, but I expect them all not to exceed 10-20 frames.
                let total_duration = frames
                    .iter()
                    .fold(Duration::from_ticks(0), |sum, val| sum.add(val.duration));

                let animation = AnimatedTile::new(
                    tile_id,
                    Animation {
                        frames,
                        duration: total_duration,
                    },
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
        let sx = (ix % self.tileset.columns) as f32 * (sw + self.tileset.spacing as f32)
            + self.tileset.margin as f32;
        let sy = (ix / self.tileset.columns) as f32 * (sh + self.tileset.spacing as f32)
            + self.tileset.margin as f32;

        // TODO: configure tiles margin
        Rect::new(sx, sy, sw, sh)
        // Rect::new(sx + 1.1, sy + 1.1, sw - 2.2, sh - 2.2)
    }

    pub fn spr(&self, sprite: u32, dest: Rect) {
        let spr_rect = self.sprite_rect(sprite);

        draw_texture_ex(
            &self.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(Rect::new(spr_rect.x, spr_rect.y, spr_rect.w, spr_rect.h)),
                ..Default::default()
            },
        );
    }

    pub fn spr_ex(&self, params: DrawTextureParams, dest: Vec2) {
        draw_texture_ex(&self.texture, dest[0], dest[1], WHITE, params);
    }
}

impl TileSet {
    /// Create a per-object animation state for the given animation.
    /// Later, use it to render it with `Self::ani_spr()`
    pub fn make_animated(
        &self,
        animation_id: u32,
        now: Instant,
        playing: bool,
    ) -> AnimatedSpriteState {
        AnimatedSpriteState::new(animation_id, now, playing)
    }

    pub fn ani_sprite_index(&self, state: &mut AnimatedSpriteState) -> u32 {
        let ani_tile = self
            .animations
            .get(&state.current_animation())
            .unwrap_or_else(|| panic!("Animation {} not found", state.current_animation()));
        ani_tile.animation.frames[state.frame as usize].tile_id
    }

    pub fn ani_spr(&self, state: &mut AnimatedSpriteState, dest: Rect) {
        let tile = self.ani_sprite_index(state);
        self.spr(tile, dest);
    }
}

impl TileSet {
    pub fn tile_by_name(&self, name: &str) -> Option<TileId> {
        for (tile_id, tile) in self.tileset.tiles() {
            if let Some(PropertyValue::StringValue(name_property)) = tile.properties.get("name") {
                if name == name_property {
                    return Some(tile_id);
                }
            }
        }
        None
    }
}
