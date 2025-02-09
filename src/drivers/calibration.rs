//! Sensor calibration routines
#![no_std]

use crate::drivers::{Vec3, Mpu6050};
use crate::hal::flash::Flash;

const CALIBRATION_SAMPLES: usize = 1000;
const FLASH_SECTOR_CALIBRATION: u32 = 0x10000;

pub struct CalibrationData {
    accel_offset: Vec3,
    accel_scale: Vec3,
    gyro_offset: Vec3,
    gyro_scale: Vec3,
    mag_offset: Vec3,
    mag_scale: Vec3,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            accel_offset: Vec3::default(),
            accel_scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            gyro_offset: Vec3::default(),
            gyro_scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            mag_offset: Vec3::default(),
            mag_scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}

/*
struct CalibrationConfig {
    samples_per_point: u16,
    settling_time_ms: u16,
    max_deviation: f32,
    temperature_comp: bool,
}

struct CalibrationStats {
    min_values: Vec3,
    max_values: Vec3,
    mean_values: Vec3,
    std_dev: Vec3,
}
*/

pub struct Calibration {
    data: CalibrationData,
    flash: Flash,
}

impl Calibration {
    pub fn new(flash: Flash) -> Self {
        Self {
            data: CalibrationData::default(),
            flash,
        }
    }

    pub fn calibrate_gyro(&mut self, imu: &mut Mpu6050) -> Result<(), ()> {
        let mut sum = Vec3::default();
        
        for _ in 0..CALIBRATION_SAMPLES {
            if let Ok(gyro) = imu.read_gyro() {
                sum.x += gyro.x;
                sum.y += gyro.y;
                sum.z += gyro.z;
            }
        }
        
        self.data.gyro_offset = Vec3 {
            x: sum.x / CALIBRATION_SAMPLES as f32,
            y: sum.y / CALIBRATION_SAMPLES as f32,
            z: sum.z / CALIBRATION_SAMPLES as f32,
        };
        
        Ok(())
    }

    pub fn calibrate_accel(&mut self, imu: &mut Mpu6050) -> Result<(), ()> {
        let mut min = Vec3 { x: f32::MAX, y: f32::MAX, z: f32::MAX };
        let mut max = Vec3 { x: f32::MIN, y: f32::MIN, z: f32::MIN };
        
        for _ in 0..CALIBRATION_SAMPLES {
            if let Ok(accel) = imu.read_accel() {
                min.x = min.x.min(accel.x);
                min.y = min.y.min(accel.y);
                min.z = min.z.min(accel.z);
                
                max.x = max.x.max(accel.x);
                max.y = max.y.max(accel.y);
                max.z = max.z.max(accel.z);
            }
        }
        
        self.data.accel_offset = Vec3 {
            x: (min.x + max.x) / 2.0,
            y: (min.y + max.y) / 2.0,
            z: (min.z + max.z) / 2.0,
        };
        
        self.data.accel_scale = Vec3 {
            x: 2.0 / (max.x - min.x),
            y: 2.0 / (max.y - min.y),
            z: 2.0 / (max.z - min.z),
        };
        
        Ok(())
    }

    pub fn apply_gyro_calibration(&self, raw: Vec3) -> Vec3 {
        Vec3 {
            x: (raw.x - self.data.gyro_offset.x) * self.data.gyro_scale.x,
            y: (raw.y - self.data.gyro_offset.y) * self.data.gyro_scale.y,
            z: (raw.z - self.data.gyro_offset.z) * self.data.gyro_scale.z,
        }
    }

    pub fn apply_accel_calibration(&self, raw: Vec3) -> Vec3 {
        Vec3 {
            x: (raw.x - self.data.accel_offset.x) * self.data.accel_scale.x,
            y: (raw.y - self.data.accel_offset.y) * self.data.accel_scale.y,
            z: (raw.z - self.data.accel_offset.z) * self.data.accel_scale.z,
        }
    }

    pub fn save_calibration(&mut self) -> Result<(), ()> {
        let data = unsafe {
            core::slice::from_raw_parts(
                (&self.data as *const CalibrationData) as *const u8,
                core::mem::size_of::<CalibrationData>(),
            )
        };
        
        self.flash.erase_sector(FLASH_SECTOR_CALIBRATION)?;
        self.flash.write(FLASH_SECTOR_CALIBRATION, data)?;
        
        Ok(())
    }

    pub fn load_calibration(&mut self) -> Result<(), ()> {
        let mut buffer = [0u8; core::mem::size_of::<CalibrationData>()];
        self.flash.read(FLASH_SECTOR_CALIBRATION, &mut buffer)?;
        
        self.data = unsafe {
            core::ptr::read(buffer.as_ptr() as *const CalibrationData)
        };
        
        Ok(())
    }

    pub fn reset_calibration(&mut self) {
        self.data = CalibrationData::default();
    }
}
