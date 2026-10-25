//! A room as a scan understands it: its category and dimensions, its
//! floors, walls and ceilings with their materials, its doors and windows,
//! and the furniture in it.

use super::anchor::{next_id, Point, Point2, Transform, IDENTITY};
use super::mesh::Mesh;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    pub enum Category { Unknown, Bedroom, LivingRoom, Kitchen, Bathroom, DiningRoom, Office, Hallway, Garage, Storage, Other }
}

named! { pub enum SurfaceType { Floor, Wall, Ceiling } }

named! { pub enum MaterialCategory { Wood, Tile, Carpet, Concrete, Paint, Glass, Metal, Other } }

named! { pub enum OpeningType { Door, Window, Opening, Fireplace } }

named! {
    pub enum ObjectCategory { Sofa, Chair, Table, Bed, Desk, Cabinet, Appliance, Plant, Lighting, Television, Storage, Other }
}

/// Width, length and height in metres, with the area and volume they make.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimensions {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

impl Dimensions {
    pub fn area(&self) -> f32 {
        self.width * self.length
    }

    pub fn volume(&self) -> f32 {
        self.area() * self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub category: MaterialCategory,
    pub color: [f32; 3],
}

/// A floor, wall or ceiling.
#[derive(Debug, Clone, PartialEq)]
pub struct Surface {
    pub id: u64,
    pub kind: SurfaceType,
    pub size: Point2,
    pub transform: Transform,
    pub material: Option<Material>,
    pub boundary: Vec<Point2>,
}

impl Surface {
    pub fn new(kind: SurfaceType, size: Point2) -> Self {
        Self { id: next_id(), kind, size, transform: IDENTITY, material: None, boundary: Vec::new() }
    }

    pub fn area(&self) -> f32 {
        self.size[0] * self.size[1]
    }
}

/// A door, window or gap in a wall.
#[derive(Debug, Clone, PartialEq)]
pub struct Opening {
    pub id: u64,
    pub kind: OpeningType,
    pub position: Point,
    pub size: Point2,
    pub wall: u64,
    pub open: bool,
}

/// A piece of furniture.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub id: u64,
    pub category: ObjectCategory,
    pub name: String,
    pub position: Point,
    pub size: Point,
    pub transform: Transform,
    pub confidence: f32,
}

impl Object {
    pub fn new(category: ObjectCategory, position: Point, size: Point) -> Self {
        Self { id: next_id(), name: crate::kit::words::said(crate::kit::tell::Named::ident(category)), category, position, size, transform: IDENTITY, confidence: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub id: u64,
    pub name: String,
    pub category: Category,
    pub dimensions: Dimensions,
    pub surfaces: Vec<Surface>,
    pub openings: Vec<Opening>,
    pub furniture: Vec<Object>,
    pub geometry: Option<Mesh>,
    /// Seconds since the epoch when captured.
    pub captured_at: u64,
}

impl Room {
    pub fn new(name: impl Into<String>, category: Category, dimensions: Dimensions) -> Self {
        Self { id: next_id(), name: name.into(), category, dimensions, surfaces: Vec::new(), openings: Vec::new(), furniture: Vec::new(), geometry: None, captured_at: 0 }
    }

    pub fn surfaces_of(&self, kind: SurfaceType) -> impl Iterator<Item = &Surface> {
        self.surfaces.iter().filter(move |s| s.kind == kind)
    }

    /// The room as a sentence: its category, then what is in it, largest first.
    pub fn sentence(&self) -> String {
        let mut things: Vec<&Object> = self.furniture.iter().collect();
        things.sort_by(|a, b| (b.size[0] * b.size[1] * b.size[2]).total_cmp(&(a.size[0] * a.size[1] * a.size[2])));
        let mut said = vec![crate::kit::words::said(crate::kit::tell::Named::ident(self.category))];
        said.extend(self.openings.iter().map(|o| crate::kit::words::said(crate::kit::tell::Named::ident(o.kind))));
        said.extend(things.iter().map(|o| o.name.clone()));
        said.join(", ")
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Category>(path);
    r.extend(cases::<SurfaceType>(path));
    r.extend(cases::<MaterialCategory>(path));
    r.extend(cases::<OpeningType>(path));
    r.extend(cases::<ObjectCategory>(path));
    r.extend(fields(path, "Dimensions", &["width", "length", "height"]));
    r.extend(fields(path, "Material", &["name", "category", "color"]));
    r.extend(fields(path, "Surface", &["id", "kind", "size", "transform", "material", "boundary"]));
    r.extend(fields(path, "Opening", &["id", "kind", "position", "size", "wall", "open"]));
    r.extend(fields(path, "Object", &["id", "category", "name", "position", "size", "transform", "confidence"]));
    r.extend(fields(path, "Room", &["id", "name", "category", "dimensions", "surfaces", "openings", "furniture", "geometry", "captured_at"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_room_says_what_is_in_it() {
        let mut room = Room::new("study", Category::Office, Dimensions { width: 3.0, length: 4.0, height: 2.5 });
        assert_eq!(room.dimensions.volume(), 30.0);
        room.surfaces.push(Surface::new(SurfaceType::Wall, [4.0, 2.5]));
        room.furniture.push(Object::new(ObjectCategory::Chair, [0.0; 3], [0.5, 0.9, 0.5]));
        room.furniture.push(Object::new(ObjectCategory::Desk, [0.0; 3], [1.6, 0.75, 0.8]));
        assert_eq!(room.surfaces_of(SurfaceType::Wall).count(), 1);
        assert_eq!(room.sentence(), "office, desk, chair");
    }
}
