#[allow(unused_imports)]
use super::*;

impl DiskPhasor {
    pub(super) fn into_phasor(self) -> SpectralPhasor {
        SpectralPhasor::from_packed(self.packed, self.amplitude as f64, self.band_n, self.count)
    }
}

impl DiskPhasorV4 {
    pub(super) fn into_phasor(self) -> SpectralPhasor {
        SpectralPhasor::from_packed_with_evidence(
            self.packed,
            widen_evidence(self.evid),
            self.amplitude as f64,
            self.band_n,
            self.count,
        )
    }
}

impl DiskPhasorV5 {
    pub(super) fn into_phasor(self) -> SpectralPhasor {
        SpectralPhasor::from_packed_sides(
            self.packed,
            self.tgt,
            widen_evidence(self.evid),
            self.amplitude as f64,
            self.band_n,
            self.count,
        )
    }
}

impl DiskPhasorV6 {
    pub(super) fn from(p: &SpectralPhasor) -> Self {
        Self {
            packed: p.packed(),
            tgt: p.packed_target(),
            evid: p.packed_evidence(),
            amplitude: p.amplitude as f32,
            band_n: p.band_n,
            count: p.count,
        }
    }

    pub(super) fn into_phasor(self) -> SpectralPhasor {
        SpectralPhasor::from_packed_sides(self.packed, self.tgt, self.evid, self.amplitude as f64, self.band_n, self.count)
    }
}

impl LegacyPhasor {
    pub(super) fn into_phasor(self) -> SpectralPhasor {
        let mut p = SpectralPhasor::from_packed(self.packed, self.amplitude, self.band_n, self.count);
        if p.channels_unset() {
            p.phase = self.phase;
        }
        p
    }
}

impl<'a> V1Reader<'a> {
    pub(super) fn u64(&mut self) -> Option<u64> {
        let v = u64::from_le_bytes(self.body.get(self.o..self.o + 8)?.try_into().ok()?);
        self.o += 8;
        Some(v)
    }
    pub(super) fn f64(&mut self) -> Option<f64> {
        let v = f64::from_le_bytes(self.body.get(self.o..self.o + 8)?.try_into().ok()?);
        self.o += 8;
        Some(v)
    }
    pub(super) fn u32(&mut self) -> Option<u32> {
        let v = u32::from_le_bytes(self.body.get(self.o..self.o + 4)?.try_into().ok()?);
        self.o += 4;
        Some(v)
    }
    pub(super) fn string(&mut self) -> Option<String> {
        let len = self.u64()? as usize;
        let s = std::str::from_utf8(self.body.get(self.o..self.o + len)?).ok()?.to_string();
        self.o += len;
        Some(s)
    }
    pub(super) fn at_end(&self) -> bool {
        self.o >= self.body.len()
    }

    /// `HashMap<String, HashMap<String, T>>` with `T` read by `leaf`.
    pub(super) fn nested<T>(&mut self, leaf: fn(&mut Self) -> Option<T>) -> Option<HashMap<String, HashMap<String, T>>> {
        let n = self.u64()? as usize;
        if n > self.body.len() {
            return None;
        }
        let mut out = HashMap::with_capacity(n);
        for _ in 0..n {
            let k = self.string()?;
            let m = self.u64()? as usize;
            if m > self.body.len() {
                return None;
            }
            let mut inner = HashMap::with_capacity(m);
            for _ in 0..m {
                let w = self.string()?;
                inner.insert(w, leaf(self)?);
            }
            out.insert(k, inner);
        }
        Some(out)
    }

    pub(super) fn parse(body: &'a [u8], record: usize, expected: usize) -> Option<V1Body> {
        let mut r = V1Reader { body, o: 0 };
        let n = r.u64()? as usize;
        if expected != 0 && n != expected {
            return None;
        }
        let mut lexicon = HashMap::with_capacity(n);
        for _ in 0..n {
            let word = r.string()?;
            let phase = r.f64()?;
            let amplitude = r.f64()?;
            let band_n = r.u32()?;
            let count = match record >= 24 {
                true => r.u32()?,
                false => 0,
            };
            let mut packed = [0u64; 8];
            if record >= 88 {
                for w in packed.iter_mut() {
                    *w = r.u64()?;
                }
            }
            if !phase.is_finite() || !amplitude.is_finite() {
                return None;
            }
            lexicon.insert(word, LegacyPhasor { phase, amplitude, band_n, count, packed });
        }
        // The lexicon must be followed by either end of file or a table that
        // parses; a wrong record width lands in the middle of a string.
        let mut v1 = V1Body {
            lexicon,
            bigrams: HashMap::new(),
            trigrams: HashMap::new(),
            phase_lags: HashMap::new(),
            grounded_version: 0,
        };
        if r.at_end() {
            return Some(v1);
        }
        v1.bigrams = r.nested(|r| r.u32())?;
        if r.at_end() {
            return Some(v1);
        }
        v1.trigrams = r.nested(|r| r.u32())?;
        if r.at_end() {
            return Some(v1);
        }
        v1.phase_lags = r.nested(|r| r.f64())?;
        if r.at_end() {
            return Some(v1);
        }
        v1.grounded_version = r.u32()?;
        Some(v1)
    }
}
