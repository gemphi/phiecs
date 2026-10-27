//! The flow field (phi-oml `phifly::flow_field`, re-based): a scene of
//! named signals twitched twice. phi-oml's field is a `width × height` grid
//! of pixel cells; here a cell is a thing seen (`hand`, `wall`), so the
//! scene is a list and its order is the parallax. The phase shift a thing shows under the eye's
//! own micro-jitter is parallax; the near moves fast and the far hardly
//! moves (phi-oml's README: `d ∝ 1/Δx`), so one twitch sorts the scene
//! into near, mid and far with no second camera. What is new here is the
//! last step: the field is **told as rows**, so a scene enters a `World`
//! by the same door as a sentence or the kit.

use super::signal::{jitter, Aoc};
use crate::Row;

/// How fast a cell's phase moved under the twitch, binned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    Near,
    Mid,
    Far,
}

impl Pace {
    /// A shift's bin against the field's lower and upper thirds.
    pub fn of(shift: f64, lo: f64, hi: f64) -> Pace {
        match shift {
            v if v >= hi => Pace::Near,
            v if v > lo => Pace::Mid,
            _ => Pace::Far,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Pace::Near => "near",
            Pace::Mid => "mid",
            Pace::Far => "far",
        }
    }
}

/// The eye's current scene: the things seen, each with its signal.
#[derive(Debug, Clone, Default)]
pub struct Flow {
    pub cells: Vec<(String, Aoc)>,
}

impl Flow {
    /// An empty scene.
    pub fn new() -> Self {
        Self::default()
    }

    /// A thing seen at a cell: its name and its signal. Depth is carried
    /// as amplitude by the sensor (a close thing presses harder), and the
    /// twitch reads it back out as phase velocity.
    pub fn see(&mut self, name: &str, signal: Aoc) {
        self.cells.push((name.to_string(), signal));
    }

    /// One micro-jitter: every cell's phase shift under a twitch of
    /// `angle`, scaled by its amplitude — the parallax each thing shows.
    pub fn twitch(&self, angle: f64) -> Vec<(String, f64)> {
        self.cells
            .iter()
            .map(|(n, s)| {
                let before = s.resonance(jitter(0.0));
                let after = s.resonance(jitter(angle));
                (n.clone(), phistd::wrap(after - before).abs() * s.amplitude)
            })
            .collect()
    }

    /// The shifts binned against the field's own thirds: the top third of
    /// movers are near, the bottom third far. An empty field has no pace.
    pub fn paces(&self, angle: f64) -> Vec<(String, Pace, f64)> {
        let shifts = self.twitch(angle);
        let mut sorted: Vec<f64> = shifts.iter().map(|x| x.1).collect();
        sorted.sort_by(f64::total_cmp);
        match (sorted.get(sorted.len() / 3), sorted.get(sorted.len() * 2 / 3)) {
            (Some(&lo), Some(&hi)) => shifts.into_iter().map(|(n, v)| (n, Pace::of(v, lo, hi), v)).collect(),
            _ => Vec::new(),
        }
    }

    /// The field told as rows under `scope` (`sight`, say): every thing
    /// `within` the scope, `does` its pace with the shift as weight, and
    /// the scope `says` the thing — the same kinds a kit or a story uses.
    pub fn rows(&self, scope: &str, angle: f64) -> Vec<Row> {
        let scope = format!("scope:{scope}");
        let mut out = Vec::new();
        for (name, pace, v) in self.paces(angle) {
            out.push(Row::new(&name, "within", &scope));
            out.push(Row::weighted(&name, "does", pace.name(), v));
            out.push(Row::new(&scope, "says", &name));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_twitch_sorts_the_scene_and_tells_it() {
        let mut f = Flow::new();
        f.see("hand", Aoc::new(0.4, 3.0)); // close: presses hard
        f.see("table", Aoc::new(1.1, 1.0));
        f.see("wall", Aoc::new(2.0, 0.2)); // far: faint
        let p = f.paces(0.25);
        assert_eq!(p[0].1, Pace::Near, "the hand moves most under the twitch");
        assert_eq!(p[2].1, Pace::Far, "the wall hardly moves");
        let rows = f.rows("sight", 0.25);
        assert_eq!(rows.len(), 9);
        assert!(rows.iter().any(|r| r.about == "hand" && r.kind == "does" && r.other == "near"));
        assert!(rows.iter().any(|r| r.about == "scope:sight" && r.kind == "says" && r.other == "wall"));
        // Rows mean a World can be told a scene directly.
        let w = crate::world::World::from_rows(&rows);
        assert!(w.lexicon.get("hand").is_some() && w.lexicon.get("near").is_some());
    }

    #[test]
    fn an_empty_field_has_no_pace_and_one_thing_is_near() {
        let f = Flow::new();
        assert!(f.paces(0.25).is_empty() && f.rows("sight", 0.25).is_empty(), "no cells, no panic");
        let mut f = Flow::new();
        f.see("lamp", Aoc::new(0.7, 1.0));
        assert_eq!(f.paces(0.25)[0].1, Pace::Near, "a lone thing is its own top third");
    }
}
