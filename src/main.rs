#![no_std]
#![no_main]
#![warn(dead_code)]

#[cfg(all(feature = "encoder", feature = "ext_pin_clk"))]
compile_error!("Features `encoder` and `ext_pin_clk` are mutually exclusive");

#[cfg(not(any(feature = "encoder", feature = "ext_pin_clk")))]
compile_error!("Either `encoder` or `ext_pin_clk` feature must be enabled");

use defmt::*;
use embassy_executor::Spawner;
use heapless::Vec;

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

#[cfg(feature = "encoder")]
use embassy_stm32::timer::qei::{Config as QeiConfig, Qei};
#[cfg(feature = "ext_pin_clk")]
use embassy_stm32::{gpio::Flex, timer::low_level::Timer as ExtClkTimer};
use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed},
    spi::{Config as SpiConfig, Spi},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};

use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

mod car_ctrl;
use car_ctrl::*;
mod spi_proto;
use spi_proto::*;
mod registry;

static REGISTRY: Mutex<ThreadModeRawMutex, registry::Registry> =
    Mutex::new(registry::Registry::new());
const MOTOR_COUNT_PER_REV: u32 = 330;

#[embassy_executor::task]
async fn motor_control_task(mut car_ctrl: Car<'static>) {
    let default_motor_state = MotorCurrState {
        rpm: 0,
        direction: registry::Direction::Stop,
        pwm_duty: 0,
    };
    let motors: Vec<MotorCurrState, 4> = Vec::from_slice(&[default_motor_state; 4]).unwrap();
    let mut prev_car_state = CarCurrState { motors };
    car_ctrl.init();
    loop {
        // match select(ticker.next(), REGISTRY_SIGNAL.wait()).await {
        //     Either::First(_) => {
        //         car_ctrl.ctrl_loop();
        //         // CAR_CURR_STATE_SIGNAL.signal(car_ctrl.get_curr_state());
        //     }
        //     Either::Second(cfg) => {
        //         car_ctrl.apply_cfg(&cfg);
        //     }
        // }
        let reg_data = {
            let registry = REGISTRY.lock().await;
            registry.get_registry_data()
        };

        let requested_loop_time_ms = reg_data.internal_loop_time;
        let loop_time_ms = registry::sanitize_internal_loop_time_ms(requested_loop_time_ms);
        if loop_time_ms != requested_loop_time_ms {
            error!(
                "Invalid InternalLoopTime {}; fallback to {} ms",
                requested_loop_time_ms, loop_time_ms
            );
        }

        car_ctrl.apply_cfg(&reg_data);
        car_ctrl.ctrl_loop();
        let car_curr_status = car_ctrl.get_curr_state();
        if car_curr_status != prev_car_state {
            let mut registry = REGISTRY.lock().await;
            registry.update_car_state(&car_curr_status);
            prev_car_state = car_curr_status.clone();
        }
        Timer::after(Duration::from_millis(u64::from(loop_time_ms))).await;
    }
}

#[embassy_executor::task]
async fn spi_task(
    mut spi: Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>,
) {
    let mut protocol_buffer = [0u8; spi_proto::PROTOCOL_MAX_BUFFER_SIZE];
    loop {
        // match select(
        //     handle_spi_transaction(&mut spi, &mut reg, &mut protocol_buffer),
        //     CAR_CURR_STATE_SIGNAL.wait(),
        // )
        // .await
        // {
        //     Either::First(res) => {
        //         if let Err(e) = res {
        //             error!("SPI transaction failed: {:?}", e);
        //             // Continue processing despite errors
        //         } else {
        //             REGISTRY_SIGNAL.signal(reg.get_registry_data());
        //         }
        //     }
        //     Either::Second(car_state) => {
        //         reg.update_car_state(&car_state);
        //     }
        // }
        let res = handle_spi_transaction(&mut spi, &mut protocol_buffer).await;
        if let Err(e) = res {
            error!("SPI transaction failed: {:?}", e);
            // Continue processing despite errors
        } else {
        }
    }
}

async fn handle_spi_transaction(
    spi: &mut Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>,
    buffer: &mut [u8],
) -> Result<(), embassy_stm32::spi::Error> {
    // Read SPI header
    let header_slice = &mut buffer[0..spi_proto::PROTOCOL_OVERHEAD];
    spi.read(header_slice).await?;
    trace!("SPI received header: {:x}", header_slice);

    let operation_hint = SpiPackOpType::try_from((header_slice[0] >> 7) & 0x01).ok();
    let data_length_hint = u16::from_le_bytes([header_slice[1], header_slice[2]]);

    let result = process_spi_header(header_slice);
    let (register, operation_type, data_length) = match result {
        Ok(vals) => vals,
        Err(err_code) => {
            error!("SPI header processing failed: {:?}", err_code);
            let mut registry = REGISTRY.lock().await;
            registry.update_error_status(err_code);
            drop(registry);

            if let Some(op_type) = operation_hint {
                match op_type {
                    SpiPackOpType::Write => {
                        consume_write_payload_if_supported(spi, buffer, data_length_hint).await?;
                    }
                    SpiPackOpType::Read => {
                        send_error_response_if_supported(spi, buffer, data_length_hint).await?;
                    }
                }
            }

            return Ok(());
        }
    };
    trace!(
        "Register: {}, Operation: {}, Length: {}",
        register, operation_type, data_length
    );

    let data_len = data_length as usize;
    let reg_offset = register as usize * core::mem::size_of::<u32>();
    let reg_window_end = reg_offset.checked_add(data_len);
    let is_registry_window_valid = reg_window_end
        .map(|end| end <= spi_proto::REGISTERS_SIZE_BYTES)
        .unwrap_or(false);

    if data_len == 0 || data_len % core::mem::size_of::<u32>() != 0 || !is_registry_window_valid {
        let mut registry = REGISTRY.lock().await;
        registry.update_error_status(registry::ErrorCode::InvalidRequestLength);
        drop(registry);

        match operation_type {
            SpiPackOpType::Write => {
                consume_write_payload_if_supported(spi, buffer, data_length).await?;
            }
            SpiPackOpType::Read => {
                send_error_response_if_supported(spi, buffer, data_length).await?;
            }
        }
        return Ok(());
    }

    match operation_type {
        SpiPackOpType::Write => {
            let total_write_size = data_len + spi_proto::PROTOCOL_CRC_SIZE;
            if total_write_size > buffer.len() {
                let mut registry = REGISTRY.lock().await;
                registry.update_error_status(registry::ErrorCode::InvalidRequestLength);
                drop(registry);
                consume_write_payload_if_supported(spi, buffer, data_length).await?;
                return Ok(());
            }

            let write_buffer = &mut buffer[0..total_write_size];
            spi.read(write_buffer).await?;
            trace!("Received data: {:?}", write_buffer);
            let data = process_spi_packet(write_buffer);
            let data = match data {
                Ok(d) => d,
                Err(err_code) => {
                    error!("SPI packet processing failed: {:?}", err_code);
                    let mut registry = REGISTRY.lock().await;
                    registry.update_error_status(err_code);
                    return Ok(());
                }
            };
            // registry.update_registry(register, data);
            let update_res = {
                let mut registry = REGISTRY.lock().await;
                let res = registry.update_registry(register, data);
                if let Err(err) = res {
                    registry.update_error_status(err);
                }
                res
            };
            if let Err(err_code) = update_res {
                error!("Registry update failed: {:?}", err_code);
                return Ok(());
            }

            let registry_data = {
                let registry = REGISTRY.lock().await;
                registry.get_registry_data()
            };
            trace!("Registry updated: {:?}", registry_data);
            Ok(())
        }
        SpiPackOpType::Read => {
            let total_response_size = data_len + spi_proto::PROTOCOL_CRC_SIZE;
            if total_response_size > buffer.len() {
                let mut registry = REGISTRY.lock().await;
                registry.update_error_status(registry::ErrorCode::InvalidRequestLength);
                drop(registry);
                send_error_response_if_supported(spi, buffer, data_length).await?;
                return Ok(());
            }

            let response_buffer = &mut buffer[0..total_response_size];

            let (data, crc_slice) = response_buffer.split_at_mut(data_len);
            // Read data from registry
            // registry.read_registry(register, data);
            let read_res = {
                let mut registry = REGISTRY.lock().await;
                let res = registry.read_registry(register, data);
                if let Err(err) = res {
                    registry.update_error_status(err);
                }
                res
            };
            if let Err(err_code) = read_res {
                error!("Registry read failed: {:?}", err_code);
                send_error_response_if_supported(spi, buffer, data_length).await?;
                return Ok(());
            }
            trace!("Read data: {:x}", data);

            // Add CRC
            populate_crc(data, crc_slice);

            // Send response
            spi.write(response_buffer).await?;
            trace!("Data sent: {:x}", response_buffer);

            Ok(())
        }
    }
}

async fn consume_write_payload_if_supported(
    spi: &mut Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>,
    buffer: &mut [u8],
    data_length: u16,
) -> Result<(), embassy_stm32::spi::Error> {
    let data_len = usize::from(data_length);
    let Some(total_write_size) = data_len.checked_add(spi_proto::PROTOCOL_CRC_SIZE) else {
        return Ok(());
    };

    if data_len > spi_proto::REGISTERS_SIZE_BYTES || total_write_size > buffer.len() {
        return Ok(());
    }

    let discard_buffer = &mut buffer[0..total_write_size];
    spi.read(discard_buffer).await?;
    Ok(())
}

async fn send_error_response_if_supported(
    spi: &mut Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>,
    buffer: &mut [u8],
    data_length: u16,
) -> Result<(), embassy_stm32::spi::Error> {
    let data_len = usize::from(data_length);
    let Some(total_response_size) = data_len.checked_add(spi_proto::PROTOCOL_CRC_SIZE) else {
        return Ok(());
    };

    if data_len > spi_proto::REGISTERS_SIZE_BYTES || total_response_size > buffer.len() {
        return Ok(());
    }

    let response_buffer = &mut buffer[0..total_response_size];
    let (data, crc_slice) = response_buffer.split_at_mut(data_len);
    data.fill(0);
    populate_crc(data, crc_slice);
    spi.write(response_buffer).await?;
    Ok(())
}

pub fn build_car_hw_cfg(
    per: embassy_stm32::Peripherals,
) -> (
    Car<'static>,
    Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>,
) {
    info!("Hello World!");
    // DC driver 1 configuration
    let m1_ins_a = Output::new(per.PB2, Level::Low, Speed::Low);
    let m1_ins_b = Output::new(per.PB10, Level::Low, Speed::Low);
    let m2_ins_a = Output::new(per.PB13, Level::Low, Speed::Low);
    let m2_ins_b = Output::new(per.PB12, Level::Low, Speed::Low);

    // DC driver 2 configuration
    let m3_ins_a = Output::new(per.PC14, Level::Low, Speed::Low);
    let m3_ins_b = Output::new(per.PC15, Level::Low, Speed::Low);
    let m4_ins_a = Output::new(per.PB1, Level::Low, Speed::Low);
    let m4_ins_b = Output::new(per.PB0, Level::Low, Speed::Low);

    // Driver control state
    let vcc_gpio = Output::new(per.PC13, Level::High, Speed::Low);
    let standby_gpio = Output::new(per.PA12, Level::Low, Speed::Low);
    // PWM configuration
    let ch1_pin = PwmPin::new(per.PA8, OutputType::PushPull);
    let ch2_pin = PwmPin::new(per.PA9, OutputType::PushPull);
    let ch3_pin = PwmPin::new(per.PA10, OutputType::PushPull);
    let ch4_pin = PwmPin::new(per.PA11, OutputType::PushPull);
    let pwm = SimplePwm::new(
        per.TIM1,
        Some(ch1_pin),
        Some(ch2_pin),
        Some(ch3_pin),
        Some(ch4_pin),
        khz(50),
        Default::default(),
    );
    // Split PWM channels
    let simple_pwm_channels = pwm.split();
    let mot1_pwm = simple_pwm_channels.ch2;
    let mot2_pwm = simple_pwm_channels.ch1;
    let mot3_pwm = simple_pwm_channels.ch3;
    let mot4_pwm = simple_pwm_channels.ch4;

    #[cfg(feature = "ext_pin_clk")]
    {
        pub const ALERNATIVE_FUNC_2: u8 = 2;
        pub const ALERNATIVE_FUNC_1: u8 = 1;
        let pin_ext_clk_mot1 = Flex::new(per.PA0);
        let pin_ext_clk_mot2 = Flex::new(per.PA6);
        let pin_ext_clk_mot3 = Flex::new(per.PB6);
        let pin_ext_clk_mot4 = Flex::new(per.PA15);

        let timer_motor1 = ExtClkTimer::new(per.TIM5);
        let timer_motor2 = ExtClkTimer::new(per.TIM3);
        let timer_motor3 = ExtClkTimer::new(per.TIM4);
        let timer_motor4 = ExtClkTimer::new(per.TIM2);

        let motor1 = Motor::new(
            "MOTOR_1",
            mot1_pwm,
            m1_ins_a,
            m1_ins_b,
            timer_motor1,
            pin_ext_clk_mot1,
            ALERNATIVE_FUNC_2,
            MOTOR_COUNT_PER_REV,
        );
        let motor2 = Motor::new(
            "MOTOR_2",
            mot2_pwm,
            m2_ins_a,
            m2_ins_b,
            timer_motor2,
            pin_ext_clk_mot2,
            ALERNATIVE_FUNC_2,
            MOTOR_COUNT_PER_REV,
        );
        let motor3 = Motor::new(
            "MOTOR_3",
            mot3_pwm,
            m3_ins_a,
            m3_ins_b,
            timer_motor3,
            pin_ext_clk_mot3,
            ALERNATIVE_FUNC_2,
            MOTOR_COUNT_PER_REV,
        );
        let motor4 = Motor::new(
            "MOTOR_4",
            mot4_pwm,
            m4_ins_a,
            m4_ins_b,
            timer_motor4,
            pin_ext_clk_mot4,
            ALERNATIVE_FUNC_1,
            MOTOR_COUNT_PER_REV,
        );
        let car = Car::new(motor1, motor2, motor3, motor4, vcc_gpio, standby_gpio);
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = khz(5000);
        let spi = Spi::new_slave(
            per.SPI1,
            per.PA5,
            per.PB5,
            per.PB4,
            per.PA4,
            per.DMA2_CH3,
            per.DMA2_CH2,
            spi_config,
        );
        (car, spi)
    }

    // Quadrature configuration
    #[cfg(feature = "encoder")]
    {
        let qei_config = QeiConfig::default();
        let qei_motor1 = Qei::new(per.TIM5, per.PA0, per.PA1, qei_config);
        let qei_motor2 = Qei::new(per.TIM3, per.PA7, per.PA6, qei_config);
        let qei_motor3 = Qei::new(per.TIM4, per.PB6, per.PB7, qei_config);
        let qei_motor4 = Qei::new(per.TIM2, per.PB3, per.PA15, qei_config);

        let motor1 = Motor::new(
            "MOTOR_1",
            mot1_pwm,
            m1_ins_a,
            m1_ins_b,
            qei_motor1,
            MOTOR_COUNT_PER_REV,
        );

        let motor2 = Motor::new(
            "MOTOR_2",
            mot2_pwm,
            m2_ins_a,
            m2_ins_b,
            qei_motor2,
            MOTOR_COUNT_PER_REV,
        );

        let motor3 = Motor::new(
            "MOTOR_3",
            mot3_pwm,
            m3_ins_a,
            m3_ins_b,
            qei_motor3,
            MOTOR_COUNT_PER_REV,
        );

        let motor4 = Motor::new(
            "MOTOR_4",
            mot4_pwm,
            m4_ins_a,
            m4_ins_b,
            qei_motor4,
            MOTOR_COUNT_PER_REV,
        );

        let car = Car::new(motor1, motor2, motor3, motor4, vcc_gpio, standby_gpio);
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = khz(5000);
        let spi = Spi::new_slave(
            per.SPI1,
            per.PA5,
            per.PB5,
            per.PB4,
            per.PA4,
            per.DMA2_CH3,
            per.DMA2_CH2,
            spi_config,
        );
        (car, spi)
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let (car, spi) = build_car_hw_cfg(p);
    spawner.spawn(motor_control_task(car).unwrap());

    spawner.spawn(spi_task(spi).unwrap());

    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        ticker.next().await;
    }
}
