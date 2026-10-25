//! Geometry as tracking reconstructs it: a mesh of vertices and triangles,
//! classified face by face, held in mesh anchors that a reconstruction
//! keeps up to date as the device looks around. The face's geometry and a
//! plane's are the same `Mesh`.

use super::anchor::{Anchor, Point, Point2, Transform};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

/// Vertices and triangles, with what else is known per vertex or per face.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub normals: Vec<Point>,
    pub texture_coordinates: Vec<Point2>,
    /// Three indices per triangle.
    pub indices: Vec<u32>,
    /// One classification per triangle, when the reconstruction classifies.
    pub classifications: Vec<Classification>,
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn vertex(&self, i: usize) -> Option<Point> {
        self.vertices.get(i).copied()
    }

    /// The three vertices of triangle `i`.
    pub fn triangle(&self, i: usize) -> Option<[Point; 3]> {
        let idx = self.indices.get(i * 3..i * 3 + 3)?;
        Some([self.vertex(idx[0] as usize)?, self.vertex(idx[1] as usize)?, self.vertex(idx[2] as usize)?])
    }
}

named! {
    /// What a reconstructed face of the world is.
    pub enum Classification { None, Wall, Floor, Ceiling, Table, Seat, Window, Door }
}

named! {
    /// Whether classification is available for a mesh.
    pub enum ClassificationStatus { NotAvailable, Available, Undetermined }
}

named! {
    /// How sure a classification is.
    pub enum Confidence { Low, Medium, High }
}

named! {
    /// What a reconstruction produces.
    pub enum Reconstruction { None, Mesh, MeshWithClassification }
}

/// A piece of the reconstructed world.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshAnchor {
    pub anchor: Anchor,
    pub mesh: Mesh,
    /// Not yet read since it was added or last changed.
    pub fresh: bool,
}

impl MeshAnchor {
    pub fn new(transform: Transform, mesh: Mesh) -> Self {
        Self { anchor: Anchor::new(transform), mesh, fresh: true }
    }
}

/// How a reconstruction runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconstructionOptions {
    pub mode: Reconstruction,
    /// Updates per second the reconstruction aims at.
    pub update_rate: f32,
    /// The largest chunk the world is cut into, in metres.
    pub chunk_size: f32,
}

impl Default for ReconstructionOptions {
    fn default() -> Self {
        Self { mode: Reconstruction::Mesh, update_rate: 10.0, chunk_size: 1.0 }
    }
}

/// The reconstruction: its options and the mesh anchors it keeps.
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub options: ReconstructionOptions,
    pub anchors: Vec<MeshAnchor>,
}

impl Scene {
    pub fn new(options: ReconstructionOptions) -> Self {
        Self { options, anchors: Vec::new() }
    }

    pub fn add(&mut self, anchor: MeshAnchor) {
        self.anchors.push(anchor);
    }

    /// Replace the mesh of anchor `id`; it is fresh again.
    pub fn update(&mut self, id: u64, mesh: Mesh) -> bool {
        match self.anchors.iter_mut().find(|a| a.anchor.id == id) {
            Some(a) => {
                a.mesh = mesh;
                a.fresh = true;
                true
            }
            None => false,
        }
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let n = self.anchors.len();
        self.anchors.retain(|a| a.anchor.id != id);
        self.anchors.len() < n
    }

    /// The anchors not yet read, marked read.
    pub fn take_fresh(&mut self) -> Vec<&MeshAnchor> {
        self.anchors.iter_mut().filter(|a| a.fresh).map(|a| { a.fresh = false; &*a }).collect()
    }

    pub fn triangle_count(&self) -> usize {
        self.anchors.iter().map(|a| a.mesh.triangle_count()).sum()
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = fields(path, "Mesh", &["vertices", "normals", "texture_coordinates", "indices", "classifications"]);
    r.extend(fields(path, "ReconstructionOptions", &["mode", "update_rate", "chunk_size"]));
    r.extend(cases::<Classification>(path));
    r.extend(cases::<ClassificationStatus>(path));
    r.extend(cases::<Confidence>(path));
    r.extend(cases::<Reconstruction>(path));
    r
}

#[cfg(test)]
mod tests {
    use super::super::anchor::IDENTITY;
    use super::*;

    #[test]
    fn a_scene_keeps_its_meshes_fresh_until_read() {
        let mesh = Mesh { vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], indices: vec![0, 1, 2], ..Default::default() };
        let mut scene = Scene::default();
        let a = MeshAnchor::new(IDENTITY, mesh.clone());
        let id = a.anchor.id;
        scene.add(a);
        assert_eq!(scene.triangle_count(), 1);
        assert_eq!(scene.anchors[0].mesh.triangle(0).unwrap()[1], [1.0, 0.0, 0.0]);
        assert_eq!(scene.take_fresh().len(), 1);
        assert_eq!(scene.take_fresh().len(), 0);
        assert!(scene.update(id, mesh));
        assert_eq!(scene.take_fresh().len(), 1);
        assert!(scene.remove(id) && !scene.remove(id));
    }
}
