use std::collections::HashMap;
use std::time::{Duration, Instant};
use tiled::properties::PropertyValue;
use tiled::tileset::Tileset;
// use coarsetime::Instant;

use macroquad_tiled_redux::animation::AnimatedSpriteState as TiAnimationState;
use macroquad_tiled_redux::animation::AnimatedTile as TiAnimation;

/// An animation "template", shared between
struct AnimationTemplate {
    pub name: String,
    pub frames: TiAnimation,

    /// First, player shoots and projectile flies, then enemy dies/blood flies,
    /// then blood decal appears. If enemy is attacking at the same time, then
    /// same thing: first attacks, then effects. Partial ordering relationship.
    /// TODO: Think more. How do different animation controllers know they need to sync?
    pub ordering: u8,

    /// Compression properties.
    /// % of time this animation can be compressed to. E.g. running can be sped up by 20 percent.
    /// (the number is arbitrary).
    /// Default: 0.
    pub max_compression: u32,
    /// If the next turn can be started before this finishes playing.
    /// E.g. NPC death animation can be played after the turn end, as that NPC has no
    /// more effect on the game state.
    /// Default: true
    pub blocks_turn: bool,
    /// Frame# after which this animation can be cancelled.
    /// Default: None
    pub cancel_frame: Option<u32>,
}

struct AnimationInstance {
    pub state: TiAnimationState,

    /// How much it moves the object, in tiles.
    /// The motion will be evenly distributed along the path.
    pub movement: (i32, i32),

    pub template: String,
}

/// Per-entity object that controls its animations.
struct AnimationController {
    pub entity_id: u32,
    /// Current animations to be played.
    animations: Vec<AnimationInstance>,
    /// If had no animations for `idle_interval`, play one of `idle_animations`
    idle_interval: Duration,
    /// Idle animations get interrupted immediately.
    idle_animations: Vec<TiAnimationState>,
}

impl AnimationController {

    pub fn update(&mut self, time: Instant) {

    }

    pub fn get_frame(&self, time: Instant) -> (frame, coordinate??) {
    ...
    }

    pub fn add_animation(&mut self, gid: u32, registry: &AnimationRegistry) {

    }
}

/// All the animations for a specific entity (character)
struct AnimationRegistry {
    tileset: Tileset,
    animations: HashMap<String, u32>,
}

impl AnimationRegistry {

    pub fn load(tileset: Tileset) -> Self {

        let mut animations: HashMap<String, u32> = HashMap::new();

        for tile in tileset.tiles.iter() {
            if let Some(value) = tile.properties.get("name") {
                match (value, &tile.animation) {
                    (PropertyValue::StringValue(name), Some(_)) => {
                        animations.insert(name.clone(), tile.id);
                    }

                    _ => {}
                }
            }
        }


        Self { tileset, animations }
    }

    pub fn get_template(&self, template: &str) -> Option<&AnimationTemplate> {

        match self.animations.get(template) {
            None => {}
            Some(id) => {}
        }

        None
    }
}
