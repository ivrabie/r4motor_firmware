use defmt::{info};
pub struct PidCfg {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub output_min: f32,
    pub output_max: f32,
}

pub struct Pid {
    pub cfg: PidCfg,
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub prev_error: f32,
}


impl PidCfg {
    pub fn new(kp: f32, ki: f32, kd: f32, output_min: f32, output_max: f32) -> Self {
        PidCfg {
            kp,
            ki,
            kd,
            output_min,
            output_max,
        }
    }
}


impl Pid { 
    pub fn new() -> Self {
        Pid {
            cfg : PidCfg::new(1.0, 0.0, 0.0, 0.0, 100.0),
            p: 0.0,
            i: 0.0,
            d: 0.0,
            prev_error: 0.0,
        }
    }

    pub fn compute(&mut self, setpoint: f32, measured: f32) -> f32 {
        // Calculate the error between desired and actual values
        let error = setpoint - measured;
        
        // Compute derivative term based on error change
        self.d = self.cfg.kd * (error - self.prev_error);
        
        // Compute proportional term based on current error
        self.p = self.cfg.kp * error;

        // Calculate raw output by summing all PID terms
        let o_raw = self.p + self.i + self.d;
        
        // Clamp output to configured limits
        let o_clamped = o_raw.clamp(self.cfg.output_min, self.cfg.output_max);
        
        // Calculate saturation error for anti-windup
        let sat_err = o_clamped - o_raw;
        
        // Calculate anti-windup gain (Ki/Kp ratio)
        let kaw = if self.cfg.kp == 0.0 {
            0.0
        } else {
            self.cfg.ki / self.cfg.kp
        };

        
        // Update integral term with error and anti-windup compensation
        self.i += self.cfg.ki * error + kaw * sat_err;
        
        // Calculate integral term limits based on remaining output range
        let i_max = self.cfg.output_max - self.p - self.d;
        let i_min = self.cfg.output_min - self.p - self.d;
        
        // Clamp integral term to prevent further saturation
        self.i = self.i.clamp(i_min, i_max);
        
        // Store current error for next derivative calculation
        self.prev_error = error;
        
        info!("PID Compute => SP: {}, Meas: {}, P: {}, I: {}, D: {}, Out raw {} Out: {}", setpoint, measured, self.p, self.i, self.d, o_raw, o_clamped);
        o_clamped
    }
}