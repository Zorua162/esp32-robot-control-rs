#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use anyhow::{bail, Result};
use esp_idf_svc::hal::{
    gpio::OutputPin,
    rmt::{
        config::TxChannelConfig,
        config::{Loop, TransmitConfig},
        encoder::CopyEncoder,
        PinState, Pulse, PulseTicks, Symbol, TxChannelDriver,
    },
    units::Hertz,
};

const RMT_RESOLUTION: Hertz = Hertz(10_000_000);

pub struct NeoPixelControl<'d> {
    tx: TxChannelDriver<'d>,
    encoder: CopyEncoder,
    t0: Symbol,
    t1: Symbol,
}

impl<'d> NeoPixelControl<'d> {
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self> {
        let tx = TxChannelDriver::new(
            pin,
            &TxChannelConfig {
                resolution: RMT_RESOLUTION,
                ..Default::default()
            },
        )?;
        let encoder = CopyEncoder::new()?;
        // WS2812 timing (at 10MHz, 1 tick = 100ns):
        //   T0H = 350ns  → 4 ticks (rounded to nearest 100ns)
        //   T0L = 800ns  → 8 ticks
        //   T1H = 700ns  → 7 ticks
        //   T1L = 600ns  → 6 ticks
        let t0 = Symbol::new(
            Pulse::new(PinState::High, PulseTicks::new(4)?),
            Pulse::new(PinState::Low, PulseTicks::new(8)?),
        );
        let t1 = Symbol::new(
            Pulse::new(PinState::High, PulseTicks::new(7)?),
            Pulse::new(PinState::Low, PulseTicks::new(6)?),
        );

        Ok(Self {
            tx,
            encoder,
            t0,
            t1,
        })
    }

    pub fn set_colour(&mut self, rgb: RGB) -> Result<()> {
        // WS2812 wants GRB order, MSB first — 24 bits total
        let color: u32 = rgb.into();
        let signal: Vec<Symbol> = (0..24)
            .rev()
            .map(|i| {
                if (color >> i) & 1 == 1 {
                    self.t1
                } else {
                    self.t0
                }
            })
            .collect();

        // SAFETY: encoder and signal are valid for the duration of this blocking call
        unsafe {
            self.tx.start_send(
                &mut self.encoder,
                &signal,
                &TransmitConfig {
                    loop_count: Loop::Count(1),
                    ..Default::default()
                },
            )
        }?;

        Ok(())
    }
}

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert hue (0–360), saturation (0–100), value (0–100) to RGB
    pub fn _from_hsv(h: u32, s: u32, v: u32) -> anyhow::Result<Self> {
        if h > 360 || s > 100 || v > 100 {
            bail!("HSV values out of range");
        }
        let s = s as f64 / 100.0;
        let v = v as f64 / 100.0;
        let c = s * v;
        let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Ok(Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
        })
    }

    pub fn off() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub fn red() -> Self {
        Self { r: 255, g: 0, b: 0 }
    }

    pub fn green() -> Self {
        Self { r: 0, g: 255, b: 0 }
    }

    pub fn blue() -> Self {
        Self { r: 0, g: 0, b: 255 }
    }

    pub fn purple() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
        }
    }

    pub fn yellow() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 0,
        }
    }

    pub fn cyan() -> Self {
        Self {
            r: 0,
            g: 255,
            b: 255,
        }
    }
}

impl From<RGB> for u32 {
    /// Pack as RGB
    ///
    /// Bits 23–16 = G, bits 15–8 = R, bits 7–0 = B
    fn from(rgb: RGB) -> Self {
        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32
    }
}
