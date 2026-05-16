#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use anyhow::{bail, Result};
use esp_idf_hal::{
    delay::FreeRtos,
    rmt::{
        config::{Loop, TransmitConfig},
        encoder::CopyEncoder,
        PinState, Pulse, PulseTicks, Symbol, TxChannelDriver,
    },
};

#[cfg(all(
    esp_idf_soc_rmt_supported,
    esp_idf_version_at_least_5_0_0,
    not(feature = "rmt-legacy")
))]
#[cfg(all(
    esp_idf_soc_rmt_supported,
    esp_idf_version_at_least_5_0_0,
    not(feature = "rmt-legacy")
))]

pub fn _disco(tx_channel: &mut TxChannelDriver, encoder: &mut CopyEncoder) -> Result<()> {
    // 3 seconds white at 10% brightness

    set_neopixel_colour(tx_channel, encoder, RGB::new(25, 25, 25))?;
    FreeRtos::delay_ms(3000);

    // Infinite rainbow loop at 20% brightness
    (0..360).cycle().try_for_each(|hue| {
        FreeRtos::delay_ms(10);
        let rgb = RGB::_from_hsv(hue, 100, 20)?;
        set_neopixel_colour(tx_channel, encoder, rgb)
    })
}

pub fn set_neopixel_colour(
    tx: &mut TxChannelDriver,
    encoder: &mut CopyEncoder,
    rgb: RGB,
) -> Result<()> {
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

    // WS2812 wants GRB order, MSB first — 24 bits total
    let color: u32 = rgb.into();
    let signal: Vec<Symbol> = (0..24)
        .rev()
        .map(|i| if (color >> i) & 1 == 1 { t1 } else { t0 })
        .collect();

    // SAFETY: encoder and signal are valid for the duration of this blocking call
    unsafe {
        tx.start_send(
            encoder,
            &signal,
            &TransmitConfig {
                loop_count: Loop::Count(1),
                ..Default::default()
            },
        )
    }?; // Block until transmission is complete

    Ok(())
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
