use defmt::{error, info};
use embassy_stm32::{
    gpio::Output,
    timer::{GeneralInstance4Channel, qei::Qei, simple_pwm::SimplePwmChannel},
};
use heapless::Vec;

use crate::registry as reg;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorCurrState {
    pub rpm: i32,
    pub direction: reg::Direction,
    pub pwm_duty: i32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CarCurrState {
    pub motors: Vec<MotorCurrState, 4>,
}
pub struct Motor<'a, S: GeneralInstance4Channel, Q: GeneralInstance4Channel> {
    pub name: &'static str,
    pub pwm: SimplePwmChannel<'a, S>,
    pub ins_a: Output<'a>,
    pub ins_b: Output<'a>,
    pub qei: Qei<'a, Q>,
    pub desired_rpm: f32,
    pub last_rpm: f32,
    pub count_per_rev: u32,
    pub last_count: u32,
    pub last_time: u64,
    pub control_mode: reg::ControlMode,
}

pub struct Car<'a> {
    pub motor1: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM5>,
    pub motor2: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM3>,
    pub motor3: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM4>,
    pub motor4: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM2>,
    pub vcc_gpio: Output<'a>,
    pub standby_gpio: Output<'a>,
}

impl<'a, S: GeneralInstance4Channel, Q: GeneralInstance4Channel> Motor<'a, S, Q> {
    pub fn new(
        name: &'static str,
        pwm: SimplePwmChannel<'a, S>,
        ins_a: Output<'a>,
        ins_b: Output<'a>,
        qei: Qei<'a, Q>,
        count_per_rev: u32,
    ) -> Self {
        Self {
            name,
            pwm,
            ins_a,
            ins_b,
            qei,
            count_per_rev: count_per_rev,
            last_count: 0,
            last_rpm: 0.0,
            last_time: embassy_time::Instant::now().as_millis(),
            desired_rpm: 0.0,
            control_mode: reg::ControlMode::PwmControl,
        }
    }

    pub fn init(&mut self) {
        self.ins_a.set_low();
        self.ins_b.set_low();
        self.pwm.enable();
    }

    pub fn set_direction(&mut self, dir: reg::Direction) {
        info!("{}: Setting direction to {:?}", self.name, dir);
        self.ins_a.set_low();
        self.ins_b.set_low();
        match dir {
            reg::Direction::Forward => {
                self.ins_a.set_high();
                self.ins_b.set_low();
            }
            reg::Direction::Backward => {
                self.ins_a.set_low();
                self.ins_b.set_high();
            }
            reg::Direction::Stop => {
                self.ins_a.set_low();
                self.ins_b.set_low();
            }
        }
    }

    pub fn set_pwm_duty(&mut self, duty: i32) {
        assert!(
            duty >= 0 && duty <= 100,
            "PWM duty cycle must be between 0 and 100"
        );
        if self.get_pwm_duty() != duty {
            info!("{}: Setting PWM duty to {}", self.name, duty);
            self.pwm.set_duty_cycle_percent(duty as u8);
        }
    }

    pub fn get_direction(&mut self) -> reg::Direction {
        if self.ins_a.is_set_low() && self.ins_b.is_set_low() && self.pwm.current_duty_cycle() == 0
        {
            reg::Direction::Stop
        } else {
            if self.ins_a.is_set_high() && self.ins_b.is_set_low() {
                reg::Direction::Forward
            } else if self.ins_a.is_set_low() && self.ins_b.is_set_high() {
                reg::Direction::Backward
            } else if self.ins_a.is_set_low() && self.ins_b.is_set_low() {
                reg::Direction::Stop
            } else {
                // Invalid state
                error!(
                    "{}: Invalid motor driver state: IN_A={}, IN_B={}",
                    self.name,
                    self.ins_a.is_set_high(),
                    self.ins_b.is_set_high()
                );
                reg::Direction::Stop
            }
        }
    }

    pub fn get_pwm_duty(&mut self) -> i32 {
        ((self.pwm.current_duty_cycle() as u32 * 100) / self.pwm.max_duty_cycle()) as i32
    }

    pub fn get_count(&mut self) -> u16 {
        self.qei.count()
    }

    pub fn calculate_rpm(&mut self, interval_ms: u32) -> f32 {
        let count = self.qei.count() as f32;
        let last_count = self.last_count as f32;
        let interval_ms = interval_ms as f32;
        let count_per_rev = self.count_per_rev as f32;
        let rpm = (count - last_count) / count_per_rev / (interval_ms / 60000.0);
        self.last_count = count as u32;
        rpm
    }

    pub fn apply_cfg(&mut self, mot_cfg: &reg::MotorConfig) {
        self.control_mode = mot_cfg.control_mode;
        if self.control_mode == reg::ControlMode::PwmControl {
            if mot_cfg.direction != self.get_direction() {
                self.set_direction(mot_cfg.direction);
            }
            self.set_pwm_duty(mot_cfg.pwm_duty_cycle);
        }
        self.count_per_rev = mot_cfg.counts_per_revolution as u32;
        self.desired_rpm = mot_cfg.rpm_desired as f32;
    }

    pub fn get_curr_state(&mut self) -> MotorCurrState {
        MotorCurrState {
            rpm: self.last_rpm as i32,
            direction: self.get_direction(),
            pwm_duty: self.pwm.current_duty_cycle() as i32,
        }
    }

    pub fn ctrl_loop(&mut self) {
        let current_time = embassy_time::Instant::now().as_millis();
        let interval_ms = (current_time - self.last_time) as u32;
        let rpm = self.calculate_rpm(interval_ms);
        if self.last_rpm != rpm {
            self.last_rpm = rpm;
            // info!("{}: Motor rpm {:?}", self.name, rpm);
        }
        self.last_time = current_time;
    }
}

impl<'a> Car<'a> {
    pub fn new(
        motor1: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM5>,
        motor2: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM3>,
        motor3: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM4>,
        motor4: Motor<'a, embassy_stm32::peripherals::TIM1, embassy_stm32::peripherals::TIM2>,
        vcc_gpio: Output<'a>,
        standby_gpio: Output<'a>,
    ) -> Self {
        Self {
            motor1,
            motor2,
            motor3,
            motor4,
            vcc_gpio,
            standby_gpio,
        }
    }

    pub fn init(&mut self) {
        self.vcc_gpio.set_high();
        self.standby_gpio.set_high();
        self.motor1.init();
        self.motor2.init();
        self.motor3.init();
        self.motor4.init();
    }

    pub fn apply_cfg(&mut self, reg_data: &reg::RegistryData) {
        self.motor1.apply_cfg(&reg_data.motors[0]);
        self.motor2.apply_cfg(&reg_data.motors[1]);
        self.motor3.apply_cfg(&reg_data.motors[2]);
        self.motor4.apply_cfg(&reg_data.motors[3]);
    }

    pub fn get_curr_state(&mut self) -> CarCurrState {
        let mut motors: Vec<MotorCurrState, 4> = Vec::new();
        motors.push(self.motor1.get_curr_state()).unwrap();
        motors.push(self.motor2.get_curr_state()).unwrap();
        motors.push(self.motor3.get_curr_state()).unwrap();
        motors.push(self.motor4.get_curr_state()).unwrap();
        CarCurrState { motors }
    }

    pub fn ctrl_loop(&mut self) {
        self.motor1.ctrl_loop();
        self.motor2.ctrl_loop();
        self.motor3.ctrl_loop();
        self.motor4.ctrl_loop();
    }
}
