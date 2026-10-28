//! The oscillator over a facet: each word's spectral phasor read as a
//! `phiton::physics::oscillator::Oscillator` on the sphere (the field), the
//! view from one point of it, and the trainer that moves the field. It was
//! phiano's `oscillator`; the evaluation against the Evaluator is
//! `phient::eval::oscillator`.

pub mod field;
pub mod train;
pub mod view;

pub use field::OscillatorField;
pub use train::OscillatorTrainer;
pub use view::SphereView;
