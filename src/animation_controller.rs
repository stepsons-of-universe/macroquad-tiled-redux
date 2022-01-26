use std::collections::HashMap;
use coarsetime::{Duration, Instant};
use tiled::animation;
use tiled::properties::PropertyValue;
use tiled::tileset::Tileset;

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
    /// % of time this animation can be compressed to. E.g. running can be sped up by 20 percent.
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
    /// Time the last current frame (should have) started at.
    pub frame_start: Instant,

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
        let total_ticks = template.frames.iter().map(|it| it.duration.as_ticks() as u64).sum();
        Self {
            frame_start: start_time,
            duration: Duration::from_ticks(total_ticks),
            frames: template.frames.clone(),
            movement,
            start_position,
            max_compression: template.max_compression,
            is_compressed: false,
        }
    }
    
    pub fn compressed(&mut self, current_time: Instant) {
            let frames = self.frames.clone();
            let mut new_frames: Vec<AnimationFrame> = vec![];
            let mut start = self.state.frame_start.clone(); 
            let mut new_start = self.state.frame_start.clone(); 
            for i in &frames {
                if start+i.duration <= current_time {
                    new_start = start;
                }
                if start+i.duration > current_time {
                    let f =  AnimationFrame {
                        tile_id: i.tile_id,
                        duration: i.duration*self.max_compression/100,
                    };
                    new_frames.push(f);
                }
                start += i.duration;
            }
            let new_duration = new_frames.iter().map(|it| it.duration.as_ticks() as u64).sum();
            let k = (self.duration.as_ticks() as u64 * self.max_compression as u64 / (new_duration * 100)) as f32;
            let new_movement = ((self.movement.0 as f32 / k) as i32, (self.movement.1 as f32 / k) as i32);
            //self.state.frame_start = self.state.frame_start + (self.duration - Duration::from_ticks((new_duration as f32 * k) as u64));
            //или так
            self.state.frame_start = new_start;
            //или так
            //self.state.frame_start = current_time;
            self.frames = new_frames;
            self.duration = Duration::from_ticks(new_duration);
            self.movement = new_movement;
            self.is_compressed = true;
    }
}

/// Per-entity object that controls its animations.
#[derive(Clone)]
pub struct AnimationController {
    /// Current animations to be played.
    animations: Vec<AnimationInstance>,
    /// If had no animations for `idle_interval`, play one of `idle_animations`
    idle_interval: Option<Duration>,
    /// Idle animations get interrupted immediately.
    idle_animations: Vec<AnimationInstance>,
}

impl AnimationController {

    pub fn new() -> Self {
        // Create an empty instance.
        Self {
            animations: vec![],
            idle_interval: None,
            idle_animations: vec![],
        }
    }

    /// Discards the frames whose time is gone.
    pub fn update(&mut self, time: Instant) {
        if self.animations.len() != 0 {
            let animations = &mut self.animations;
            animations.retain(|i|i.frame_start + i.duration >= time);
        }
    }

    /// Returns (frame_id, (x, y)) for the given time moment, if there is
    /// a frame to show, otherwise None.
    /// Only goes down to current or next frame.
    pub fn get_frame(&self, time: Instant) -> Option<(u32, (f32, f32))> {
        match self.animations.get(0) {
            Some(i) => {
                let instance = i;
                let tile_id = AnimationController::get_tile_id(time, instance);
                let position = AnimationController::get_position(time, instance);
                let frame:(u32, (f32, f32)) = (tile_id, position);
                return Some(frame);
            }
            None => None
        }
    }

    pub fn add_animation(&mut self, start_time: Instant, template: &AnimationTemplate, movement: (f32, f32)) {
        let mut new_start_time = start_time;
        let mut new_start_position: (f32, f32) = (0.0, 0.0);
        if self.animations.len() != 0 {
            let i = self.animations.last().unwrap();
            new_start_time = i.frame_start + i.duration;
            new_start_position = (i.start_position.0 + i.movement.0 as f32, i.start_position.1 + i.movement.1 as f32)
        }
        let instance = AnimationInstance::new(new_start_time, template, movement, new_start_position);
        self.animations.push(instance);
    }
    
    pub fn add_animation_with_compression(&mut self, start_time: Instant, template: &AnimationTemplate, movement: (i32, i32)) {
        let mut new_start_time = start_time;
        let mut new_start_position: (f32, f32) = (0.0, 0.0);
        if self.animations.len() != 0 {
            self.get_compressed(start_time);
            let i = self.animations.last().unwrap();
            new_start_time = i.state.frame_start + i.duration;
            new_start_position = (i.start_position.0 + i.movement.0 as f32, i.start_position.1 + i.movement.1 as f32)
        }
        let instance = AnimationInstance::new_movement(new_start_time, template, movement, new_start_position);
        self.animations.push(instance);
    }
    
    pub fn get_compressed(&mut self, time: Instant) {
            let mut animations = self.animations.clone();
            for i in &mut animations {
                if !i.is_compressed {
                    i.compressed(time);
                }
            }
            self.animations = animations;
    }

    fn get_tile_id(finish_time: Instant, instance: &AnimationInstance) -> u32 {
        let frames = &instance.frames;
        let mut tile_id = 0;
        let start_time = instance.frame_start;
        let mut time = finish_time - start_time;
        for i in frames {
            if time < i.duration {
                tile_id = i.tile_id;
                break;
            }
            time -= i.duration;
        }
        tile_id
    }

    fn get_position(finish_time:Instant, instance: &AnimationInstance) -> (f32,f32) {
        let movement = instance.movement;
        let start_position = instance.start_position;
        let start_time = instance.frame_start;
        let duration = (finish_time - start_time).as_ticks() as f32;
        let total_duration = instance.duration.as_ticks() as f32;
        let x = start_position.0 as f32 + ((movement.0 as f32 * duration) / total_duration) as f32;
        let y = start_position.1 as f32 + ((movement.1 as f32 * duration) / total_duration) as f32;
        (x.round(), y.round())
    }
}

/// All the animations for a specific entity (character)
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
                match (value, &tile.animation) {
                    (PropertyValue::StringValue(name), Some(frames)) => {
                        animations.insert(name.clone(), tile.id);

                        let template = AnimationTemplate {
                            name: name.clone(),
                            gid: tile.id,
                            frames: frames.iter().map(|it| it.into()).collect(),
                            ordering: 0,
                            // todo: read these from Properties.
                            max_compression: 0,
                            blocks_turn: true,
                            cancel_frame: None
                        };

                        templates.insert(tile.id, template);
                    }

                    _ => {}
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

#[cfg(test)]
mod tests {
    use coarsetime::{Duration, Instant};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn test_frame0() {
        let mut controller = AnimationController::new();
        let time_start = Instant::now();
        let now = time_start;
        //// total duration: 1000 ms
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 1, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 2, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 3, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 4, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ////ordering: 0,
            ////max_compression: 0,
            ////blocks_turn: false,
            ////cancel_frame: None,
            ordering: 0,
            max_compression: 0,
            blocks_turn: false,
            cancel_frame: None
        };

        controller.add_animation(now, &template, (1000.0, 100.0));

        controller.update(now);
        let frame_at_0 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_0.0, 1);
        assert_eq!(frame_at_0.1, (0.0, 0.0));

        let now = time_start + Duration::from_millis(90);
        println!("90ms: {}", (now - time_start).as_millis());
        controller.update(now);
        let frame_at_91 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_91.0, 1);
        assert_eq!(frame_at_91.1, (90.0, 9.0));

        let now = time_start + Duration::from_millis(100);
        let frame_at_100 = controller.get_frame(now)
            .expect("Frame expected");
        //// it's time for tile 2, b/c first frame duration is 100ms
        assert_eq!(frame_at_100.0, 2);
        assert_eq!(frame_at_100.1, (100.0, 10.0));

        let now = time_start + Duration::from_millis(101);
        controller.update(now);
        let frame_at_101 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_101.0, 2);
        assert_eq!(frame_at_101.1, (101.0, 10.0));

        let now = time_start + Duration::from_millis(151);
        controller.update(now);
        let frame_at_200 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_200.0, 2);
        assert_eq!(frame_at_200.1, (151.0, 15.0));
        let now = time_start + Duration::from_millis(1000);
        controller.update(now);
        let frame_at_1000 = controller.get_frame(now);
        assert_eq!(frame_at_1000, None);
    }

    #[test]
    fn test_2_instances() {
        let mut controller = AnimationController::new();
        let time_start = Instant::now();
        let now = time_start;
        //// total duration: 1000 ms
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 1, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 2, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 3, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 4, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression: 0,
            blocks_turn: false,
            cancel_frame: None
        };

        controller.add_animation(now, &template, (1000.0, 100.0));
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 5, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 6, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 7, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 8, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression: 0,
            blocks_turn: false,
            cancel_frame: None,
        };
        controller.add_animation(now, &template, (1000.0, 100.0));
        println!("{}", controller.animations.len());
        for i in &controller.animations {
            println!("{},{}", i.start_position.0,i.start_position.1);
        };
        controller.update(now);
        let frame_at_0 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_0.0, 1);
        assert_eq!(frame_at_0.1, (0.0, 0.0));

        let now = time_start + Duration::from_millis(90);
        println!("90ms: {}", (now - time_start).as_millis());
        controller.update(now);
        let frame_at_91 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_91.0, 1);
        assert_eq!(frame_at_91.1, (90.0, 9.0));

        let now = time_start + Duration::from_millis(100);
        let frame_at_100 = controller.get_frame(now)
            .expect("Frame expected");
        //// it's time for tile 2, b/c first frame duration is 100ms
        assert_eq!(frame_at_100.0, 2);
        assert_eq!(frame_at_100.1, (100.0, 10.0));

        let now = time_start + Duration::from_millis(1090);
        controller.update(now);
        let frame_at_1090 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_1090.0, 5);
        assert_eq!(frame_at_1090.1, (1090.0, 109.0));

        let now = time_start + Duration::from_millis(1100);
        controller.update(now);
        let frame_at_1100  = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_1100.0, 6);
        assert_eq!(frame_at_1100.1, (1100.0, 110.0));

        let now = time_start + Duration::from_millis(2000);
        controller.update(now);
        let frame_at_2000 = controller.get_frame(now);
        assert_eq!(frame_at_2000, None);
    }
    
    #[test]
    fn test_3_instances_with_compression() {
        let mut controller = AnimationController::new();
        let time_start = Instant::now();
        let now = time_start;
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 1, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 2, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 3, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 4, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression: 50,
            blocks_turn: false,
            cancel_frame: None,
        };
        controller.add_animation_with_compression(now, &template, (1000, 100));

        let now = time_start + Duration::from_millis(1);
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 5, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 6, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 7, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 8, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression: 50,
            blocks_turn: false,
            cancel_frame: None,
        };
        controller.add_animation_with_compression(now, &template, (1000, 100));
        let now = time_start + Duration::from_millis(2);
        let frames: Vec<AnimationFrame> = vec![
            AnimationFrame { tile_id: 9, duration: Duration::from_millis(100), },
            AnimationFrame { tile_id: 10, duration: Duration::from_millis(200), },
            AnimationFrame { tile_id: 11, duration: Duration::from_millis(400), },
            AnimationFrame { tile_id: 12, duration: Duration::from_millis(300), },
        ];
        let template = AnimationTemplate {
            name: "dummy".to_string(),
            gid: 1,
            frames,
            ordering: 0,
            max_compression: 50,
            blocks_turn: false,
            cancel_frame: None,
        };
        controller.add_animation_with_compression(now, &template, (1000, 100));
        println!("{}", controller.animations.len());
        for i in &controller.animations {
            println!("{},{}", i.start_position.0,i.start_position.1);
        };
        for y in &controller.animations{
            for i in &y.frames {
                println!("{}", i.duration.as_millis());
            }
        }

        controller.update(now);
        let frame_at_0 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_0.0, 1);
        //assert_eq!(frame_at_0.1, (0.0, 0.0));
        println!("position 3.0, 0.0 = {}, {}", frame_at_0.1.0, frame_at_0.1.1);

        let now = time_start + Duration::from_millis(45);
        controller.update(now);
        let frame_at_45 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_45.0, 1);
        println!("position 90.0, 9.0 = {}, {}", frame_at_45.1.0, frame_at_45.1.1);
        //assert_eq!(frame_at_45.1, (90.0, 9.0));

        let now = time_start + Duration::from_millis(51);
        let frame_at_50 = controller.get_frame(now)
           .expect("Frame expected");
        assert_eq!(frame_at_50.0, 2);
        //assert_eq!(frame_at_50.1, (100.0, 10.0));
        println!("position 100.0, 10.0 = {}, {}", frame_at_50.1.0, frame_at_50.1.1);

        let now = time_start + Duration::from_millis(545);
        controller.update(now);
        let frame_at_545 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_545.0, 5);
        //assert_eq!(frame_at_545.1, (1090.0, 109.0));
        println!("position 1090.0, 109.0 = {}, {}", frame_at_545.1.0, frame_at_545.1.1);

        let now = time_start + Duration::from_millis(551);
        controller.update(now);
        let frame_at_551  = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_551.0, 6);
        //assert_eq!(frame_at_600.1, (1100.0, 110.0));
        println!("position 1100.0, 110.0 = {}, {}", frame_at_551.1.0, frame_at_551.1.1);
        
        let now = time_start + Duration::from_millis(1090);
        controller.update(now);
        let frame_at_1090 = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_1090.0, 9);
        //assert_eq!(frame_at_590.1, (1090.0, 109.0));
        println!("position 2090.0, 209.0 = {}, {}", frame_at_1090.1.0, frame_at_1090.1.1);

        let now = time_start + Duration::from_millis(1103);
        controller.update(now);
        let frame_at_1103  = controller.get_frame(now)
            .expect("Frame expected");
        assert_eq!(frame_at_1103.0, 10);
        //assert_eq!(frame_at_600.1, (1100.0, 110.0));
        println!("position 2100.0, 210.0 = {}, {}", frame_at_1103.1.0, frame_at_1103.1.1);


        let now = time_start + Duration::from_millis(2000);
        controller.update(now);
        let frame_at_2000 = controller.get_frame(now);
        assert_eq!(frame_at_2000, None);
    }
}