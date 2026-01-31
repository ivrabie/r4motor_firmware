#![no_std]
#![no_main]
#![warn(dead_code)]

use defmt::*;
use embassy_executor::Spawner;
use heapless::Vec;

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};


use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed},
    spi::{Config as SpiConfig, Spi},
    time::khz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
    },
};
#[cfg(feature = "encoder")]
use embassy_stm32::{
    timer::qei::{
        Config as QeiConfig, Qei
    },
};
#[cfg(feature = "ext_pin_clk")]
use embassy_stm32::{
    gpio::Flex,
    timer::low_level::Timer,
};

use embassy_time::{Duration, Ticker};
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
async fn motor_control_task() {
    let mut car_ctrl = build_car_hw_cfg();
    let mut ticker = Ticker::every(Duration::from_millis(50));
    let mut print_cnt: u32 = 0;
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
        car_ctrl.apply_cfg(&reg_data);
        car_ctrl.ctrl_loop();
        let car_curr_status = car_ctrl.get_curr_state();
        if car_curr_status != prev_car_state {
            let mut registry = REGISTRY.lock().await;
            registry.update_car_state(&car_curr_status);
            prev_car_state = car_curr_status.clone();
        }
        ticker.next().await;
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

    let result = process_spi_header(header_slice);
    let (register, operation_type, data_length) = match result {
        Ok(vals) => vals,
        Err(err_code) => {
            error!("SPI header processing failed: {:?}", err_code);
            // registry.update_error_status(err_code);
            {
                let mut registry = REGISTRY.lock().await;
                registry.update_error_status(err_code);
            }
            return Ok(()); // Continue processing
        }
    };
    trace!(
        "Register: {}, Operation: {}, Length: {}",
        register, operation_type, data_length
    );

    match operation_type {
        SpiPackOpType::Write => {
            let write_buffer = &mut buffer[0..data_length as usize + spi_proto::PROTOCOL_CRC_SIZE];
            spi.read(write_buffer).await?;
            trace!("Received data: {:?}", write_buffer);
            let data = process_spi_packet(write_buffer);
            let data = match data {
                Ok(d) => d,
                Err(err_code) => {
                    error!("SPI packet processing failed: {:?}", err_code);
                    // registry.update_error_status(err_code);
                    {
                        let mut registry = REGISTRY.lock().await;
                        registry.update_error_status(err_code);
                    }
                    return Ok(()); // Continue processing
                }
            };
            // registry.update_registry(register, data);
            let registry_data = {
                let mut registry = REGISTRY.lock().await;
                registry.update_registry(register, data);
                registry.get_registry_data()
            };
            trace!("Registry updated: {:?}", registry_data);
            Ok(())
        }
        SpiPackOpType::Read => {
            let total_response_size = data_length as usize + spi_proto::PROTOCOL_CRC_SIZE;
            let response_buffer = &mut buffer[0..total_response_size];

            let (data, crc_slice) = response_buffer.split_at_mut(data_length as usize);
            // Read data from registry
            // registry.read_registry(register, data);
            {
                let mut registry = REGISTRY.lock().await;
                registry.read_registry(register, data);
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

pub fn build_car_hw_cfg<'a>() -> Car<'a> {
    info!("Hello World!");
    let per = unsafe { embassy_stm32::Peripherals::steal() };
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
        pub const ALERNATIVE_FUNC_2 : u8 = 2;
        pub const ALERNATIVE_FUNC_1 : u8 = 1;
        let pin_ext_clk_mot1 = Flex::new(per.PA0);
        let pin_ext_clk_mot2 = Flex::new(per.PA6);
        let pin_ext_clk_mot3 = Flex::new(per.PB6);
        let pin_ext_clk_mot4 = Flex::new(per.PA15); 
        
        
        let timer_motor1 = Timer::new(per.TIM5);
        let timer_motor2 = Timer::new(per.TIM3);
        let timer_motor3 = Timer::new(per.TIM4);
        let timer_motor4 = Timer::new(per.TIM2);

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
        Car::new(motor1, motor2, motor3, motor4, vcc_gpio, standby_gpio)
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
    
        Car::new(motor1, motor2, motor3, motor4, vcc_gpio, standby_gpio)
    }

}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    // SPI configuration (Slave mode)S
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = khz(5000);
    let spi = Spi::new_slave(
        p.SPI1, p.PA5, // SCK
        p.PB5, // MOSI
        p.PB4, // MISO
        p.PA4, // CS
        p.DMA2_CH3, p.DMA2_CH2, spi_config,
    );
    spawner.spawn(motor_control_task().unwrap());

    spawner.spawn(spi_task(spi).unwrap());

    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        ticker.next().await;
    }
}
