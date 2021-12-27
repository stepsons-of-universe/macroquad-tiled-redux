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
    pub tile_id: usize,
    pub duration: Duration,
}

impl From<&Frame> for AnimationFrame {
    fn from(f: &Frame) -> Self {
        Self {
            tile_id: f.tile_id as usize,
            duration: Duration::from_millis(f.duration as u64),
        }
    }
}

/// Save in each instance of animated object.
#[derive(Clone, Copy)]
pub struct AnimatedSpriteState {
    animation_id: usize,
    /// Current frame
    pub(crate) frame: u32,
    /// Time the last current (should have) started at.
    time: Instant,
    pub playing: bool,
}

/// In future, we might need this to belong to Tile. So far,
/// let's just keep a list of animations separately.
/// Shared by all objects that have this animation.
#[derive(Clone, Debug)]
pub struct AnimatedTile {
    /// From TMX doc (https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#animation):
    /// > Each tile can have exactly one animation associated with it. In the future,
    /// > there could be support for multiple named animations on a tile.
    pub(crate) animation: Animation,
}

impl AnimatedSpriteState {
    pub(crate) fn new(current_animation: usize, playing: bool) -> Self {
        Self {
            animation_id: current_animation,
            time: Instant::now(),
            frame: 0,
            playing,
        }
    }

    pub fn current_animation(&self) -> usize {
        self.animation_id
    }

    /// Sets the animation unless it was already set.
    pub fn set_animation(&mut self, animation: usize) {
        if self.animation_id != animation {
            self.reset_animation(animation);
        }
    }

    /// Starts the animation unconditionally, from the beginning.
    pub fn reset_animation(&mut self, animation_id: usize) {
        self.animation_id = animation_id;
        self.frame = 0;
        self.time = Instant::now();
    }

    /// Call before drawing.
    pub fn update(&mut self, sprite: &AnimatedTile) {
        let animation = &sprite.animation;

        if self.playing {
            let now = Instant::now();
            // let now = Instant::recent();
            let mut dt = now - self.time;
            if dt > animation.duration {
                let new_dt = dt.as_ticks() % animation.duration.as_ticks();
                dt = Duration::from_ticks(new_dt);
            }

            while dt > animation.frames[self.frame as usize].duration {
                dt -= animation.frames[self.frame as usize].duration;
                self.time += animation.frames[self.frame as usize].duration;
                dt = now - self.time;
                self.frame += 1;
                if self.frame >= animation.frames.len() as u32 {
                    self.frame = 0;
                }
            }
        }
    }

}

impl AnimatedTile {
    pub fn new(animation: Animation) -> Self { Self { animation } }

    // pub fn frame(&self) -> AnimationFrame {
    //     let animation = &self.animations[self.current_animation];
    //
    //     AnimationFrame {
    //         source_rect: Rect::new(
    //             self.tile_width * self.frame as f32,
    //             self.tile_height * animation.row as f32,
    //             self.tile_width,
    //             self.tile_height,
    //         ),
    //         dest_size: vec2(self.tile_width, self.tile_height),
    //     }
    // }
}
