// The Sift's sky, painted where the sky shows once the world is drawn (vanilla's renderer still
// draws the End's sky box underneath; this covers it), with or without a shader pack: a gradient
// from the fog's colour at the horizon to the sky's colour above, the sun by day and the moon by
// night where The Sift's timeline puts them. MIT; port of Mielon's The Sift (MIT).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ON: AtomicBool = AtomicBool::new(false);
static VALUES: [AtomicU32; 12] = [const { AtomicU32::new(0) }; 12];

/// The sky this frame: colours as stored (0..1), angles in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sky {
    pub sky: [f32; 3],
    pub fog: [f32; 3],
    pub sun_angle: f32,
    pub moon_angle: f32,
    /// 1 by day, 0 by night (which body shows).
    pub day: f32,
    /// Rain, 0 to 1: the bodies fade behind it.
    pub rain: f32,
}

/// From the game thread: paint the sky this frame (`on`: in The Sift), with `sky`.
pub fn set(on: bool, sky: Sky) {
    let v = [sky.sky[0], sky.sky[1], sky.sky[2], sky.fog[0], sky.fog[1], sky.fog[2], sky.sun_angle,
        sky.moon_angle, sky.day, sky.rain, 0.0, 0.0];
    for (slot, value) in VALUES.iter().zip(v) {
        slot.store(value.to_bits(), Ordering::Relaxed);
    }
    ON.store(on, Ordering::Release);
}

pub fn get() -> (bool, Sky) {
    let on = ON.load(Ordering::Acquire);
    let v: Vec<f32> = VALUES.iter().map(|s| f32::from_bits(s.load(Ordering::Relaxed))).collect();
    (on, Sky { sky: [v[0], v[1], v[2]], fog: [v[3], v[4], v[5]], sun_angle: v[6], moon_angle: v[7], day: v[8], rain: v[9] })
}

/// The world's depth is needed only while the sky is painted.
pub fn needs(world_depth: &mut bool) {
    if ON.load(Ordering::Relaxed) {
        *world_depth = true;
    }
}

/// A body's direction for its angle (SkyRenderer: Y by -90 degrees, then X by the angle).
pub fn body_direction(angle_degrees: f32) -> [f32; 3] {
    let t = angle_degrees.to_radians();
    [-t.sin(), t.cos(), 0.0]
}

/// The uniform's bytes: the clip-to-world matrix, the colours, the bodies, the sizes.
pub fn uniform_bytes(clip_to_world: [f32; 16], sky: &Sky, size: [u32; 2], depth_size: [u32; 2]) -> [u8; 144] {
    let mut f = [0f32; 36];
    f[..16].copy_from_slice(&clip_to_world);
    f[16..20].copy_from_slice(&[sky.sky[0], sky.sky[1], sky.sky[2], 1.0]);
    f[20..24].copy_from_slice(&[sky.fog[0], sky.fog[1], sky.fog[2], 1.0]);
    let sun = body_direction(sky.sun_angle);
    let moon = body_direction(sky.moon_angle);
    f[24..28].copy_from_slice(&[sun[0], sun[1], sun[2], sky.day.clamp(0.0, 1.0)]);
    f[28..32].copy_from_slice(&[moon[0], moon[1], moon[2], sky.rain.clamp(0.0, 1.0)]);
    f[32..36].copy_from_slice(&[size[0] as f32, size[1] as f32, depth_size[0] as f32, depth_size[1] as f32]);
    let mut out = [0u8; 144];
    for (chunk, v) in out.chunks_exact_mut(4).zip(f) {
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    out
}

