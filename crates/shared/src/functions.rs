use crate::{components::*, consts::*, resources::PlayerInput};
use bevy_ecs::prelude::*;
use bevy_rapier3d::{
    na::clamp,
    parry::shape::{Ball, Shape},
    prelude::*,
};
use glam::{Quat, Vec2, Vec3};
use std::f32::consts::PI;

pub fn integrate(position: Vec3, velocity: Vec3, dt: f32) -> Vec3 {
    position + velocity
}

pub fn spring_damper(k: f32, c: f32, x: f32, x_dot: f32, dt: f32) -> f32 {
    let f = (k * x) + (c * -x_dot) * dt;
    f.max(0.0)
}

pub fn apply_gravity(velocity: Vec3, scale: f32, dt: f32) -> Vec3 {
    let a = Vec3::new(0., GRAVITY, 0.) * scale * dt;
    velocity + a
}

pub fn project_and_scale(v: Vec3, n: Vec3) -> Vec3 {
    let mag: f32 = v.length();
    project_onto_plane(v, n).normalize_or_zero() * mag
}

pub fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
    v - n * v.dot(n)
}

pub fn collide_and_slide(
    position: Vec3,
    velocity: Vec3,
    shape: &dyn Shape,
    depth: u8,
    gravity_pass: bool,
    vel_init: Vec3,
    rapier_context: &RapierContext,
) -> Vec3 {
    // check if exceeded max depth
    if depth >= MAX_COLLISION_BOUNCES {
        return Vec3::ZERO;
    }

    let dist: f32 = velocity.length() + SKIN_WIDTH;

    if let Some(result) = rapier_context.cast_shape(
        position,
        Quat::IDENTITY,
        velocity.normalize(),
        shape,
        ShapeCastOptions::with_max_time_of_impact(dist),
        QueryFilter::only_fixed(),
    ) {
        let hit = result.1;
        let normal = hit.details.unwrap().normal1;

        let mut snap_to_surface: Vec3 =
            velocity.normalize_or_zero() * (hit.time_of_impact - SKIN_WIDTH);
        let mut leftover: Vec3 = velocity - snap_to_surface;
        let angle: f32 = Vec3::angle_between(Vec3::Y, normal).to_degrees();

        if snap_to_surface.length() <= SKIN_WIDTH {
            snap_to_surface = Vec3::ZERO;
        }

        // normal ground / slope
        if angle <= MAX_SLOPE_ANGLE {
            if gravity_pass {
                return snap_to_surface;
            }

            leftover = project_and_scale(leftover, normal);
        } else {
            let scale: f32 = 1.0
                - Vec3::dot(
                    Vec3::new(normal.x, 0., normal.z).normalize_or_zero(),
                    -Vec3::new(vel_init.x, 0., vel_init.z).normalize_or_zero(),
                );

            leftover = project_and_scale(leftover, normal) * scale;
        }

        return snap_to_surface
            + collide_and_slide(
                position + snap_to_surface,
                leftover,
                shape,
                depth + 1,
                gravity_pass,
                vel_init,
                rapier_context,
            );
    }

    velocity
}

fn get_wish_dir(input: &PlayerInput, rotation: Quat) -> Vec3 {
    let forward_vec = rotation * Vec3::Z;
    let side_vec = rotation * Vec3::X;

    let x = (input.right as i8 - input.left as i8) as f32;
    let y = (input.down as i8 - input.up as i8) as f32;

    (Vec3::ZERO + (forward_vec * y) + (side_vec * x)).normalize_or_zero()
}

fn apply_friction(vel: Vec3, dt: f32) -> Vec3 {
    let control = f32::max(vel.length(), PLAYER_DEACCEL);
    let drop = control * GROUND_FRICTION * dt;
    let mut new_speed = f32::max(vel.length() - drop, 0.0);

    if vel.length() > 0.0 {
        new_speed /= vel.length();
    }

    vel * new_speed
}

fn update_vel_ground(wishdir: Vec3, mut vel: Vec3, dt: f32) -> Vec3 {
    vel = apply_friction(vel, dt);

    let current_speed = Vec3::dot(vel, wishdir);
    let add_speed = (PLAYER_MOVE_SPEED - current_speed).clamp(0., PLAYER_ACCEL * dt);

    vel + add_speed * wishdir
}

pub fn simulate_player(
    rapier_context: &RapierContext,
    mut position: Vec3,
    rotation: Quat,
    mut velocity: Vec3,
    gravity: &mut Gravity,
    grounded: &mut Grounded,
    resting_height: f32,
    input: &PlayerInput,
    dt: f32,
) -> (Vec3, Vec3, Vec3, Quat) {
    // (position, velocity, wish dir, rotation)
    // get player rotation from look angles
    let new_rot = Quat::from_axis_angle(Vec3::Y, input.look_angles.yaw);
    let wish_dir = get_wish_dir(input, new_rot);

    velocity = update_vel_ground(wish_dir, velocity, dt);

    // apply jump input
    if input.jump && grounded.0 {
        gravity.vector.y += PLAYER_JUMP_POWER;
        grounded.0 = false;
    }

    // apply gravity
    gravity.vector = apply_gravity(gravity.vector, gravity.scale, dt);

    // apply ground check and spring damper
    if let Some(result) = rapier_context.cast_ray(
        position,
        Vec3::NEG_Y,
        resting_height,
        true,
        QueryFilter::only_fixed(),
    ) {
        grounded.0 = true;

        let x = resting_height - result.1;
        let x_dot = gravity.vector.y;

        let spring_force = spring_damper(1.0, 10.0, x, x_dot, dt);
        gravity.vector.y += spring_force;
    } else {
        grounded.0 = false;
    }

    // integrate with collide and slide
    // scale velocity by delta time for integration
    let move_vel: Vec3 = velocity * dt;
    let gravity_vel: Vec3 = gravity.vector * dt;

    let shape = Ball::new(1.0 - SKIN_WIDTH);

    // collide and slide movement pass
    let collision_move_vector = collide_and_slide(
        position,
        move_vel,
        &shape,
        0,
        false,
        move_vel,
        &rapier_context,
    );

    position = integrate(position, collision_move_vector, dt);

    // collide and slide gravity pass
    let collision_gravity_vector = collide_and_slide(
        position,
        gravity_vel,
        &shape,
        0,
        true,
        gravity_vel,
        &rapier_context,
    );

    position = integrate(position, collision_gravity_vector, dt);

    (position, velocity, wish_dir, new_rot)
}

fn perpendicular_unit_vector(n: Vec3, v: Vec3) -> Vec3 {
    let projected = v - n * v.dot(n);
    let len_sq = projected.length_squared();
    if len_sq > 1e-6 {
        projected / len_sq.sqrt()
    } else {
        n.any_orthonormal_vector()
    }
}

pub fn solve_two_bone_ik(
    a: Vec3, // root position (shoulder, hip)
    b: Vec3, // initial mid position (elbow, knee)
    c: Vec3, // initial end position (hand, foot)
    t: Vec3, // target position
) -> (Vec3, Vec3) {
    // returns (b position, c position)
    let len_ab: f32 = (b - a).length();
    let len_bc: f32 = (c - b).length();
    let len_at: f32 = (t - a).length();

    // unit vector from t to a
    let mut n: Vec3 = (a - t).normalize();
    if len_at < 0.001 {
        n = Vec3::new(1.0, 0.0, 0.);
    }

    let mut reached_target: bool = false;
    let b_final: Vec3;
    let c_final: Vec3;
    if len_at < (len_ab - len_bc).abs() {
        // target is too close
        let sign = if len_ab > len_bc { 1.0 } else { -1.0 };
        b_final = a - sign * len_ab * n;
        c_final = b_final + sign * len_bc * n;
    } else if len_at <= len_ab + len_bc {
        // target is reachable
        reached_target = true;
        c_final = t;

        // the final knee position will rest somewhere on a circle that is the intersecting point
        // of two spheres, the sphere of the radius len_ab and the sphere of the radius len_bc

        // cos of angle t - a - b_final
        let cos_theta: f32 = // law of cosines
            (len_bc * len_bc - len_ab * len_ab - len_at * len_at) / (-2. * len_ab * len_at);
        let sin_theta: f32 = (1.0 - cos_theta * cos_theta).sqrt();

        // radius of the cirlce that b_final must lay on
        let r: f32 = len_ab * sin_theta;

        // center of the circle
        let m: Vec3 = a - len_ab * cos_theta * n;

        let u: Vec3 = perpendicular_unit_vector(n, b - m);

        // compute final knee position
        b_final = m + r * u;
    } else {
        // target is too far
        b_final = a - len_ab * n;
        c_final = b_final - len_bc * n;
    }

    (b_final, c_final)
}

pub fn quat_from_to(from: Vec3, to: Vec3) -> Quat {
    let f = from.normalize();
    let t = to.normalize();
    let dot = f.dot(t).clamp(-1.0, 1.0);

    if dot > 0.9999 {
        Quat::IDENTITY
    } else if dot < -0.9999 {
        Quat::from_axis_angle(f.any_orthonormal_vector(), PI)
    } else {
        let axis = f.cross(t).normalize();
        let angle = dot.acos();
        Quat::from_axis_angle(axis, angle)
    }
}

pub fn solve_two_bone_rotation() {}
