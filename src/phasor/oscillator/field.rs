//! OscillatorField - a collection of oscillators built from the facet.

use phiton::physics::oscillator::Oscillator;
use crate::phasor::facet::Facet;
use phistd::c64;
use rayon::prelude::*;
use std::collections::HashMap;
use std::f64::consts::PI;

/// OscillatorField - the oscillator equivalent of the facet + wave system.
pub struct OscillatorField {
    pub oscillators: Vec<(String, Oscillator)>,
    pub index: HashMap<String, usize>,
}

impl OscillatorField {
    /// Builds an oscillator field from the facet's lexicon.
    pub fn from_facet(facet: &Facet) -> Self {
        let oscillators: Vec<(String, Oscillator)> = facet
            .lexicon
            .iter()
            .map(|(word, phasor)| {
                let osc = Oscillator::from_phasor(phasor.phase, phasor.amplitude, phasor.band_n);
                (word.clone(), osc)
            })
            .collect();

        let index: HashMap<String, usize> = oscillators
            .iter()
            .enumerate()
            .map(|(i, (w, _))| (w.clone(), i))
            .collect();

        Self { oscillators, index }
    }

    /// Returns the oscillator for a specific word.
    pub fn get(&self, word: &str) -> Option<&Oscillator> {
        self.index.get(word).map(|&i| &self.oscillators[i].1)
    }

    /// Computes the Kuramoto order parameter r ∈ [0, 1] for a sentence.
    pub fn sentence_coherence(&self, words: &[String]) -> f64 {
        let oscs: Vec<&Oscillator> = words.iter().filter_map(|w| self.get(w)).collect();
        if oscs.is_empty() { return 0.0; }

        let n = oscs.len() as f64;
        let sum: c64 = oscs
            .iter()
            .map(|o| c64::from_polar(1.0, o.longitude))
            .sum();
        sum.norm() / n
    }

    /// Projects the field onto a viewing angle at time t.
    pub fn project(&self, view_lat: f64, view_lon: f64, t: f64, top_k: usize) -> Vec<(String, String, f64)> {
        let mut visible: Vec<(String, String, f64)> = self
            .oscillators
            .par_iter()
            .map(|(word, osc)| {
                let vis = osc.visibility(view_lat, view_lon, t);
                let color = osc.color(t);
                (word.clone(), color, vis * osc.amplitude)
            })
            .filter(|(_, _, w)| *w > 0.0)
            .collect();

        visible.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        visible.into_iter().take(top_k).collect()
    }

    /// Computes the average synchronization across all pairs in a sentence.
    pub fn sentence_sync(&self, words: &[String]) -> f64 {
        let oscs: Vec<&Oscillator> = words.iter().filter_map(|w| self.get(w)).collect();
        if oscs.len() < 2 { return 0.0; }

        let mut total = 0.0;
        let mut count = 0;
        for i in 0..oscs.len() {
            for j in (i + 1)..oscs.len() {
                total += oscs[i].synchronization(oscs[j]);
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { total / count as f64 }
    }

    /// Computes the spectral entropy of the field at time t.
    pub fn spectral_entropy(&self, t: f64) -> f64 {
        let mut color_counts: HashMap<String, f64> = HashMap::new();
        let mut total = 0.0;

        for (_, osc) in &self.oscillators {
            let color = osc.color(t);
            let weight = osc.amplitude;
            *color_counts.entry(color).or_insert(0.0) += weight;
            total += weight;
        }

        if total == 0.0 { return 0.0; }

        color_counts
            .values()
            .map(|&w| { let p = w / total; -p * p.ln() })
            .sum()
    }

    /// Returns the dominant colors at time t (top N by total amplitude).
    pub fn dominant_colors(&self, t: f64, top_n: usize) -> Vec<(String, f64)> {
        let mut color_amp: HashMap<String, f64> = HashMap::new();
        for (_, osc) in &self.oscillators {
            let color = osc.color(t);
            *color_amp.entry(color).or_insert(0.0) += osc.amplitude;
        }

        let mut ranked: Vec<(String, f64)> = color_amp.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.into_iter().take(top_n).collect()
    }

    /// Returns oscillators within a latitude band (for sphere rendering).
    pub fn words_at_latitude(&self, lat: f64, t: f64) -> Vec<(String, String, f64)> {
        let mut band_words: Vec<(String, String, f64)> = self
            .oscillators
            .iter()
            .filter(|(_, o)| (o.latitude - lat).abs() < PI / 6.0)
            .map(|(w, o)| (w.clone(), o.color(t), o.amplitude))
            .collect();

        band_words.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        band_words
    }
}
