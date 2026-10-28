//! The lexicon's starting values, its sector resolution and its syntax lag.
//! They were phiano's `config` constants; phiano's `config` re-exports them
//! from here.

/// Initial amplitude for new words.
pub const AMPLITUDE_INITIAL: f64 = 1.0;

/// Initial band_n for new words.
pub const BAND_N_INITIAL: u32 = 1;

/// Sector resolution - number of sectors dividing the 2π phase circle.
/// Must be a power of 2 with exponent ≥ 6 (64, 128, 256, 512, 1024).
pub const SECTOR_RESOLUTION: u16 = 64;

/// Minimum sector resolution exponent (2⁶ = 64).
pub const SECTOR_EXPONENT_MIN: u32 = 6;

/// Kuramoto-Sakaguchi asymmetric syntax phase lag (β = π/16).
/// Enforces time-ordered syntactic flow in sequences.
pub const SYNTACTIC_LAG_BETA: f64 = std::f64::consts::PI / 16.0;

/// EMA rate for learning pairwise phase lags from word order.
pub const SYNTAX_LAG_LEARN_RATE: f64 = 0.08;

/// How far apart two words may stand and still be coupled as a pair.
pub const RELATION_WINDOW: usize = 8;

// The sector resolution is a power of two no smaller than 2^SECTOR_EXPONENT_MIN;
// phiano checked it at run time, the compiler checks it here.
const _: () = assert!(SECTOR_RESOLUTION.is_power_of_two() && SECTOR_RESOLUTION >= 1 << SECTOR_EXPONENT_MIN);

/// Kuramoto learning rate for phase relaxation.
/// Recommended range: [0.01, 0.15]. Default: 0.05 (balanced).
pub const LEARNING_RATE: f64 = 0.05;

/// Anti-phase pulse applied to a misunderstood concept (π radians).
pub const PHASE_REPULSION: f64 = std::f64::consts::PI;

/// Default syntactic forward phase momentum velocity.
pub const SYNTACTIC_MOMENTUM_DEFAULT: f64 = 0.15;
