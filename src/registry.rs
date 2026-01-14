/*
 ## Regs
| Reg(1byte) | Description |  Default Value (4bytes) | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "x.x.x" | RO | Current firmware version |
| 0x02 | Motor1 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 |
| 0x05 | Motor1 counts per revolution | 0 | RW | Counts per revolution for Motor1 |
| 0x06 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x07 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x08 | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x09 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 |
| 0x0A | Motor1 rpm current | 0 | R | Current rpm for Motor1 |
| 0x0B | Motor2 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x0C | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x0D | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 |
| 0x0E | Motor2 counts per revolution | 0 | RW | Counts per revolution for Motor2 |
| 0x0F | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x10 | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x11 | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x12 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 |
| 0x13 | Motor2 rpm current | 0 | R | Current rpm for Motor2 |
| 0x14 | Motor3 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x15 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x16 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 |
| 0x17 | Motor3 counts per revolution | 0 | RW | Counts per revolution for Motor3 |
| 0x18 | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x19 | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1A | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x1B | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 |
| 0x1C | Motor3 rpm current | 0 | R | Current rpm for Motor3 |
| 0x1D | Motor4 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x1E | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x1F | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 |
| 0x20 | Motor4 counts per revolution | 0 | RW | Counts per revolution for Motor4 |
| 0x21 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x22 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x23 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x24 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 |
| 0x25 | Motor4 rpm current | 0 | R | Current rpm for Motor4 |
| 0x26 | Internal loop time | 0 | RW | Internal loop time |
| 0x27 | Last error status | 0 | R | Last error status 1 byte | reg 1 byte | 2 reserved |
# Errors
| Error code | Description |
|-------------|-------------|
| 0x00 | No error |
| 0x01 | Invalid register address |
| 0x02 | Invalid request length |
| 0x03 | CRC mismatch |
| 0x04 | Invalid control mode |
| 0x05 | Write not allowed in this control mode |*/

use crate::spi_proto;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use defmt::{Format, info};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum RegisterID {
    DeviceID = 0x00,
    FirmwareVersion = 0x01,
    Motor1OperationMode = 0x02,
    Motor1Direction = 0x03,
    Motor1PWMDutyCycle = 0x04,
    Motor1CountsPerRevolution = 0x05,
    Motor1PIDKp = 0x06,
    Motor1PIDKi = 0x07,
    Motor1PIDKd = 0x08,
    Motor1RPMDesired = 0x09,
    Motor1RPMCurrent = 0x0A,
    Motor2OperationMode = 0x0B,
    Motor2Direction = 0x0C,
    Motor2PWMDutyCycle = 0x0D,
    Motor2CountsPerRevolution = 0x0E,
    Motor2PIDKp = 0x0F,
    Motor2PIDKi = 0x10,
    Motor2PIDKd = 0x11,
    Motor2RPMDesired = 0x12,
    Motor2RPMCurrent = 0x13,
    Motor3OperationMode = 0x14,
    Motor3Direction = 0x15,
    Motor3PWMDutyCycle = 0x16,
    Motor3CountsPerRevolution = 0x17,
    Motor3PIDKp = 0x18,
    Motor3PIDKi = 0x19,
    Motor3PIDKd = 0x1A,
    Motor3RPMDesired = 0x1B,
    Motor3RPMCurrent = 0x1C,
    Motor4OperationMode = 0x1D,
    Motor4Direction = 0x1E,
    Motor4PWMDutyCycle = 0x1F,
    Motor4CountsPerRevolution = 0x20,
    Motor4PIDKp = 0x21,
    Motor4PIDKi = 0x22,
    Motor4PIDKd = 0x23,
    Motor4RPMDesired = 0x24,
    Motor4RPMCurrent = 0x25,
    InternalLoopTime = 0x26,
    LastErrorStatus = 0x27,
}
#[repr(u32)]
#[derive(Debug, Clone, Copy, Format, PartialEq, Eq, TryFromPrimitive)]
pub enum ErrorCode {
    NoError = 0x00,
    InvalidRegisterAddress = 0x01,
    InvalidRequestLength = 0x02,
    CRCMismatch = 0x03,
    InvalidControlMode = 0x04,
    WriteNotAllowedInThisControlMode = 0x05,
}

#[repr(C)]
#[derive(Copy, Clone, Format)]
struct DeviceInfo {
    device_id: [u8; 4],
    firmware_version: [u8; 4],
}

#[derive(Copy, Clone, Format)]
#[repr(C)]
struct MotorConfig {
    control_mode: u32,
    direction: u32,
    pwm_duty_cycle: u32,
    counts_per_revolution: u32,
    pid_kp: u32,
    pid_ki: u32,
    pid_kd: u32,
    rpm_desired: u32,
    rpm_current: u32,
}
#[derive(Copy, Clone, Format)]
#[repr(C)]
pub struct RegistryData {
    device_info: DeviceInfo,
    motors: [MotorConfig; 4],
    internal_loop_time: u32,
    last_error_status: ErrorCode,
}
#[repr(C)]
pub union Registry {
    registry: RegistryData,
    bytes: [u8; core::mem::size_of::<RegistryData>()],
}


impl RegistryData {
    pub fn new() -> Self {
        RegistryData {
            device_info: DeviceInfo {
                device_id: *b"4MOT",
                firmware_version: [0, 0, 0, 1],
            },
            motors: [MotorConfig {
                control_mode: 0,
                direction: 0,
                pwm_duty_cycle: 0,
                counts_per_revolution: 1,
                pid_kp: 0,
                pid_ki: 0,
                pid_kd: 0,
                rpm_desired: 0,
                rpm_current: 0,
            }; 4],
            internal_loop_time: 10,
            last_error_status: ErrorCode::NoError,
        }
    }
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            registry: RegistryData::new(),
        }
    }

    pub fn update_registry(&mut self, reg_id: RegisterID, data: &[u8]) {
        let u32_size = core::mem::size_of::<u32>();
        let reg_offset = reg_id as usize * u32_size;
        let end_offset = RegisterID::LastErrorStatus as usize * u32_size + u32_size;
        assert!(reg_offset + data.len() <= end_offset,
                "Data write exceeds registry bounds");
        //ToDo Add data validation based on reg_id
        let registry_bytes = unsafe {&mut self.bytes};
        registry_bytes[reg_offset..reg_offset + data.len()].copy_from_slice(data);
        info!("Updated buffer {}", registry_bytes);
    }


    pub fn read_registry(&self, reg_id: RegisterID, data: &mut [u8]) {
        let u32_size = core::mem::size_of::<u32>();
        let reg_offset = reg_id as usize * u32_size;
        let end_offset = RegisterID::LastErrorStatus as usize * u32_size + u32_size;
        assert!(reg_offset + data.len() <= end_offset,
                "Data read exceeds registry bounds");
        let registry_bytes = unsafe {&self.bytes};
        info!("Reg offset {} and reg offset end {}", reg_offset, reg_offset + data.len());
        data.copy_from_slice(&registry_bytes[reg_offset..reg_offset + data.len()]);
    }

    pub fn get_registry_data(&self) -> &RegistryData {
        unsafe { &self.registry }
    }
}