use std::collections::HashMap;
use coarsetime::{Duration, Instant};
use tiled::animation;
use tiled::properties::PropertyValue;
use tiled::tileset::Tileset;

pub struct OutputFrame {
    pub tile_id: u32,
    pub position: (f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationFrame {
    pub tile_id: u32,
    pub duration: Duration,
}

impl From<&animation::Frame> for AnimationFrame {
    fn from(f: &animation::Frame) -> Self {
        Self {
            tile_id: f.tile_id,
            duration: Duration::from_millis(f.duration as u64),
        }
    }
}

/// An animation "template", shared between
pub struct AnimationTemplate {
    /// Animation name, stored in Properties -> "name": String
    pub name: String,
    /// Tile that the animation is attached to
    pub gid: u32,

    pub frames: Vec<AnimationFrame>,

    /// First, player shoots and projectile flies, then enemy dies/blood flies,
    /// then blood decal appears. If enemy is attacking at the same time, then
    /// same thing: first attacks, then effects. Partial ordering relationship.
    ///
    /// TODO: Think more. How do different animation controllers know they need to sync?
    /// Probably, they don't.
    ///
    /// Probably, instead we need add_compressed(target_duration). Then, we call:
    /// * add_compressed(attack_animation, dur1) on each combatant,
    /// * add_compressed(projectile_animation, dur1) on each projectile,
    /// * add_compressed(damaged_animation, dur2) on each combatant,
    /// * add_compressed(damaged_animation, dur2) on each combatant,
    /// * somehow add blood decal, delayed. Either we also need Animation
    /// to spawn decals, or other delayed way to spawn things. I certainly don't want
    /// to wait for animations to end to do something else.
    pub ordering: u8,

    /// Speed compression properties. Depending on the size of the animations queue,
    /// the controller can increase the animation speed and/or cancel a "tail" of it.
    /// % of time this animation can be compressed to. E.g. running can be compressed by 20%, i.e.
    /// running is 5 times faster than walking
    /// (the number is arbitrary).
    /// Default: 100.
    pub max_compression: u32,
    /// If the next turn can be started before this finishes playing.
    /// E.g. NPC death animation can be played after the turn end, as that NPC has no
    /// more effect on the game state.
    /// Default: true
    pub blocks_turn: bool,
    /// Frame# after which this animation can be cancelled.
    /// Default: None
    pub cancel_frame: Option<u32>,

    // Nice to have: depending on compression level, change move animation
    // from step to walk to running.
}

#[derive(Clone)]
struct AnimationInstance {
    /// Time the animation (should have) started at.
    pub animation_start: Instant,

    /// A copy of frames from AnimationTemplate.
    /// Excessive but works.
    pub frames: Vec<AnimationFrame>,

    pub duration: Duration,

    /// How much it moves the object, in tiles. E.g. walking or knockback animations do it.
    /// The motion will be evenly distributed along the path.
    pub movement: (f32, f32),

    pub start_position: (f32, f32),
    pub max_compression: u32,
    pub is_compressed: bool,
}

impl AnimationInstance {
    /// Creates animation of a sprite that moves by `movement` relative to its starting position.
    pub fn new(start_time: Instant, template: &AnimationTemplate, movement: (f32, f32), start_position: (f32,f32)) -> Self {
        let total_ticks = template.frames.iter().map(|it| it.duration.as_ticks()).sum();
        Self {
            animation_start: start_time,
            duration: Duration::from_ticks(total_ticks),
            frames: template.frames.clone(),
            movement,
            start_position,
            max_compression: template.max_compression,
            is_compressed: false,
        }
    }

    /// The compression starts immediately when key is pressed
    pub fn compress(&mut self, current_time: Instant) {
        if self.max_compression >= 100 {
            self.is_compressed = true;
            return;
        }

        let mut new_frames: Vec<AnimationFrame> = vec![];
        let mut start = self.animation_start;

        for frame in &self.frames {
            let new_duration;
            if start + frame.duration <= current_time {
                start += frame.duration;
                continue;
            } else if start < current_time && start + frame.duration > current_time {
                new_duration = (frame.duration - (current_time - start)) * self.max_compression / 100;
            } else {
                new_duration = frame.duration * self.max_compression / 100;
            }
            let f = AnimationFrame {
                tile_id: frame.tile_id,
                duration: new_duration,
            };
            new_frames.push(f);
            start += frame.duration;
        }

        let new_duration = new_frames.iter().map(|it| it.duration.as_ticks()).sum();
        let k = (self.duration.as_ticks() * self.max_compression as u64) as f32 / (new_duration * 100) as f32;
        let new_movement = (self.movement.0 /  k, self.movement.1 / k);
        let new_start_position = (self.start_position.0 + (self.movement.0 - new_movement.0), self.start_position.1 + (self.movement.1 - new_movement.1));

        self.animation_start = current_time;
        self.frames = new_frames;
        self.duration = Duration::from_ticks(new_duration);
        self.movement = new_movement;
        self.start_position = new_start_position;
        self.is_compressed = true;
    }
}

/// Per-entity object that controls its animations.
#[derive(Clone, Default)]
pub struct AnimationController {
    /// Current animations to be played.
    animations: Vec<AnimationInstance>,
    /// If had no animations for `idle_interval`, play one of `idle_animations`
    idle_interval: Option<Duration>,
    /// Idle animations get interrupted immediately.
    idle_animations: Vec<IdleInstance>,
    idle_start: Option<IdleStart>,
}

impl AnimationController {

    pub fn new() -> Self { Self::default() }

    /// Discards the animations whose time is gone.
    pub fn update(&mut self, time: Instant) {
        if ! self.animations.is_empty() {
            self.animations.retain(|i|i.animation_start + i.duration >= time);
        }
    }

    /// Returns OutputFrame for the given time moment, if there is
    /// a frame to show, otherwise None.
    /// Only goes down to current or next frame.
    pub fn get_frame(&self, time: Instant) -> Option<OutputFrame> {
        match self.animations.get(0) {
            Some(instance) => {
                let tile_id = Self::get_tile_id(time, instance);
                let position = Self::get_position(time, instance);
                Some( OutputFrame {
                    tile_id,
                    position,
                })
            }
            None => self.get_idle_animation(time),
        }
    }

    pub fn add_animation(&mut self, start_time: Instant, template: &AnimationTemplate, movement: (f32, f32), start_position: (f32, f32)) {
        if template.max_compression == 0 {
            return;
        }
        let mut new_start_time = start_time;
        let mut new_start_position = start_position;
        let mut new_instance = AnimationInstance::new(new_start_time, template, movement, new_start_position);
        if !self.animations.is_empty() {
            self.compress(start_time);
            let last_instance = self.animations.last().unwrap();
            new_start_time = last_instance.animation_start + last_instance.duration;
            new_start_position = (last_instance.start_position.0 + last_instance.movement.0, last_instance.start_position.1 + last_instance.movement.1);
            new_instance = AnimationInstance::new(new_start_time, template, movement, new_start_position);
            new_instance.compress(new_start_time);
        }
        let end_time = new_instance.animation_start + new_instance.duration;
        let end_position = (new_instance.start_position.0 + new_instance.movement.0, new_instance.start_position.1 + new_instance.movement.1);
        self.idle_start = Some(IdleStart::new(end_time, end_position));
        self.animations.push(new_instance);
    }

    fn compress(&mut self, time: Instant) {
        for animation in &mut self.animations {
            if !animation.is_compressed {
                animation.compress(time);
            }
        }
    }

    fn get_tile_id(finish_time: Instant, instance: &AnimationInstance) -> u32 {
        let start_time = instance.animation_start;
        let mut time = finish_time - start_time;
        for frame in &instance.frames {
            if time < frame.duration {
                return frame.tile_id;
            }
            time -= frame.duration;
        }
        // Is it normal to return 0?..
        // Yes, it is. It is a flag that something is wrong
        0
    }

    fn get_position(finish_time: Instant, instance: &AnimationInstance) -> (f32,f32) {
        let movement = instance.movement;
        let start_position = instance.start_position;
        let start_time = instance.animation_start;
        let duration = (finish_time - start_time).as_ticks() as f32;
        let total_duration = instance.duration.as_ticks() as f32;
        let x = start_position.0 + movement.0  * duration / total_duration;
        let y = start_position.1 + movement.1 * duration / total_duration;
        (x.round(), y.round())
    }

    #[allow(dead_code)]
    fn set_idle_from_registry(&mut self, registry: &AnimationRegistry, interval: u64) {
        let template = registry.get_template(&"idle".to_string()).expect("Expected idle template");
        self.set_idle_animation(template, interval);
    }

    pub fn add_idle_animation(&mut self, template: &AnimationTemplate, interval: u64) {
        //if self.idle_start.is_none() {
            //self.idle_start = Some(IdleStart::new(start_time, position));
        //}
        let interval = Duration::from_secs(interval);
        self.idle_interval = Some(interval);
        let animation = IdleInstance::new(template);
        self.idle_animations.push(animation);
    }

    // "set" supposes a singular entity. "add" supposes a collection of entities.
    // Is it really necessary? if we need only one instance, we may use idle_animations[0]
    pub fn set_idle_animation(&mut self, template: &AnimationTemplate, interval: u64) {
        self.idle_animations.clear();
        self.add_idle_animation(template, interval);
    }

    fn get_idle_animation(&self, now: Instant) -> Option<OutputFrame> {

        match (self.idle_interval, self.idle_animations.get(0), self.idle_start) {
            (Some(interval), Some(instance), Some(idle_start)) => {
                let mut start = idle_start.start_time;
                while start + interval + instance.duration <= now {
                    start += interval + instance.duration;
                }
                let animation_start = start + interval;
                if animation_start > now {
                    return None;
                }
                let mut time = now - animation_start;
                let mut tile_id: u32  = 0; 
                for frame in &instance.frames {
                    if time < frame.duration {
                        tile_id = frame.tile_id;
                        break;
                    }
                    time -= frame.duration;
                }
                let frame = OutputFrame {
                    tile_id,
                    position: idle_start.position,
                };
                Some(frame)
            }
            _ => None
        }
    }
}

#[derive(Copy, Clone)]
pub struct IdleStart {
    start_time: Instant,
    position: (f32, f32),
}

impl IdleStart {
    pub fn new(start_time: Instant, position: (f32, f32)) -> Self {
        Self {
            start_time,
            position,
        }
    }
}

#[derive(Clone)]
struct IdleInstance {
    pub frames: Vec<AnimationFrame>,
    pub duration: Duration,
}

impl IdleInstance {
    pub fn new(template: &AnimationTemplate) -> Self {
        let total_ticks = template.frames.iter().map(|it| it.duration.as_ticks()).sum();
        Self {
            duration: Duration::from_ticks(total_ticks),
            frames: template.frames.clone(),
        }
    }
}

pub struct AnimationRegistry {
    animations: HashMap<String, u32>,
    templates: HashMap<u32, AnimationTemplate>,
}

impl AnimationRegistry {

    pub fn load(tileset: &Tileset) -> Self {

        let mut animations: HashMap<String, u32> = HashMap::new();
        let mut templates = HashMap::new();

        for tile in tileset.tiles.iter() {
            if let Some(value) = tile.properties.get("name") {
                if let (PropertyValue::StringValue(name), Some(frames)) = (value, &tile.animation) {
                    animations.insert(name.clone(), tile.id);

                    let template = AnimationTemplate {
                        name: name.clone(),
                        gid: tile.id,
                        frames: frames.iter().map(|it| it.into()).collect(),
                        ordering: 0,
                        // todo: read these from Properties.
                        max_compression: 40,
                        blocks_turn: true,
                        cancel_frame: None
                    };

                    templates.insert(tile.id, template);
                }
            }
        }

        // TODO: Add custom properties for other template fields:
        // compression, blocks_turn, cancel_frame

        Self { animations, templates }
    }

    /// Maybe we only need one of these two
    pub fn get_animation_id(&self, template: &str) -> Option<u32> {
        self.animations
            .get(template)
            .cloned()
    }

    pub fn get_template(&self, template: &str) -> Option<&AnimationTemplate> {
        match self.animations.get(template) {
            None => None,
            Some(id) => self.templates.get(id)
        }
    }
}

//fn main(){
//todo!();
//}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;
    use coarsetime::{Duration, Instant};

    use super::*;

    fn mock_template(frames: Vec<AnimationFrame>, max_compression: u32) -> AnimationTemplate {
        AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression,
            blocks_turn: false,
            cancel_frame: None
        }
    }

    //total duration: 1000 ms, for 4 frames.
    fn mock_frames1243(ids: RangeInclusive<u32>) -> Vec<AnimationFrame> {
        let mut result = vec![];
        let durations = [100, 200, 400, 300];
        for (index, tile_id) in ids.enumerate() {
            result.push(AnimationFrame {
                tile_id,
                duration: Duration::from_millis(durations[index % durations.len()]),
            });
        }
        result
    }

    #[test]
    fn incorrect_compression() {
        let mut controller = AnimationController::new();
        let time_start = Instant::now();
        let mut current_time = time_start;
        let mut start = 1;
        let mut end = 4;
        let animation_start = time_start;
        let mut time_points: Vec<i32> = vec![];
        let durations = [100, 200, 400, 300];
        let mut time_point = 0;

        for i in 1..5 {
            let mut compression: u32 = 0;

            for i in &durations {
                time_point += i*compression/100;
                time_points.push(time_point as i32);
            }

            let template = mock_template(mock_frames1243(start..=end), compression);
            controller.add_animation(animation_start, &template, (1000.0, 100.0), (0.,0.));
            start += 4;
            end += 4;
            println!("for {} compression = {}", i, compression);
            // FIXME: Dead code (value assigned to `compression` is never read)
            // Really it IS read. I don't know why compiler return this error
            compression += 50;
        };
        println!("{:?}", time_points);

        let mut time: i32 = 0;
        loop {
            controller.update(current_time);
            let frame = match controller.get_frame(current_time) {
                Some(frame) => frame,
                None => break,
            };
            println!("At current time {} tileid is {}, position is {},{}", time, frame.tile_id, frame.position.0, frame.position.1);

            let mut expected_tileid = 0;
            for (index, time_point) in time_points.iter().enumerate() {
                if time < *time_point {
                    expected_tileid = index as u32 + 5;
                    break;
                }
            }
            let expected_position = {
                let x;
                if time < 500 {
                    x = time * 2;
                } else if time < 1500 {
                    x = 1000 + time - 500;
                } else {
                    x = 2000 + time - 2500;
                }
                let y = x / 10;
                (x as f32, y as f32)
            };
            println!("At current time {} expected tileid is {}, expected position is {},{}", time, expected_tileid, expected_position.0, expected_position.1);
            assert_eq!(frame.tile_id, expected_tileid);
            assert_eq!(frame.position, expected_position);
            time += 50;
            current_time = time_start + Duration::from_millis(time as u64);
        }
        println!("Finish!!!");
        let frame = controller.get_frame(current_time);
        assert!(frame.is_none());
    }

    macro_rules! assert_pos_almost_eq {
        ($x:expr, $y:expr, $d:expr) => {
            if (($x.0 - $y.0).abs() > $d || ($x.1 - $y.1).abs() > $d) {
                panic!("Expected: {:?} and actual: {:?} are different", $x, $y);
            }
        }
    }


    struct TestState {
        pub controller: AnimationController,
        pub start_time: Instant,
        pub now: Instant,
    }

    impl TestState {
        pub fn new() -> Self {
            Self {
                controller: AnimationController::new(),
                start_time: Instant::now(),
                now: Instant::recent(),
            }
        }

        pub fn assert_frame_at(&mut self, now_ms: u64, tile_id: u32, expected_pos: (f32, f32)) {
            self.now = self.start_time + Duration::from_millis(now_ms);
            self.controller.update(self.now);

            let frame_now = self.controller.get_frame(self.now)
                .expect("Frame expected");

            assert_eq!(frame_now.tile_id, tile_id, "tiles_id differ: expected {}, got {}", tile_id, frame_now.tile_id);
            assert_pos_almost_eq!(expected_pos, frame_now.position, 1.1);
        }

        pub fn assert_in_interval(&mut self, now_ms: u64, expected_tile_id: u32, expected_pos: (f32, f32)) {
            let mut start: u64 = 0;
            if now_ms > 5 {
                start = now_ms - 5;
            }

            //if expected tile_id is present in time interval now_ms +- 5 ms
            let mut tile_id = false;
            let mut got_tile_id: u32 = 0;
            for i in start..(start + 11) {
                self.now = self.start_time + Duration::from_millis(i);
                self.controller.update(self.now);
                if self.controller.get_frame(self.now).is_none() {
                    continue;
                }
                let frame_now = self.controller.get_frame(self.now).unwrap();
                if frame_now.tile_id == expected_tile_id {
                    tile_id = true;
                    break;
                }
                if i == now_ms {
                    got_tile_id = frame_now.tile_id;
                }
            }
            assert!(tile_id, "At {} tile_id is not {}, it is {}", now_ms, expected_tile_id, got_tile_id);

            //if real position == expected position +- 1
            let mut pos = false;
            let mut got_frame_pos = (0., 0.);
            for i in start..(start + 11) {
                self.now = self.start_time + Duration::from_millis(i);
                self.controller.update(self.now);
                if self.controller.get_frame(self.now).is_none() {
                    continue;
                }
                let frame_now = self.controller.get_frame(self.now).unwrap();
                let frame_pos = frame_now.position;
                if (expected_pos.0 - frame_pos.0).abs() <= 1.
                    && (expected_pos.1 - frame_pos.1).abs() <= 1. {
                    pos = true;
                    break;
                }
                if i == now_ms {
                    got_frame_pos = frame_pos;
                }
            }
            assert!(pos, "At {} position is not {:?}, it is {:?}", now_ms, expected_pos, got_frame_pos);
        }

        pub fn assert_empty_at(&mut self, now_ms: u64) {
            self.now = self.start_time + Duration::from_millis(now_ms);
            self.controller.update(self.now);
            assert!(self.controller.get_frame(self.now).is_none())
        }

        pub fn assert_animation_characteristics(&mut self, number: usize, frames_len: usize, start: u64, duration: u64, start_pos: (f32, f32), movement: (f32, f32)) {
            if let Some(animation) = self.controller.animations.get(number) {
                assert_eq!(frames_len, animation.frames.len(), "expected number of frames is {}, got {}", frames_len, animation.frames.len());
                for f in &animation.frames {
                    println!("frame duration is {}", f.duration.as_millis());
                }
                let animation_start = (animation.animation_start - self.start_time).as_millis();
                assert_eq!(start, animation_start, "animation doesn't start at {}, it starts at {} ", start, animation_start);
                assert_eq!(duration, animation.duration.as_millis(), "animation duration is not {}, it is {}", duration, animation.duration.as_millis());
                assert_eq!(start_pos, animation.start_position, "start position is not {:?}, it is {:?}", start_pos, animation.start_position);
                assert_eq!(movement, animation.movement, "movement is not {:?}, it is {:?}", movement, animation.movement);
            }
        }
    }

    #[test]
    pub fn test_movement() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 100);
        state.controller.add_animation(state.now, &template, (1000.0, 100.0), (0., 0.));

        state.assert_frame_at(0, 1, (0., 0.));

        state.assert_frame_at(99, 1, (99., 9.));
        state.assert_frame_at(101, 2, (101., 10.));
        state.assert_frame_at(151, 2, (151., 15.));

        state.assert_empty_at(1000);
    }

    #[test]
    pub fn test_movement_in_interval() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 100);
        state.controller.add_animation(state.now, &template, (1000.0, 100.0), (0., 0.));

        state.assert_in_interval(0, 1, (0., 0.));

        state.assert_in_interval(99, 1, (99., 9.));
        state.assert_in_interval(101, 2, (101., 10.));
        state.assert_in_interval(150, 2, (151., 15.));
        state.assert_in_interval(199, 2, (199., 20.));
        state.assert_in_interval(999, 4, (999., 100.));
        state.assert_animation_characteristics(0, 4, 0, 999, (0., 0.), (1000., 100.));

        state.assert_empty_at(1000);
    }

    #[test]
    fn test_2_instances() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 100);
        state.controller.add_animation(state.now, &template, (1000.0, 100.0), (0., 0.));
        let template = mock_template(mock_frames1243(5..=8), 100);
        state.controller.add_animation(state.now, &template, (1000.0, 100.0), (0., 0.));

        state.assert_in_interval(0, 1, (0., 0.));
        state.assert_in_interval(90, 1, (90., 9.));
        state.assert_in_interval(100, 2, (100., 10.));
        state.assert_in_interval(1090, 5, (1090., 109.));
        state.assert_in_interval(1100, 6, (1100., 110.));
        state.assert_empty_at(2000);
    }

    #[test]
    fn test_right_up_compressed() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 0.0),
            (1000., 300.));
        let template = mock_template(mock_frames1243(5..=8), 50);
        state.controller.add_animation(
            state.now, &template,
            (0.0, -100.0),
            (1100., 200.));
        state.assert_animation_characteristics(0,4,0,499,(1000.,300.),(100.,0.));
        state.assert_animation_characteristics(1,4,499,499,(1100.,300.),(0.,-100.));



        // 500ms to go right
        state.assert_frame_at(0, 1, (1000., 300.));
        state.assert_frame_at(49, 1, (1009., 300.));
        state.assert_frame_at(50, 2, (1010., 300.));
        state.assert_frame_at(499, 4, (1100., 300.));

        // 500ms to go up.
        state.assert_frame_at(500, 5, (1100., 300.));
        // + 300ms = +60% of the animation and its movement.
        state.assert_frame_at(800, 7, (1100., 240.));
        state.assert_frame_at(999, 8, (1100., 200.));

        state.assert_empty_at(1000);
    }

    #[test]
    fn test_right_up_compressed_inflight() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 0.0),
            (1000., 300.));

        state.assert_frame_at(350, 3, (1035., 300.));

        // Note this happens after 350ms.
        // Only the remaining part of the present animation should be compressed!
        let template = mock_template(mock_frames1243(5..=8), 50);
        state.controller.add_animation(
            state.now, &template,
            (0.0, -100.0),
            (1100., 200.));

        for i in &state.controller.animations {
            println!("number of frames is {}", i.frames.len());
            for f in &i.frames {
                println!("frame duration is {}", f.duration.as_millis());
            }
            println!("animation starts at {}", (i.animation_start - state.start_time).as_millis());
            println!("animation duration is {}", i.duration.as_millis());
            println!("start position is {:?}", i.start_position);
            println!("movement is {:?}", i.movement);
        }

        // At the time of 350 the third frame is passing 
        // The compression starts immediately
        // The rest of first animation will be compressed ~ to 325
        // The first animation finishes at 350+ 325 = 675 
        state.assert_frame_at(673, 4, (1100., 300.));

        // Transition into the second animation.
        // It should start ~ at 675
        // and last ~ till 675+500=1175
        state.assert_frame_at(675, 5, (1100., 300.));

        state.assert_frame_at(1173, 8, (1100., 200.));
        state.assert_empty_at(1175);
    }


    #[test]
    fn test_right_up_compressed_when_frame_starts() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 0.0),
            (1000., 300.));

        state.assert_frame_at(299, 2, (1030., 300.));

        // At the time of 299 the second frame is passing 
        // The compression starts immediately
        let template = mock_template(mock_frames1243(5..=8), 50);
        state.controller.add_animation(
            state.now, &template,
            (0.0, -100.0),
            (1100., 200.));

        state.assert_frame_at(300, 3, (1030., 300.));
        // The rest of first animation will be compressed ~ to 350
        // The first animation finishes at 300+ 350 = 650 
        state.assert_frame_at(647, 4, (1100., 300.));

        // Transition into the second animation.
        // It should start ~ at 650
        // and last ~ till 650+500=1150
        state.assert_frame_at(650, 5, (1100., 300.));
        state.assert_frame_at(1147, 8, (1100., 200.));
        state.assert_empty_at(1150);
    }

    #[test]
    fn test_add_with_zero_compression() {
        let mut state = TestState::new();

        //add with 0 compression at the beginning
        let template = mock_template(mock_frames1243(1..=4), 0);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        assert_eq!(1, state.controller.animations.len());
        //first animation is dropped, second is added uncompressed
        state.assert_animation_characteristics(0,4, 0, 999, (0.,0.), (100.,10.));

        //add with 0 compression at the end
        let template = mock_template(mock_frames1243(1..=4), 0);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        assert_eq!(1, state.controller.animations.len());
        state.assert_animation_characteristics(0,4, 0, 999, (0.,0.), (100.,10.));

        //add with 0 compression between two normal compression
        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        let template = mock_template(mock_frames1243(1..=4), 0);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        let template = mock_template(mock_frames1243(1..=4), 50);
        state.controller.add_animation(
            state.now, &template,
            (100.0, 10.0),
            (0., 0.));

        assert_eq!(3, state.controller.animations.len());
        state.assert_animation_characteristics(0,4, 0, 499, (0.,0.), (100.,10.));
        state.assert_animation_characteristics(1,4, 499, 499, (100.,10.), (100.,10.));
        state.assert_animation_characteristics(2,4, 999, 499, (200.,20.), (100.,10.));
    }

    #[test]
    fn test_square_walking() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 100);
        state.controller.add_animation(
            state.now, &template,
            (100., 0.),
            (200., 200.));

        let template = mock_template(mock_frames1243(5..=8), 100);
        state.controller.add_animation(
            state.now, &template,
            (0., 100.),
            (0., 0.));

        let template = mock_template(mock_frames1243(9..=12), 100);
        state.controller.add_animation(
            state.now, &template,
            (-100., 0.),
            (0., 0.));

        let template = mock_template(mock_frames1243(13..=16), 100);
        state.controller.add_animation(
            state.now, &template,
            (0., -100.),
            (0., 0.));

        state.assert_frame_at(999, 4, (300., 200.));
        state.assert_in_interval(1999, 8, (300., 300.));
        state.assert_in_interval(2999, 12, (200., 300.));
        //state.assert_animation_characteristics(3,4,2999,999,(200.,300.),(0.,-100.));
        //due to coarsetime crate bug the animation ends in 3999, not in 4000.
        //But assertion in interval allow us not to think about this bug
        state.assert_in_interval(4000, 16, (200., 200.));
        //and this method is for precious time
        state.assert_empty_at(4000);
    }

    #[test]
    fn test_idle_animation() {
        let mut state = TestState::new();

        let template = mock_template(mock_frames1243(1..=4), 100);
        state.controller.add_animation(
            state.now, &template,
            (100., 100.0),
            (0., 0.));
        let template = mock_template(mock_frames1243(101..=104), 100);
        state.controller.add_idle_animation(&template, 10);
        state.assert_in_interval(1, 1, (0.,0.));
        state.assert_in_interval(1000, 4, (100.,100.));
        state.assert_empty_at(1010);
        state.assert_in_interval(11000,101,(100.,100.));
        state.assert_in_interval(12000,104,(100.,100.));
        state.assert_empty_at(13000);
        state.assert_in_interval(22000,101,(100.,100.));
        state.assert_in_interval(23000,104,(100.,100.));
        state.assert_empty_at(24000);

    }
}
