//! SphereView - renders text-based projections of the oscillator sphere.

use super::OscillatorField;
use std::f64::consts::PI;

/// SphereView - renders the color spectrum from a viewing angle.
pub struct SphereView;

impl SphereView {
    /// Renders the equatorial color wheel at time t.
    pub fn render_wheel(field: &OscillatorField, t: f64) -> String {
        let mut output = String::new();
        output.push_str("  ── oscillator sphere: equatorial wheel ──\n\n");

        let bands = phiton::SpectralBand::BANDS;
        let n = bands.len();
        let sector_size = 2.0 * PI / n as f64;

        for (i, band) in bands.iter().enumerate() {
            let view_lon = i as f64 * sector_size;
            let visible = field.project(0.0, view_lon, t, 3);

            output.push_str(&format!("  {:>10} │", band.name));
            if visible.is_empty() {
                output.push_str("  (empty)\n");
            } else {
                let words: Vec<String> = visible
                    .iter()
                    .map(|(w, _, weight)| format!("{} [{:.2}]", w, weight))
                    .collect();
                output.push_str(&format!("  {}\n", words.join(", ")));
            }
        }
        output
    }

    /// Renders a latitude cross-section of the sphere.
    pub fn render_sphere(field: &OscillatorField, t: f64) -> String {
        let mut output = String::new();
        output.push_str("  ── oscillator sphere: full projection ──\n\n");

        let bands = [
            ("N pole",      PI / 2.0),
            ("N temperate", PI / 6.0),
            ("Equator",     0.0),
            ("S temperate", -PI / 6.0),
            ("S pole",     -PI / 2.0),
        ];

        for (band_name, lat) in &bands {
            output.push_str(&format!("  {:>12} │", band_name));
            let band_words = field.words_at_latitude(*lat, t);

            if band_words.is_empty() {
                output.push_str("  (no oscillators)\n");
            } else {
                let shown: Vec<String> = band_words
                    .iter()
                    .take(5)
                    .map(|(w, _, amp)| format!("{}({:.1})", w, amp))
                    .collect();
                output.push_str(&format!("  {}\n", shown.join("  ")));
            }
        }

        let entropy = field.spectral_entropy(t);
        output.push_str(&format!("\n  spectral entropy: {:.4}\n", entropy));

        let dominant = field.dominant_colors(t, 5);
        output.push_str("  dominant spectrum: ");
        let colors: Vec<String> = dominant
            .iter()
            .map(|(c, a)| format!("{}({:.1})", c, a))
            .collect();
        output.push_str(&colors.join(" "));
        output.push('\n');
        output
    }
}
