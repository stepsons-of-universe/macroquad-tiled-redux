use coarsetime::{Instant, Duration};
use tiled::animation::Frame;

#[derive(Clone, Debug)]
pub struct Animation {
    // Useful, but not included in TMX. Maybe utilize <properties> some day.
    // pub name: String,

    pub frames: Vec<AnimationFrame>,
    pub(crate) duration: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationFrame {
    pub tile_id: u32,
    pub duration: Duration,
}

impl From<&Frame> for AnimationFrame {
    fn from(f: &Frame) -> Self {
        Self {
            tile_id: f.tile_id,
            duration: Duration::from_millis(f.duration as u64),
        }
    }
}

/// Save in each instance of animated object.
#[derive(Clone, Copy)]
pub struct AnimatedSpriteState {
    pub animation_id: u32,
    /// Current frame
    pub frame: u32,
    /// Time the last current frame (should have) started at.
    pub frame_start: Instant,
    pub playing: bool,
}

/// In future, we might need this to belong to Tile. So far,
/// let's just keep a list of animations separately.
/// Shared by all objects that have this animation.
#[derive(Clone, Debug)]
pub struct AnimatedTile {
    pub id: u32,
    /// From TMX doc (https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#animation):
    /// > Each tile can have exactly one animation associated with it. In the future,
    /// > there could be support for multiple named animations on a tile.
    pub(crate) animation: Animation,
}

impl AnimatedSpriteState {
    pub fn new(current_animation: u32, start: Instant, playing: bool) -> Self {
        Self {
            animation_id: current_animation,
            frame_start: start,
            frame: 0,
            playing,
        }
    }

    pub fn current_animation(&self) -> u32 {
        self.animation_id
    }

    /// Sets the animation unless it was already set.
    pub fn set_animation(&mut self, animation: u32) {
        if self.animation_id != animation {
            self.reset_animation(animation);
        }
    }

    /// Starts the animation unconditionally, from the beginning.
    pub fn reset_animation(&mut self, animation_id: u32) {
        self.animation_id = animation_id;
        self.frame = 0;
        // todo: make it an Option? Because nobody should
        // call now() directly but the top level code.
        self.frame_start = Instant::now();
    }

    /// Call before drawing.
    pub fn update(&mut self, sprite: &AnimatedTile, now: Instant) {
        let animation = &sprite.animation;

        if self.playing {
            let mut dt = now - self.frame_start;
            if dt > animation.duration {
                let new_dt = dt.as_ticks() % animation.duration.as_ticks();
                dt = Duration::from_ticks(new_dt);
            }

            while dt > animation.frames[self.frame as usize].duration {
                dt -= animation.frames[self.frame as usize].duration;
                self.frame_start += animation.frames[self.frame as usize].duration;
                dt = now - self.frame_start;
                self.frame += 1;
                if self.frame >= animation.frames.len() as u32 {
                    self.frame = 0;
                }
            }
        }
    }

}

impl AnimatedTile {
    pub fn new(id: u32, animation: Animation) -> Self { Self { id, animation } }
}
