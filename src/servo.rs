use esp_idf_svc::hal::ledc::LedcDriver;
use esp_idf_svc::sys::EspError;

pub struct Servo<'d> {
    pwm: LedcDriver<'d>,
    max_duty: u32,
}

impl<'d> Servo<'d> {
    // Pulse range for 0 and 180 degrees. Many hobby servos want 500..2500,
    // others 1000..2000. Tune these to your servo so it doesn't hit the end stops.
    const MIN_US: u32 = 500;
    const MAX_US: u32 = 2500;
    const PERIOD_US: u32 = 20_000; // 50 Hz

    pub fn new(pwm: LedcDriver<'d>) -> Result<Self, EspError> {
        let max_duty = pwm.get_max_duty();
        let mut servo = Self { pwm, max_duty };
        servo.set_angle(90)?; // start centred
        Ok(servo)
    }

    /// Angle in degrees, clamped to 0..=180.
    pub fn set_angle(&mut self, degrees: u32) -> Result<(), EspError> {
        let degrees = degrees.min(180);
        let pulse_us = Self::MIN_US + (Self::MAX_US - Self::MIN_US) * degrees / 180;
        let duty = self.max_duty * pulse_us / Self::PERIOD_US;
        self.pwm.set_duty(duty)
    }
}
