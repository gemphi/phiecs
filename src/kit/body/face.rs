//! A tracked face, and the face as a language. Every frame the tracker
//! gives the face's anchor, its fifty-two shape coefficients, where each
//! eye is turned and where the gaze lands. Here a frame is a sentence: the
//! shapes at or above a threshold are what it says, strongest first, each
//! in its words (`mouth smile left`); a sequence of frames under a path
//! (`faces/<persona>/<scene>`) is a paragraph, told with the rows a story
//! is told with. Then phient's `Knowing` reads a face as it reads a story
//! and goes on from it: a persona's expressions are what it was told its
//! face does, counted under the scenes they happened in.
//!
//! No frame is produced here: frames come from a device. What is here is
//! the telling.

use super::shape::{Coefficients, Shape};
use crate::kit::tell::{fields, Named};
use crate::kit::track::anchor::{Anchor, Point, Transform, IDENTITY};
use crate::kit::track::mesh::Mesh;
use crate::quest::Row;
use crate::speak::paragraph;

/// One frame of a tracked face.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub anchor: Anchor,
    pub shapes: Coefficients,
    pub geometry: Mesh,
    pub left_eye: Transform,
    pub right_eye: Transform,
    /// Where the gaze lands, in the face's space, when known.
    pub look_at: Option<Point>,
}

impl Face {
    pub fn new(transform: Transform) -> Self {
        Self { anchor: Anchor::new(transform), shapes: [0.0; Shape::ALL.len()], geometry: Mesh::default(), left_eye: IDENTITY, right_eye: IDENTITY, look_at: None }
    }

    /// A face with the given shapes set; the rest at rest.
    pub fn with(shapes: &[(Shape, f32)]) -> Self {
        let mut f = Self::new(IDENTITY);
        for (s, c) in shapes {
            f.shapes[s.index()] = *c;
        }
        f
    }

    pub fn coefficient(&self, shape: Shape) -> f32 {
        self.shapes[shape.index()]
    }

    /// The shapes at or above `tau`, strongest first.
    pub fn said(&self, tau: f32) -> Vec<Shape> {
        let mut v: Vec<(Shape, f32)> = Shape::ALL.iter().map(|s| (*s, self.coefficient(*s))).filter(|(_, c)| *c >= tau).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v.into_iter().map(|(s, _)| s).collect()
    }

    /// The frame as a sentence: the shapes it says, each in its words,
    /// separated by commas; a frame saying nothing is a pause, `⟨rest⟩`.
    pub fn sentence(&self, tau: f32) -> String {
        let said: Vec<String> = self.said(tau).iter().map(|s| s.words().join(" ")).collect();
        if said.is_empty() { "⟨rest⟩".to_string() } else { said.join(", ") }
    }
}

/// A sequence of frames under `path` as the rows a paragraph is told with:
/// each frame a sentence, the words `next` to each other, every word
/// `within` the scene.
pub fn rows(path: &str, frames: &[Face], tau: f32) -> Vec<Row> {
    let sentences: Vec<String> = frames.iter().map(|f| f.sentence(tau)).collect();
    paragraph(path, &sentences)
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    fields(path, "Face", &["anchor", "shapes", "geometry", "left_eye", "right_eye", "look_at"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use phient::prose::Knowing;

    #[test]
    fn a_face_is_read_and_gone_on_from() {
        // A smile that closes the eyes, told three times under one scene.
        let smile = Face::with(&[(Shape::MouthSmileLeft, 0.9), (Shape::MouthSmileRight, 0.85), (Shape::CheekSquintLeft, 0.4), (Shape::JawOpen, 0.05)]);
        let squint = Face::with(&[(Shape::EyeBlinkLeft, 0.7), (Shape::EyeBlinkRight, 0.7), (Shape::MouthSmileLeft, 0.6), (Shape::MouthSmileRight, 0.6)]);
        let rest = Face::with(&[(Shape::JawOpen, 0.02)]);
        assert_eq!(smile.said(0.3), [Shape::MouthSmileLeft, Shape::MouthSmileRight, Shape::CheekSquintLeft]);
        assert_eq!(smile.sentence(0.3), "mouth smile left, mouth smile right, cheek squint left");
        assert_eq!(rest.sentence(0.3), "⟨rest⟩");
        let frames = vec![smile.clone(), squint.clone(), rest.clone(), smile, squint, rest.clone(), rest];
        let dir = std::env::temp_dir().join(format!("phikit_face_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("faces")).unwrap();
        let text: Vec<String> = frames.iter().map(|f| f.sentence(0.3)).collect();
        std::fs::write(dir.join("faces/greeting.txt"), text.join(". ") + ".").unwrap();
        let (k, told) = Knowing::learn(&dir);
        assert_eq!(told.len(), 1);
        let asked: Vec<String> = phient::prose::tokens("mouth smile left, mouth smile right, cheek squint left.");
        let (_, ranked, _) = k.next(&asked);
        assert_eq!(ranked[0].0, "eye", "after the smile the face was told to blink: {:?}", &ranked[..3]);
        assert_eq!(rows("faces/greeting", &frames, 0.3).iter().filter(|r| r.kind == "next").count(), phient::prose::tokens(&text.join(" ")).len());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
