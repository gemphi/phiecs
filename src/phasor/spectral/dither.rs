#[allow(unused_imports)]
use super::*;

/// How a nudge lands on the channel's byte.
///
/// A channel is one byte: 256 quanta of 2π, 0.0245 rad each. A nudge of
/// `rate · sin(d)` with `RELATION_RATE = 0.06` is smaller than half a quantum
/// once `sin(d) < 0.2` — 11° — and nearest rounding then writes back the byte
/// it read. Every coupled pair in the quest ingestion stopped exactly 8
/// quanta apart (`bin/quest_ingest`, docs/83), which put at most ~32
/// distinguishable targets on a channel however long it trained.
///
/// Dithering rounds up with probability equal to the fraction, so the
/// *expected* movement equals the nudge and small steps accumulate. It adds
/// variance — a coin per step — which a corpus of a million nudges pays for
/// (docs/83 measures it), so it is a choice the trainer makes, not the
/// phasor's default.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Rounding {
    /// `round()`: what every objective did before docs/83.
    #[default]
    Nearest,
    /// A coin on the fractional quantum, every step.
    Dither,
}

/// Round `quanta` to a byte under `rounding`. The coin is a hash of the
/// phasor's own state — its bytes and the channel's evidence count, which
/// every coupling increments — and the step, so a run is reproducible and the
/// same state is never asked the same coin twice.
#[inline]
pub fn quantise(quanta: f64, salt: u64, rounding: Rounding) -> u64 {
    let q = quanta.rem_euclid(CHANNEL_QUANTA);
    match rounding {
        Rounding::Nearest => (q.round() as u64) % CHANNEL_QUANTA as u64,
        Rounding::Dither => {
            let floor = q.floor();
            let frac = q - floor;
            let coin = (mix64(salt ^ quanta.to_bits()) >> 11) as f64 / (1u64 << 53) as f64;
            (floor as u64 + (frac > coin) as u64) % CHANNEL_QUANTA as u64
        }
    }
}

impl SpectralPhasor {
    /// Rotates channel `k` by `delta` radians under `rounding`.
    pub fn nudge_by(&mut self, k: usize, delta: f64, rounding: Rounding) {
        let k = k % PHASE_CHANNELS;
        let q = self.theta_byte(k) as f64 + delta / TWO_PI * CHANNEL_QUANTA;
        let salt = self.packed[k / 8] ^ self.evid[k / 2] ^ ((k as u64) << 56);
        let q = quantise(q, salt, rounding);
        let shift = 8 * (k % 8);
        let w = k / 8;
        self.packed[w] = (self.packed[w] & !(0xFFu64 << shift)) | (q << shift);
        if k == 0 {
            self.phase = self.theta(0);
        }
    }

    /// Rotates the target side of channel `k` by `delta` radians under
    /// `rounding`.
    pub fn nudge_target_by(&mut self, k: usize, delta: f64, rounding: Rounding) {
        let k = k % PHASE_CHANNELS;
        let raw = (self.tgt[k / 8] >> (8 * (k % 8))) & 0xFF;
        let q = raw as f64 + delta / TWO_PI * CHANNEL_QUANTA;
        let salt = self.tgt[k / 8] ^ self.evid[k / 2] ^ ((k as u64) << 48) ^ 0xD17;
        let q = quantise(q, salt, rounding);
        let shift = 8 * (k % 8);
        let w = k / 8;
        self.tgt[w] = (self.tgt[w] & !(0xFFu64 << shift)) | (q << shift);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_steps_accumulate_in_expectation() {
        // A quarter-quantum step, 4,000 times, moves ~1,000 quanta dithered
        // and exactly zero under nearest rounding.
        let step = 0.25 * TWO_PI / CHANNEL_QUANTA;
        let mut moved = [0i64; 2];
        for (slot, rounding) in [Rounding::Dither, Rounding::Nearest].into_iter().enumerate() {
            let mut p = SpectralPhasor::seeded("dither", 1.0, 1);
            let mut last = p.theta_byte(3) as i64;
            for _ in 0..4_000 {
                p.deposit(3, 1); // every coupling deposits, which is what varies the coin
                p.nudge_by(3, step, rounding);
                let now = p.theta_byte(3) as i64;
                let d = (now - last + 384) % 256 - 128;
                moved[slot] += d;
                last = now;
            }
        }
        assert!((moved[0] - 1_000).abs() < 150, "dithered moved {} quanta, expected ~1000", moved[0]);
        assert_eq!(moved[1], 0);
    }

    #[test]
    fn whole_steps_are_exact_either_way() {
        for rounding in [Rounding::Dither, Rounding::Nearest] {
            let mut p = SpectralPhasor::seeded("exact", 1.0, 1);
            let before = p.theta_byte(5);
            p.nudge_by(5, 3.0 * TWO_PI / CHANNEL_QUANTA, rounding);
            assert_eq!(p.theta_byte(5), before.wrapping_add(3), "{rounding:?}");
        }
    }
}
