//! MPU6050 6-axis IMU driver
#![no_std]

use crate::hal::Twi;

const MPU6050_ADDR: u8 = 0x68;

// MPU6050 registers
const REG_PWR_MGMT_1: u8 = 0x6B;
const REG_SMPLRT_DIV: u8 = 0x19;
const REG_CONFIG: u8 = 0x1A;
const REG_GYRO_CONFIG: u8 = 0x1B;
const REG_ACCEL_CONFIG: u8 = 0x1C;
const REG_ACCEL_XOUT_H: u8 = 0x3B;

/// Accelerometer full-scale range
#[derive(Clone, Copy)]
pub enum AccelScale {
    G2 = 0,  // ±2g
    G4 = 1,  // ±4g
    G8 = 2,  // ±8g
    G16 = 3, // ±16g
}

/// Gyroscope full-scale range
#[derive(Clone, Copy)]
pub enum GyroScale {
    Dps250 = 0,  // ±250°/s
    Dps500 = 1,  // ±500°/s
    Dps1000 = 2, // ±1000°/s
    Dps2000 = 3, // ±2000°/s
}

/// 3-axis sensor data
#[derive(Default, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MPU6050 driver
pub struct Mpu6050 {
    twi: Twi,
    accel_scale: f32,
    gyro_scale: f32,
}

impl Mpu6050 {
    /// Create new MPU6050 instance
    pub fn new(twi: Twi) -> Result<Self, ()> {
        let mut mpu = Self {
            twi,
            accel_scale: 16384.0, // Default ±2g
            gyro_scale: 131.0,    // Default ±250°/s
        };
        
        // Initialize sensor
        mpu.init()?;
        
        Ok(mpu)
    }

    /// Initialize the sensor
    fn init(&mut self) -> Result<(), ()> {
        // Wake up the sensor
        self.write_reg(REG_PWR_MGMT_1, 0x00)?;
        
        // Set sample rate to 1kHz
        self.write_reg(REG_SMPLRT_DIV, 0x07)?;
        
        // Set DLPF to 44Hz (acc) and 42Hz (gyro)
        self.write_reg(REG_CONFIG, 0x03)?;
        
        // Configure ranges
        self.set_accel_scale(AccelScale::G2)?;
        self.set_gyro_scale(GyroScale::Dps250)?;
        
        Ok(())
    }

    /// Set accelerometer full-scale range
    pub fn set_accel_scale(&mut self, scale: AccelScale) -> Result<(), ()> {
        self.write_reg(REG_ACCEL_CONFIG, (scale as u8) << 3)?;
        self.accel_scale = match scale {
            AccelScale::G2 => 16384.0,
            AccelScale::G4 => 8192.0,
            AccelScale::G8 => 4096.0,
            AccelScale::G16 => 2048.0,
        };
        Ok(())
    }

    /// Set gyroscope full-scale range
    pub fn set_gyro_scale(&mut self, scale: GyroScale) -> Result<(), ()> {
        self.write_reg(REG_GYRO_CONFIG, (scale as u8) << 3)?;
        self.gyro_scale = match scale {
            GyroScale::Dps250 => 131.0,
            GyroScale::Dps500 => 65.5,
            GyroScale::Dps1000 => 32.8,
            GyroScale::Dps2000 => 16.4,
        };
        Ok(())
    }

    /// Read raw accelerometer data
    pub fn read_accel(&mut self) -> Result<Vec3, ()> {
        let mut data = [0u8; 6];
        self.read_regs(REG_ACCEL_XOUT_H, &mut data)?;
        
        let raw_x = (data[0] as i16) << 8 | data[1] as i16;
        let raw_y = (data[2] as i16) << 8 | data[3] as i16;
        let raw_z = (data[4] as i16) << 8 | data[5] as i16;
        
        Ok(Vec3 {
            x: raw_x as f32 / self.accel_scale,
            y: raw_y as f32 / self.accel_scale,
            z: raw_z as f32 / self.accel_scale,
        })
    }

    /// Read raw gyroscope data
    pub fn read_gyro(&mut self) -> Result<Vec3, ()> {
        let mut data = [0u8; 6];
        self.read_regs(REG_ACCEL_XOUT_H + 8, &mut data)?;
        
        let raw_x = (data[0] as i16) << 8 | data[1] as i16;
        let raw_y = (data[2] as i16) << 8 | data[3] as i16;
        let raw_z = (data[4] as i16) << 8 | data[5] as i16;
        
        Ok(Vec3 {
            x: raw_x as f32 / self.gyro_scale,
            y: raw_y as f32 / self.gyro_scale,
            z: raw_z as f32 / self.gyro_scale,
        })
    }

    /// Write to register
    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), ()> {
        self.twi.start()?;
        self.twi.write_address(MPU6050_ADDR, false)?;
        self.twi.write_byte(reg)?;
        self.twi.write_byte(val)?;
        self.twi.stop();
        Ok(())
    }

    /// Read multiple registers
    fn read_regs(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), ()> {
        self.twi.start()?;
        self.twi.write_address(MPU6050_ADDR, false)?;
        self.twi.write_byte(reg)?;
        self.twi.start()?;
        self.twi.write_address(MPU6050_ADDR, true)?;
        
        for i in 0..buffer.len() {
            buffer[i] = self.twi.read_byte(i < buffer.len() - 1)?;
        }
        
        self.twi.stop();
        Ok(())
    }
}
