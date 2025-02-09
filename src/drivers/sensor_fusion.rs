//! Sensor fusion implementation using Madgwick filter
//! 
//! This implementation is based on Sebastian Madgwick's paper:
//! "An efficient orientation filter for inertial and inertial/magnetic sensor arrays"
//! 

#![no_std]

use core::f32::consts::PI;
use libm::{sqrtf, atan2f};
use crate::drivers::Vec3;

// Filter parameters - these were tuned through extensive testing
// TODO: Make these configurable through a builder pattern
const BETA: f32 = 0.1;  // Filter gain
const ZETA: f32 = 0.015;  // Gyro drift bias gain

/// Quaternion for 3D rotation representation
#[derive(Clone, Copy)]
pub struct Quaternion {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Quaternion {
    pub fn new() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn normalize(&mut self) {
        let norm = sqrtf(
            self.w * self.w +
            self.x * self.x +
            self.y * self.y +
            self.z * self.z
        );
        if norm > 0.0 {
            self.w /= norm;
            self.x /= norm;
            self.y /= norm;
            self.z /= norm;
        }
    }
}

/// Sensor fusion filter using Madgwick algorithm
pub struct MadgwickFilter {
    q: Quaternion,
    beta: f32,
    zeta: f32,
    gyro_bias: Vec3,
    sample_freq: f32,
    
    // Performance stats for debugging
    update_count: u32,
    max_update_time_us: u32,
    
    /*
    #[allow(dead_code)]
    adaptive_beta: bool,
    #[allow(dead_code)]
    min_beta: f32,
    #[allow(dead_code)]
    max_beta: f32,
    
    // Additional sensor fusion modes we might add later
    #[allow(dead_code)]
    fusion_modes: [bool; 4] = [
        true,   // Use accelerometer
        true,   // Use gyroscope
        false,  // Use magnetometer
        false   // Use barometer
    ];
    */
}

impl MadgwickFilter {
    pub fn new(sample_freq: f32) -> Self {
        Self {
            q: Quaternion::new(),
            beta: BETA,
            zeta: ZETA,
            gyro_bias: Vec3::default(),
            sample_freq,
            update_count: 0,
            max_update_time_us: 0,
        }
    }

    /// Update filter with new sensor readings
    pub fn update(&mut self, accel: Vec3, gyro: Vec3) {
        // Start timing the update for performance monitoring
        let start_time = get_micros();
        
        // Remove gyro bias
        let gyro = Vec3 {
            x: gyro.x - self.gyro_bias.x,
            y: gyro.y - self.gyro_bias.y,
            z: gyro.z - self.gyro_bias.z,
        };

        // Convert gyro readings from degrees to radians
        let gyro = Vec3 {
            x: gyro.x * PI / 180.0,
            y: gyro.y * PI / 180.0,
            z: gyro.z * PI / 180.0,
        };

        // Normalize accelerometer measurement
        let accel_norm = sqrtf(accel.x * accel.x + accel.y * accel.y + accel.z * accel.z);
        if accel_norm == 0.0 {
            return; // Handle NaN
        }
        let accel = Vec3 {
            x: accel.x / accel_norm,
            y: accel.y / accel_norm,
            z: accel.z / accel_norm,
        };

        // Gradient descent algorithm corrective step
        let qw = self.q.w;
        let qx = self.q.x;
        let qy = self.q.y;
        let qz = self.q.z;

        // Auxiliary variables to avoid repeated calculations
        let _2qw = 2.0 * qw;
        let _2qx = 2.0 * qx;
        let _2qy = 2.0 * qy;
        let _2qz = 2.0 * qz;
        let _4qw = 4.0 * qw;
        let _4qx = 4.0 * qx;
        let _4qy = 4.0 * qy;
        let _8qx = 8.0 * qx;
        let _8qy = 8.0 * qy;
        let q0q0 = qw * qw;
        let q1q1 = qx * qx;
        let q2q2 = qy * qy;
        let q3q3 = qz * qz;

        // Gradient decent algorithm corrective step
        let s0 = _4qw * q2q2 + _2qy * accel.x + _4qw * q1q1 - _2qx * accel.y;
        let s1 = _4qx * q3q3 - _2qz * accel.x + 4.0 * q0q0 * qx - _2qw * accel.y - _4qx + _8qx * q1q1 + _8qx * q2q2 + _4qx * accel.z;
        let s2 = 4.0 * q0q0 * qy + _2qw * accel.x + _4qy * q3q3 - _2qz * accel.y - _4qy + _8qy * q1q1 + _8qy * q2q2 + _4qy * accel.z;
        let s3 = 4.0 * q1q1 * qz - _2qx * accel.x + 4.0 * q2q2 * qz - _2qy * accel.y;
        let norm = sqrtf(s0 * s0 + s1 * s1 + s2 * s2 + s3 * s3);
        if norm == 0.0 {
            return; // Handle NaN
        }
        let s0 = s0 / norm;
        let s1 = s1 / norm;
        let s2 = s2 / norm;
        let s3 = s3 / norm;

        // Rate of change of quaternion from gyroscope
        let qDot1 = 0.5 * (-qx * gyro.x - qy * gyro.y - qz * gyro.z);
        let qDot2 = 0.5 * (qw * gyro.x + qy * gyro.z - qz * gyro.y);
        let qDot3 = 0.5 * (qw * gyro.y - qx * gyro.z + qz * gyro.x);
        let qDot4 = 0.5 * (qw * gyro.z + qx * gyro.y - qy * gyro.x);

        // Compute and integrate rate of change of quaternion
        let dt = 1.0 / self.sample_freq;
        self.q.w += (qDot1 - self.beta * s0) * dt;
        self.q.x += (qDot2 - self.beta * s1) * dt;
        self.q.y += (qDot3 - self.beta * s2) * dt;
        self.q.z += (qDot4 - self.beta * s3) * dt;

        // Normalize quaternion
        self.q.normalize();

        // Update performance stats
        self.update_count += 1;
        let update_time = get_micros() - start_time;
        if update_time > self.max_update_time_us {
            self.max_update_time_us = update_time;
        }
    }

    /// Get Euler angles (roll, pitch, yaw) in degrees
    pub fn get_euler_angles(&self) -> Vec3 {
        let qw = self.q.w;
        let qx = self.q.x;
        let qy = self.q.y;
        let qz = self.q.z;

        let roll = atan2f(2.0 * (qw * qx + qy * qz), 1.0 - 2.0 * (qx * qx + qy * qy)) * 180.0 / PI;
        let pitch = (2.0 * (qw * qy - qz * qx)).asin() * 180.0 / PI;
        let yaw = atan2f(2.0 * (qw * qz + qx * qy), 1.0 - 2.0 * (qy * qy + qz * qz)) * 180.0 / PI;

        Vec3 {
            x: roll,
            y: pitch,
            z: yaw,
        }
    }

    /* Keeping this code commented out for future reference
    /// Experimental: Adaptive filter gain based on motion intensity
    #[allow(dead_code)]
    fn update_adaptive_gain(&mut self, accel: Vec3, gyro: Vec3) {
        let accel_magnitude = sqrtf(
            accel.x * accel.x +
            accel.y * accel.y +
            accel.z * accel.z
        );
        
        let gyro_magnitude = sqrtf(
            gyro.x * gyro.x +
            gyro.y * gyro.y +
            gyro.z * gyro.z
        );
        
        // Increase beta during high motion
        if gyro_magnitude > 100.0 || (accel_magnitude > 1.2 || accel_magnitude < 0.8) {
            self.beta = self.max_beta;
        } else {
            self.beta = self.min_beta;
        }
    }
    */
}

// Helper function to get microsecond timestamp
// TODO: Replace this with proper timer implementation
fn get_micros() -> u32 {
    // This is just a placeholder - we should use a hardware timer
    0
}
