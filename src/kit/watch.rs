//! Watching: a phiecs `System` that reads a session's frames into the
//! world one step at a time. Each step takes one frame; the shapes a face
//! says in it are related `near` one another, and each shape said is
//! related `before` each shape the next frame says — pairs of words moving
//! under a condition, which is phiecs's whole learning rule. Nothing is
//! stored but the relations; what a persona's face does is read back as
//! the agreement of two shapes' phasors on a channel.

use crate::kit::body::shape::Shape;
use crate::kit::tell::Named;
use crate::kit::track::frame::Frame;
use crate::prelude::*;
use crate::system::System;
use std::collections::VecDeque;

/// Frames waiting to be read, and the threshold a shape must reach to be said.
pub struct Watch {
    pub frames: VecDeque<Frame>,
    pub tau: f32,
    last: Vec<Shape>,
    /// Frames read so far.
    pub steps: usize,
}

impl Watch {
    pub fn new(tau: f32) -> Self {
        Self { frames: VecDeque::new(), tau, last: Vec::new(), steps: 0 }
    }

    pub fn show(&mut self, frame: Frame) {
        self.frames.push_back(frame);
    }

    /// The word of a shape in `world`, coined if the world has not met it.
    pub fn word(world: &mut World, shape: Shape) -> Word {
        let name = shape.name();
        match world.lexicon.get(&name) {
            Some(w) if world.contains(w) => w,
            _ => world.coin(&name).bind(Phasor::seeded(&name)).word(),
        }
    }

    /// How far two shapes agree on `kind` (`near`, `before`): 1 together,
    /// −1 opposed, `None` when either has no evidence there.
    pub fn agreement(world: &World, a: Shape, b: Shape, kind: &str) -> Option<f64> {
        let k = world.relations.get(kind)?;
        let (wa, wb) = (world.lexicon.get(&a.name())?, world.lexicon.get(&b.name())?);
        world.read::<Phasor>(wa)?.agreement(world.read::<Phasor>(wb)?, k)
    }

    /// The shapes that follow `shape` most — READ FROM THE PHASORS: the
    /// shapes standing one turn past it on `before` (docs/187), not a count
    /// of frames.
    pub fn follows(world: &World, shape: Shape) -> Vec<(Shape, f64)> {
        let Some(k) = world.relations.get("before") else { return Vec::new() };
        let Some(w) = world.lexicon.get(&shape.name()) else { return Vec::new() };
        let mut v: Vec<(Shape, f64)> = world.reads(w, k).into_iter().filter_map(|(o, weight)| Shape::by_name(world.spelling(o).strip_prefix("shape:")?).map(|s| (s, weight))).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v
    }
}

impl System for Watch {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let Some(frame) = self.frames.pop_front() else { return };
        let (Some(near), Some(before)) = (world.declare("near"), world.declare("before")) else { return };
        let said: Vec<Shape> = frame.faces().flat_map(|f| f.said(self.tau)).collect();
        let words: Vec<Word> = said.iter().map(|s| Watch::word(world, *s)).collect();
        for (i, a) in words.iter().enumerate() {
            for b in &words[i + 1..] {
                world.relate(*a, near, *b);
            }
        }
        let last: Vec<Word> = self.last.iter().map(|s| Watch::word(world, *s)).collect();
        for a in &last {
            for b in &words {
                world.relate(*a, before, *b);
            }
        }
        self.last = said;
        self.steps += 1;
    }

    fn name(&self) -> &str {
        "watch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::body::face::Face;
    use crate::kit::track::frame::Tracked;

    fn frame(shapes: &[(Shape, f32)]) -> Frame {
        let mut f = Frame::default();
        f.anchors.push(Tracked::Face(Face::with(shapes)));
        f
    }

    #[test]
    fn a_world_watching_a_face_learns_what_follows_what() {
        let mut world = World::new();
        let mut watch = Watch::new(0.3);
        for _ in 0..12 {
            // A smile, then a blink; elsewhere a yawn, then the tongue out.
            watch.show(frame(&[(Shape::MouthSmileLeft, 0.9), (Shape::MouthSmileRight, 0.9)]));
            watch.show(frame(&[(Shape::EyeBlinkLeft, 0.8), (Shape::EyeBlinkRight, 0.8)]));
            watch.show(frame(&[]));
            watch.show(frame(&[(Shape::JawOpen, 0.9)]));
            watch.show(frame(&[(Shape::TongueOut, 0.7)]));
            watch.show(frame(&[]));
        }
        let n = watch.frames.len();
        world.add_system(watch);
        for _ in 0..n {
            world.step(1.0);
        }
        let near = Watch::agreement(&world, Shape::MouthSmileLeft, Shape::MouthSmileRight, "near").unwrap();
        // Twelve observations at the coupling rate close 0.84¹² ≈ 0.12 of the seeded gap.
        assert!(near > 0.9, "the two sides of the smile move together: {near}");
        // `before` is a TURN (docs/187): the blink stands one turn past the
        // smile, so their raw agreement is cos(turn), and the tongue is
        // further off; the ordering is what the physics promises.
        let smile_blink = Watch::agreement(&world, Shape::MouthSmileLeft, Shape::EyeBlinkLeft, "before").unwrap();
        let smile_tongue = Watch::agreement(&world, Shape::MouthSmileLeft, Shape::TongueOut, "before").unwrap();
        assert!(smile_blink > smile_tongue, "smile ⇒ blink {smile_blink:.3}, not tongue {smile_tongue:.3}");
        // Both eyes blink one turn past the smile — either is the right read.
        assert!(matches!(Watch::follows(&world, Shape::MouthSmileLeft)[0].0, Shape::EyeBlinkLeft | Shape::EyeBlinkRight));
        assert_eq!(Watch::follows(&world, Shape::JawOpen)[0].0, Shape::TongueOut);
        assert!(Watch::agreement(&world, Shape::MouthSmileLeft, Shape::BrowDownLeft, "near").is_none(), "never shown, never said");
    }
}
