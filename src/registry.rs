/*
 ## Regs
| Reg(1byte) | Description |  Default Value (4bytes) | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "0.0.0.1" | RO | Current firmware version |
| 0x02 | Motor1 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 (0-100) |
| 0x05 | Motor1 counts per revolution | 1 | RW | Counts per revolution for Motor1 |
| 0x06 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x07 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x08 | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x09 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 (0-200) |
| 0x0A | Motor1 rpm current | 0 | R | Current rpm for Motor1 |
| 0x0B | Motor2 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x0C | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x0D | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 (0-100) |
| 0x0E | Motor2 counts per revolution | 1 | RW | Counts per revolution for Motor2 |
| 0x0F | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x10 | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x11 | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x12 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 (0-200) |
| 0x13 | Motor2 rpm current | 0 | R | Current rpm for Motor2 |
| 0x14 | Motor3 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x15 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x16 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 (0-100) |
| 0x17 | Motor3 counts per revolution | 1 | RW | Counts per revolution for Motor3 |
| 0x18 | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x19 | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1A | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x1B | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 (0-200) |
| 0x1C | Motor3 rpm current | 0 | R | Current rpm for Motor3 |
| 0x1D | Motor4 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x1E | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x1F | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 (0-100) |
| 0x20 | Motor4 counts per revolution | 1 | RW | Counts per revolution for Motor4 |
| 0x21 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x22 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x23 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x24 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 (0-200) |
| 0x25 | Motor4 rpm current | 0 | R | Current rpm for Motor4 |
| 0x26 | Internal loop time | 10 | RW | Internal loop time (1-10000) |
| 0x27 | Last error status | 0 | R | Last error status --> clears on read |

# Errors
| Error code | Description |
|-------------|-------------|
| 0x00 | No error |
| 0x01 | Invalid register address |
| 0x02 | Invalid request length |
| 0x03 | Invalid register value range |
| 0x04 | Not allowed read only |
| 0x05 | Invalid control mode |
| 0x06 | Write not allowed in this control mode |
| 0x07 | CRC validation failed |

# Notes
- PWM duty cycle becomes read-only when motor is in RPM control mode
- Error status register clears to NoError when read
*/

use defmt::trace;
use defmt::{Format, info};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::car_ctrl;
use crate::spi_proto;
macro_rules! copy_motor_configs {
    ($configs:expr, $start_idx:expr, $motor_cfg:expr) => {
        $configs[$start_idx + 0] = $motor_cfg[0]; // Control Mode
        $configs[$start_idx + 1] = $motor_cfg[1]; // Direction
        $configs[$start_idx + 2] = $motor_cfg[2]; // PWM duty cycle
        $configs[$start_idx + 3] = $motor_cfg[3]; // Counts per revolution
        $configs[$start_idx + 4] = $motor_cfg[4]; // PID Kp
        $configs[$start_idx + 5] = $motor_cfg[5]; // PID Ki
        $configs[$start_idx + 6] = $motor_cfg[6]; // PID Kd
        $configs[$start_idx + 7] = $motor_cfg[7]; // RPM desired
        $configs[$start_idx + 8] = $motor_cfg[8]; // RPM current
    };
}
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Format, TryFromPrimitive)]
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
    InvalidRegisterValueRange = 0x03,
    NowAllowedReadOnly = 0x04,
    InvalidControlMode = 0x05,
    WriteNotAllowedInThisControlMode = 0x06,
    CrcValidationFailed = 0x07,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Format, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum ControlMode {
    PwmControl = 0,
    RpmControl = 1,
}
#[repr(u32)]
#[derive(Debug, Clone, Copy, Format, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Direction {
    Stop = 0,
    Forward = 1,
    Backward = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Format)]
pub struct DeviceInfo {
    device_id: [u8; 4],
    firmware_version: [u8; 4],
}

#[derive(Copy, Clone, Format)]
#[repr(C)]
pub struct MotorConfig {
    pub control_mode: ControlMode,
    pub direction: Direction,
    pub pwm_duty_cycle: i32,
    pub counts_per_revolution: i32,
    pub pid_kp: i32,
    pub pid_ki: i32,
    pub pid_kd: i32,
    pub rpm_desired: i32,
    pub rpm_current: i32,
}
#[derive(Copy, Clone, Format)]
#[repr(C)]
pub struct RegistryData {
    pub device_info: DeviceInfo,
    pub motors: [MotorConfig; 4],
    pub internal_loop_time: u32,
    pub last_error_status: ErrorCode,
}
#[repr(C)]
pub union RegistryUnion {
    pub registry: RegistryData,
    pub bytes: [u8; core::mem::size_of::<RegistryData>()],
}

#[derive(Copy, Clone, Debug)]
pub enum RegisterAccessType {
    ReadOnly,
    ReadWrite,
}

#[derive(Copy, Clone, Debug)]
pub struct RegisterConfig {
    pub access: RegisterAccessType,
    pub low_range: i32,
    pub high_range: i32,
}
pub struct Registry {
    data: RegistryUnion,
    register_configs: [RegisterConfig; 40],
}

pub const MOTOR_MAX_PWM_DUTY_CYCLE: i32 = 100;
pub const MOTOR_MAX_DESIRED_RPM: i32 = 200;
pub const MOTOR_DEFAULT_REV_COUNT: i32 = 330;

pub const MOTOR_REGISTER_CONFIGS: [RegisterConfig; 9] = [
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: ControlMode::PwmControl as i32,
        high_range: ControlMode::RpmControl as i32,
    }, // Control Mode
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: Direction::Stop as i32,
        high_range: Direction::Backward as i32,
    }, // direction
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 0,
        high_range: MOTOR_MAX_PWM_DUTY_CYCLE,
    }, // pwm duty cycle
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 1,
        high_range: i32::MAX,
    }, // counts per revolution
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 0,
        high_range: i32::MAX,
    }, // pid kp
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 0,
        high_range: i32::MAX,
    }, // pid ki
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 0,
        high_range: i32::MAX,
    }, // pid kd
    RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: -MOTOR_MAX_DESIRED_RPM,
        high_range: MOTOR_MAX_DESIRED_RPM,
    }, // rpm desired
    RegisterConfig {
        access: RegisterAccessType::ReadOnly,
        low_range: i32::MIN,
        high_range: i32::MAX,
    }, // rpm current
];

impl RegistryData {
    pub const fn new() -> Self {
        RegistryData {
            device_info: DeviceInfo {
                device_id: *b"4MOT",
                firmware_version: [0, 0, 0, 1],
            },
            motors: [MotorConfig {
                control_mode: ControlMode::PwmControl,
                direction: Direction::Stop,
                pwm_duty_cycle: 0,
                counts_per_revolution: MOTOR_DEFAULT_REV_COUNT,
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
const fn create_register_config_with_macro() -> [RegisterConfig; 40] {
    let mut configs = [RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 0,
        high_range: 0,
    }; 40];

    // Device registers
    configs[RegisterID::DeviceID as usize] = RegisterConfig {
        access: RegisterAccessType::ReadOnly,
        low_range: 0,
        high_range: 0,
    };
    configs[RegisterID::FirmwareVersion as usize] = RegisterConfig {
        access: RegisterAccessType::ReadOnly,
        low_range: 0,
        high_range: 0,
    };

    // Copy motor configs for all 4 motors
    copy_motor_configs!(
        configs,
        RegisterID::Motor1OperationMode as usize,
        MOTOR_REGISTER_CONFIGS
    ); // Motor 1
    copy_motor_configs!(
        configs,
        RegisterID::Motor2OperationMode as usize,
        MOTOR_REGISTER_CONFIGS
    ); // Motor 2
    copy_motor_configs!(
        configs,
        RegisterID::Motor3OperationMode as usize,
        MOTOR_REGISTER_CONFIGS
    ); // Motor 3
    copy_motor_configs!(
        configs,
        RegisterID::Motor4OperationMode as usize,
        MOTOR_REGISTER_CONFIGS
    ); // Motor 4

    // System registers
    configs[RegisterID::InternalLoopTime as usize] = RegisterConfig {
        access: RegisterAccessType::ReadWrite,
        low_range: 1,
        high_range: 10000,
    };
    configs[RegisterID::LastErrorStatus as usize] = RegisterConfig {
        access: RegisterAccessType::ReadOnly,
        low_range: ErrorCode::NoError as i32,
        high_range: ErrorCode::WriteNotAllowedInThisControlMode as i32,
    };

    configs
}
impl Registry {
    pub const fn new() -> Self {
        Registry {
            data: RegistryUnion {
                registry: RegistryData::new(),
            },
            register_configs: create_register_config_with_macro(),
        }
    }

    fn validate_register_write(&self, reg_id: RegisterID, value: i32) -> Result<(), ErrorCode> {
        let config = &self.register_configs[reg_id as usize];
        match config.access {
            RegisterAccessType::ReadOnly => Err(ErrorCode::NowAllowedReadOnly),
            RegisterAccessType::ReadWrite => {
                if value < config.low_range || value > config.high_range {
                    Err(ErrorCode::InvalidRegisterValueRange)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn update_registry(&mut self, reg_id: RegisterID, data: &[u8]) {
        let u32_size = core::mem::size_of::<u32>();
        let reg_offset = reg_id as usize * u32_size;
        let end_offset = RegisterID::LastErrorStatus as usize * u32_size + u32_size;
        assert!(
            reg_offset + data.len() <= end_offset,
            "Data write exceeds registry bounds"
        );
        if data.len() % 4 != 0 {
            self.update_error_status(ErrorCode::InvalidRequestLength);
            return;
        }
        for (i, chunk) in data.chunks(4).enumerate() {
            let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let current_reg_id = RegisterID::try_from((reg_id as usize + i) as u8);
            let current_reg_id = match current_reg_id {
                Ok(id) => id,
                Err(_) => {
                    self.update_error_status(ErrorCode::InvalidRegisterAddress);
                    return;
                }
            };
            if let Err(err) = self.validate_register_write(current_reg_id, value) {
                self.update_error_status(err);
                return;
            }
        }
        let registry_bytes = unsafe { &mut self.data.bytes };
        registry_bytes[reg_offset..reg_offset + data.len()].copy_from_slice(data);
        trace!("Updated buffer {:?}", registry_bytes);
        for i in 0..spi_proto::DEVICE_SUPPORTED_MOTORS {
            let motor_cfg = unsafe { self.data.registry.motors[i] };
            let reg_pwm_idx = (RegisterID::Motor1PWMDutyCycle as usize
                + i * spi_proto::DEVICE_MOTOR_BLOCK_COUNT as usize)
                as u8;
            let reg_dir_idx = (RegisterID::Motor1Direction as usize
                + i * spi_proto::DEVICE_MOTOR_BLOCK_COUNT as usize)
                as u8;
            if motor_cfg.control_mode == ControlMode::RpmControl {
                self.register_configs[reg_pwm_idx as usize].access = RegisterAccessType::ReadOnly;
                self.register_configs[reg_dir_idx as usize].access = RegisterAccessType::ReadOnly;
            }
            if motor_cfg.control_mode == ControlMode::PwmControl {
                self.register_configs[reg_pwm_idx as usize].access = RegisterAccessType::ReadWrite;
                self.register_configs[reg_dir_idx as usize].access = RegisterAccessType::ReadWrite;
            }
            trace!("Motor {} config: {:?}", i + 1, motor_cfg);
        }
    }

    pub fn read_registry(&mut self, reg_id: RegisterID, data: &mut [u8]) {
        let u32_size = core::mem::size_of::<u32>();
        let reg_offset = reg_id as usize * u32_size;
        let end_offset = RegisterID::LastErrorStatus as usize * u32_size + u32_size;
        assert!(
            reg_offset + data.len() <= end_offset,
            "Data read exceeds registry bounds"
        );
        assert!(
            data.len() % 4 == 0,
            "Data length must be multiple of 4 bytes"
        );
        let registry_bytes = unsafe { &self.data.bytes };
        trace!(
            "Reg offset {} and reg offset end {}",
            reg_offset,
            reg_offset + data.len()
        );
        data.copy_from_slice(&registry_bytes[reg_offset..reg_offset + data.len()]);
        let no_of_regs = data.len() / 4;
        let last_reg_id = RegisterID::try_from((reg_id as usize + no_of_regs - 1) as u8).unwrap();
        if last_reg_id >= RegisterID::LastErrorStatus {
            self.update_error_status(ErrorCode::NoError);
        }
    }

    pub fn update_car_state(&mut self, car_state: &car_ctrl::CarCurrState) {
        for (i, motor_state) in car_state.motors.iter().enumerate() {
            let motor_cfg = &mut unsafe { &mut self.data.registry.motors[i] };
            motor_cfg.rpm_current = motor_state.rpm;
            if motor_cfg.control_mode == ControlMode::RpmControl {
                motor_cfg.pwm_duty_cycle = motor_state.pwm_duty;
                motor_cfg.direction = motor_state.direction;
            }
        }
    }

    pub fn update_error_status(&mut self, error: ErrorCode) {
        self.data.registry.last_error_status = error;
    }

    pub fn get_registry_data(&self) -> RegistryData {
        unsafe { self.data.registry }
    }
}
