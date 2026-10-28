#[allow(unused_imports)]
use super::*;

use crate::phasor::facet::Facet;
use phiton::PhitonSpectrum;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io;
use std::path::Path;

/// How many words a codon directory lists.
const CODON_WORDS: usize = 200;

/// Write `facet` as a helix tree at `dir`.
pub fn write(facet: &Facet, dir: &Path) -> io::Result<usize> {
    std::fs::create_dir_all(dir.join("strands"))?;
    std::fs::create_dir_all(dir.join("codons"))?;
    // codons.tsv — the code table, with the facet's own name for each channel.
    let mut table = String::from("k\tcodon\thexagram\tdegree\torder\tx\ty\tz\trelation\n");
    for c in Codon::table() {
        let _ = writeln!(
            table,
            "{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{}",
            c.k, c.codon, c.hexagram, c.degree, c.order, c.position[0], c.position[1], c.position[2], facet.relation_name(c.k)
        );
    }
    std::fs::write(dir.join("codons.tsv"), table)?;
    std::fs::write(dir.join("README.md"), readme(facet))?;

    // strands/<aa>/<word>.dna — one strand per word, sorted so the tree is
    // reproducible.
    let mut words: Vec<&String> = facet.lexicon.keys().collect();
    words.sort();
    let mut by_codon: Vec<Vec<(u32, &str)>> = vec![Vec::new(); CODONS];
    for w in &words {
        let p = &facet.lexicon[*w];
        let shard = shard_of(w);
        let sub = dir.join("strands").join(&shard);
        std::fs::create_dir_all(&sub)?;
        let mut s = format!("# {w}\tamplitude {:.6}\tband {}\tcount {}\n", p.amplitude, p.band_n, p.count);
        for (k, slot) in by_codon.iter_mut().enumerate() {
            let e = p.evidence_count(k);
            // The codon's light, in PhiTon's units: the source angle as a
            // wavelength, its spectral band and colour — a strand reads as a
            // spectrum, not a column of bytes.
            let light = PhitonSpectrum::phase_to_color(p.theta(k), p.band_n);
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{:.1}\t{}\t{}",
                codon(k),
                p.theta_byte(k),
                target_byte(p, k),
                e,
                light.wavelength_nm,
                light.name(),
                light.hex()
            );
            if e > 0 {
                slot.push((e, w.as_str()));
            }
        }
        std::fs::write(sub.join(format!("{}.dna", file_name(w))), s)?;
    }
    // codons/<CODON>/words.tsv — the channel's heaviest words.
    for (k, list) in by_codon.iter_mut().enumerate() {
        list.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        let sub = dir.join("codons").join(codon(k));
        std::fs::create_dir_all(&sub)?;
        let mut s = format!("# {} · {} · {}\nword\tevidence\n", codon(k), hexagram_glyph(k), facet.relation_name(k));
        for (e, w) in list.iter().take(CODON_WORDS) {
            let _ = writeln!(s, "{w}\t{e}");
        }
        std::fs::write(sub.join("words.tsv"), s)?;
    }
    Ok(words.len())
}

/// The stored target byte, untied (what `read` must get back).
fn target_byte(p: &crate::phasor::SpectralPhasor, k: usize) -> u8 {
    (p.theta_target_raw(k) / (2.0 * std::f64::consts::PI) * 256.0).round() as u8
}

/// Two-character shard so no directory holds the whole lexicon.
pub(super) fn shard_of(w: &str) -> String {
    let mut it = w.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase());
    let a = it.next().unwrap_or('_');
    let b = it.next().unwrap_or('_');
    format!("{a}{b}")
}

/// A file name that survives every filesystem: alphanumerics kept, the rest
/// hex-escaped, so `read` can invert it.
pub(super) fn file_name(w: &str) -> String {
    let mut s = String::new();
    for c in w.chars() {
        match c.is_ascii_alphanumeric() || c == '-' {
            true => s.push(c),
            false => {
                let _ = write!(s, "%{:02x}", c as u32);
            }
        }
    }
    s
}

fn readme(facet: &Facet) -> String {
    let markers = facet.relations.markers();
    let mut kinds: BTreeMap<usize, String> = BTreeMap::new();
    for k in 0..CODONS {
        kinds.insert(k, facet.relation_name(k));
    }
    let named: Vec<String> = kinds.iter().filter(|(k, _)| **k >= 8 && **k < 8 + markers).map(|(k, n)| format!("`{}` {n}", codon(*k))).collect();
    format!(
        "# helix\n\n{} words · {} marker relations\n\nA word is a strand of 64 codons, one per relation channel; each codon carries \
the word's source angle, target angle (the second strand), evidence on that channel, and the codon's light in PhiTon's units (wavelength nm, band, hex). \
`codons.tsv` is the code table: codon ↔ channel ↔ hexagram ↔ spherical-harmonic degree/order ↔ position on the helix.\n\n\
Named relations: {}\n\nQueries: `cat codons/GAT/words.tsv` (who carries a relation), \
`grep -l '^GAT\\t.*\\t[1-9]' strands/*/*.dna` (every word with evidence on it), \
`awk '$4>0' strands/ca/cat.dna` (a word's active codons).\n",
        facet.lexicon.len(),
        markers,
        named.join(", ")
    )
}
