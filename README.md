# r4motor_firmware

## Dev Container

This repository includes a `.devcontainer` configuration with the embedded Rust
build dependencies preinstalled.

- Open the repo in VS Code and run `Dev Containers: Reopen in Container`.
- The current remote runner setup in `.cargo/config.toml` is kept unchanged.
- The container uses host networking (`--network=host`) and USB passthrough.
- On container start, connectivity checks run for `192.168.1.146:3000` and
  `192.168.1.146:50000` and print warning-only diagnostics.


# SPI protocol

The SPI protocol used by `r4motor_firmware` is:

- Header (master -> slave): `reg_rw (1 byte) | req_len (2 bytes, LE) | header_crc (2 bytes, LE)`
- `reg_rw` encoding:
  - bit 7: operation (`0 = read`, `1 = write`)
  - bits 0..6: register id (`0x00..0x27`)
- Write payload (master -> slave): `data (req_len bytes) | data_crc (2 bytes, LE)`
- Read response (slave -> master): `data (req_len bytes) | data_crc (2 bytes, LE)`

CRC uses `CRC_16_IBM_SDLC`.

## Register map

- Register count: `40` (`0x00..0x27`)
- Register size: `4 bytes` each
- Data encoding for numeric values: little-endian

| Reg (1 byte) | Description | Default value (4 bytes) | Access | Notes |
|-------------|-------------|--------------------------|--------|-------|
| `0x00` | Device ID | `"4MOT"` | RO | Unique device identifier |
| `0x01` | Firmware version | `[0, 0, 0, 1]` | RO | 4-byte version field |
| `0x02` | Motor1 operation mode | `0` | RW | `0: PwmControl`, `1: RpmControl` |
| `0x03` | Motor1 direction | `0` | RW | `0: Stop`, `1: Forward`, `2: Backward`, `3: Brake` |
| `0x04` | Motor1 PWM duty cycle | `0` | RW | Range `0..100` |
| `0x05` | Motor1 counts per revolution | `330` | RW | Range `1..i32::MAX` |
| `0x06` | Motor1 PID Kp | `0` | RW | Range `0..i32::MAX` |
| `0x07` | Motor1 PID Ki | `0` | RW | Range `0..i32::MAX` |
| `0x08` | Motor1 PID Kd | `0` | RW | Range `0..i32::MAX` |
| `0x09` | Motor1 RPM desired | `0` | RW | Range `-200..200` |
| `0x0A` | Motor1 RPM current | `0` | RO | Runtime telemetry |
| `0x0B` | Motor2 operation mode | `0` | RW | `0: PwmControl`, `1: RpmControl` |
| `0x0C` | Motor2 direction | `0` | RW | `0: Stop`, `1: Forward`, `2: Backward`, `3: Brake` |
| `0x0D` | Motor2 PWM duty cycle | `0` | RW | Range `0..100` |
| `0x0E` | Motor2 counts per revolution | `330` | RW | Range `1..i32::MAX` |
| `0x0F` | Motor2 PID Kp | `0` | RW | Range `0..i32::MAX` |
| `0x10` | Motor2 PID Ki | `0` | RW | Range `0..i32::MAX` |
| `0x11` | Motor2 PID Kd | `0` | RW | Range `0..i32::MAX` |
| `0x12` | Motor2 RPM desired | `0` | RW | Range `-200..200` |
| `0x13` | Motor2 RPM current | `0` | RO | Runtime telemetry |
| `0x14` | Motor3 operation mode | `0` | RW | `0: PwmControl`, `1: RpmControl` |
| `0x15` | Motor3 direction | `0` | RW | `0: Stop`, `1: Forward`, `2: Backward`, `3: Brake` |
| `0x16` | Motor3 PWM duty cycle | `0` | RW | Range `0..100` |
| `0x17` | Motor3 counts per revolution | `330` | RW | Range `1..i32::MAX` |
| `0x18` | Motor3 PID Kp | `0` | RW | Range `0..i32::MAX` |
| `0x19` | Motor3 PID Ki | `0` | RW | Range `0..i32::MAX` |
| `0x1A` | Motor3 PID Kd | `0` | RW | Range `0..i32::MAX` |
| `0x1B` | Motor3 RPM desired | `0` | RW | Range `-200..200` |
| `0x1C` | Motor3 RPM current | `0` | RO | Runtime telemetry |
| `0x1D` | Motor4 operation mode | `0` | RW | `0: PwmControl`, `1: RpmControl` |
| `0x1E` | Motor4 direction | `0` | RW | `0: Stop`, `1: Forward`, `2: Backward`, `3: Brake` |
| `0x1F` | Motor4 PWM duty cycle | `0` | RW | Range `0..100` |
| `0x20` | Motor4 counts per revolution | `330` | RW | Range `1..i32::MAX` |
| `0x21` | Motor4 PID Kp | `0` | RW | Range `0..i32::MAX` |
| `0x22` | Motor4 PID Ki | `0` | RW | Range `0..i32::MAX` |
| `0x23` | Motor4 PID Kd | `0` | RW | Range `0..i32::MAX` |
| `0x24` | Motor4 RPM desired | `0` | RW | Range `-200..200` |
| `0x25` | Motor4 RPM current | `0` | RO | Runtime telemetry |
| `0x26` | Internal loop time | `10` | RW | Range `1..10000` ms |
| `0x27` | Last error status | `0` | RO | Clears to `NoError` when read |

Note: when a motor is in `RpmControl`, that motor's PWM duty and direction registers become read-only.

## Error codes

| Error code | Name | Description |
|------------|------|-------------|
| `0x00` | `NoError` | No error |
| `0x01` | `InvalidRegisterAddress` | Register id out of range |
| `0x02` | `InvalidRequestLength` | Invalid packet or data length |
| `0x03` | `InvalidRegisterValueRange` | Written value out of allowed range |
| `0x04` | `NowAllowedReadOnly` | Attempted write to read-only register |
| `0x05` | `InvalidControlMode` | Unsupported control mode value |
| `0x06` | `WriteNotAllowedInThisControlMode` | Write rejected for current control mode |
| `0x07` | `CrcValidationFailed` | Header or payload CRC check failed |
