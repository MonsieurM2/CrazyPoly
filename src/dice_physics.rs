//! The dice throw, simulated head-less with the generic `rapier3d` engine.
//!
//! A tiny rigid-body world is built on demand: a fixed floor, four fixed walls
//! (the tray) and two dynamic cubes. The world is stepped at 240 Hz while the
//! pose of both cubes is recorded every second step, until both dice have come
//! to rest. Playing back those keyframes is all the renderer does, so the dice
//! on screen move exactly like the simulated ones.
//!
//! The result of a throw is not chosen: it is *read* from the simulation. Once
//! the cubes are at rest the face pointing at the sky is looked up in the die's
//! face table, and that value is what the game uses. A short final segment
//! eases each cube onto its flat resting pose, so the top face is always fully
//! visible when the throw ends.

use rapier3d::prelude::*;

use crate::dice::{flatten, m3_blend, norm3, top_value, HOME_X, M3, V3};

/// One recorded pose of a die, in die-edge units.
#[derive(Clone, Copy)]
pub struct Frame {
    pub pos: V3,
    pub rot: M3,
}

/// A finished throw: what the dice show, and how they got there.
pub struct Throw {
    pub values: (u8, u8),
    pub frames: [Vec<Frame>; 2],
    /// Playback length in seconds (the recorded duration plus the settle ease).
    pub duration: f32,
}

pub const FRAME_DT: f32 = 1.0 / 120.0;

const HALF: f32 = 0.5;
const TRAY: f32 = 1.9;
const WALL: f32 = 0.3;
const WALL_H: f32 = 6.0;
const REST_Z: f32 = HALF;
const GRAVITY: f32 = 55.0;
const PHYS_DT: f32 = 1.0 / 240.0;
const STEPS_PER_FRAME: usize = 2;
const MAX_TIME: f32 = 3.0;
const MIN_TIME: f32 = 0.9;
const QUIET_TIME: f32 = 0.30;
/// Time the dice take to come back to their home spots once they have stopped.
const RETURN_TIME: f32 = 0.45;
/// Height of the little hop they make on the way back.
const RETURN_HOP: f32 = 0.45;

fn rand_range(a: f32, b: f32) -> f32 {
    macroquad::rand::gen_range(a, b)
}

fn v3(a: f32, b: f32, c: f32) -> Vec3 {
    Vec3::new(a, b, c)
}

fn pose_of(bodies: &RigidBodySet, handle: RigidBodyHandle) -> Frame {
    let pose = bodies[handle].position();
    let ex = pose.transform_vector(v3(1.0, 0.0, 0.0));
    let ey = pose.transform_vector(v3(0.0, 1.0, 0.0));
    let ez = pose.transform_vector(v3(0.0, 0.0, 1.0));
    let t = pose.translation;
    Frame {
        pos: [t.x, t.y, t.z],
        rot: [
            [ex.x, ey.x, ez.x],
            [ex.y, ey.y, ez.y],
            [ex.z, ey.z, ez.z],
        ],
    }
}

/// Simulate one throw of both dice and record it.
pub fn roll() -> Throw {
    let gravity = v3(0.0, 0.0, -GRAVITY);

    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();

    // ---- the tray: one thick floor and four tall walls, all immovable.
    let floor = bodies.insert(
        RigidBodyBuilder::fixed()
            .translation(v3(0.0, 0.0, -1.0))
            .build(),
    );
    colliders.insert_with_parent(
        ColliderBuilder::cuboid(TRAY + WALL, TRAY + WALL, 1.0)
            .friction(0.9)
            .restitution(0.28)
            .build(),
        floor,
        &mut bodies,
    );

    let walls = [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
    ];
    for (sx, sy) in walls {
        let body = bodies.insert(
            RigidBodyBuilder::fixed()
                .translation(v3(sx * (TRAY + WALL), sy * (TRAY + WALL), WALL_H * 0.5))
                .build(),
        );
        let hx = if sx == 0.0 { TRAY + WALL } else { WALL };
        let hy = if sy == 0.0 { TRAY + WALL } else { WALL };
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(hx, hy, WALL_H * 0.5)
                .friction(0.45)
                .restitution(0.35)
                .build(),
            body,
            &mut bodies,
        );
    }

    // ---- two cubes, dropped into the tray with a random shove and spin.
    let mut handles = [RigidBodyHandle::invalid(); 2];
    for (i, handle) in handles.iter_mut().enumerate() {
        let side = if i == 0 { -1.0 } else { 1.0 };
        let spin_axis = norm3([rand_range(-1.0, 1.0), rand_range(-1.0, 1.0), rand_range(-1.0, 1.0)]);
        let spin = rand_range(14.0, 32.0);
        let tilt_axis = norm3([rand_range(-1.0, 1.0), rand_range(-1.0, 1.0), rand_range(-1.0, 1.0)]);
        let tilt = rand_range(0.0, std::f32::consts::TAU);

        let body = RigidBodyBuilder::dynamic()
            .translation(v3(
                side * rand_range(0.45, 0.85),
                rand_range(-0.35, 0.35),
                rand_range(1.20, 1.95),
            ))
            .rotation(v3(
                tilt_axis[0] * tilt,
                tilt_axis[1] * tilt,
                tilt_axis[2] * tilt,
            ))
            .linvel(v3(
                side * rand_range(1.2, 3.4) + rand_range(-0.8, 0.8),
                rand_range(-2.6, 2.6),
                rand_range(-2.2, -0.4),
            ))
            .angvel(v3(
                spin_axis[0] * spin,
                spin_axis[1] * spin,
                spin_axis[2] * spin,
            ))
            .linear_damping(0.4)
            .angular_damping(0.7)
            .ccd_enabled(true)
            .build();
        *handle = bodies.insert(body);
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(HALF, HALF, HALF)
                .density(1.0)
                .friction(0.75)
                .restitution(0.34)
                .build(),
            *handle,
            &mut bodies,
        );
    }

    // ---- step the world and record both poses along the way.
    let mut pipeline = PhysicsPipeline::new();
    let mut islands = IslandManager::new();
    let mut broad_phase = BroadPhaseBvh::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut multibody_joints = MultibodyJointSet::new();
    let mut soft_bodies = SoftBodySet::new();
    let mut ccd_solver = CCDSolver::new();

    let mut params = IntegrationParameters::default();
    params.dt = PHYS_DT;

    let mut tracks: [Vec<Frame>; 2] = [
        Vec::with_capacity((MAX_TIME / FRAME_DT) as usize + 32),
        Vec::with_capacity((MAX_TIME / FRAME_DT) as usize + 32),
    ];
    for i in 0..2 {
        tracks[i].push(pose_of(&bodies, handles[i]));
    }

    let steps = (MAX_TIME / PHYS_DT) as usize;
    let mut elapsed = 0.0f32;
    let mut quiet = 0.0f32;
    for step in 0..steps {
        pipeline.step(
            gravity,
            &params,
            &mut islands,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut impulse_joints,
            &mut multibody_joints,
            &mut soft_bodies,
            &mut ccd_solver,
            &(),
            &(),
        );
        elapsed += PHYS_DT;

        let mut moving = false;
        for handle in handles {
            let body = &bodies[handle];
            if body.linvel().length() > 0.30 || body.angvel().length() > 0.70 {
                moving = true;
            }
        }
        if moving {
            quiet = 0.0;
        } else {
            quiet += PHYS_DT;
        }

        if step % STEPS_PER_FRAME == STEPS_PER_FRAME - 1 {
            for i in 0..2 {
                tracks[i].push(pose_of(&bodies, handles[i]));
            }
        }

        if elapsed >= MIN_TIME && quiet >= QUIET_TIME {
            break;
        }
    }
    for i in 0..2 {
        tracks[i].push(pose_of(&bodies, handles[i]));
    }

    // ---- read the value off the upper face of each cube.
    let mut last = [tracks[0][tracks[0].len() - 1], tracks[1][tracks[1].len() - 1]];
    let values = (top_value(&last[0].rot), top_value(&last[1].rot));

    // ---- then bring each cube back to its home spot: it is laid flat on the
    // very face it landed on (same value, same way round) and hops back home.
    let return_frames = ((RETURN_TIME / FRAME_DT).round() as usize).max(2);
    for i in 0..2 {
        let from = last[i];
        let value = if i == 0 { values.0 } else { values.1 };
        let target = flatten(&from.rot);
        let home = [if i == 0 { -HOME_X } else { HOME_X }, 0.0, REST_Z];
        for k in 1..=return_frames {
            let w = k as f32 / return_frames as f32;
            let ease = w * w * (3.0 - 2.0 * w);
            let hop = (std::f32::consts::PI * w).sin() * RETURN_HOP;
            let pos = [
                from.pos[0] + (home[0] - from.pos[0]) * ease,
                from.pos[1] + (home[1] - from.pos[1]) * ease,
                from.pos[2] + (home[2] - from.pos[2]) * ease + hop,
            ];
            let rot = m3_blend(&from.rot, &target, ease);
            tracks[i].push(Frame { pos, rot });
        }
        last[i] = *tracks[i].last().unwrap();
        debug_assert_eq!(
            top_value(&last[i].rot),
            value,
            "a die returning home must keep the value it landed on"
        );
    }

    let longest = tracks[0].len().max(tracks[1].len());
    Throw {
        values,
        frames: tracks,
        duration: (longest.saturating_sub(1)) as f32 * FRAME_DT,
    }
}

