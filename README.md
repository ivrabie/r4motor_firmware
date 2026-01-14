# r4motor_firmware


# SPI protocol
The SPI protocol used by the r4motor firmware is as follows:
Read sequence:
M -> reg_addr (1 byte) | req_len (2 bytes) | crc (2 bytes)
S -> data (req_len bytes) | crc (2 bytes)
Write sequence:
M -> reg_addr (1 byte) | req_len (2 bytes) | data (req_len bytes) | crc (2 bytes)

## Regs
| Reg(2byte) | Description |  Default Value | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "x.x.x" | RO | Current firmware version |
| 0x02 | Control Mode  | 0 | RW | 0: Position control, 1: Velocity control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x05 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x06 | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x07 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 |
| 0x08 | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 |
| 0x09 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 |
| 0x0A | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 |
| 0x0B | Motor1 counts per revolution | 0 | RW | Counts per revolution for Motor1 |
| 0x0C | Motor2 counts per revolution | 0 | RW | Counts per revolution for Motor2 |
| 0x0D | Motor3 counts per revolution | 0 | RW | Counts per revolution for Motor3 |
| 0x0E | Motor4 counts per revolution | 0 | RW | Counts per revolution for Motor4 |
| 0x0F | Motor1 rpm current | 0 | RW | Current rpm for Motor1 |
| 0x10 | Motor2 rpm current | 0 | RW | Current rpm for Motor2 |
| 0x11 | Motor3 rpm current | 0 | RW | Current rpm for Motor3 |
| 0x12 | Motor4 rpm current | 0 | RW | Current rpm for Motor4 |
| 0x13 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 |
| 0x14 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 |
| 0x15 | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 |
| 0x16 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 |
| 0x17 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x18 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x19 | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x1A | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x1B | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x1C | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x1D | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x1E | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1F | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x20 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x21 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x22 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x23 | Internal loop time | 0 | RW | Internal loop time |
| 0x24 | Last error status | 0 | RW | Last error status |

### Errors
| Error code | Description |
|-------------|-------------|
| 0x00 | No error |
| 0x01 | Invalid register address |
| 0x02 | Invalid request length |
| 0x03 | CRC mismatch |
| 0x04 | Invalid control mode |
| 0x05 | Write not allowed in this control mode |

