//! How a candidate is weighed — the PHI_READ modes (docs/198–199).

use super::TURN;

/// How a candidate is weighed against the condition — under play (docs/198).
/// `PHI_READ`: `count` (the old read: agreement times the square root of
/// ln(1+n), so a loud word wins a crowded read by frequency alone),
/// `agree` (agreement alone — the difference, nothing else), or
/// `band:<turns>` (agreement alone, and only from candidates whose phase
/// lies within that many TURNs of the condition: a RESTRICTION OF
/// INFLUENCE, without which there are no groups, only one crowd).
pub(super) fn weigh(theta: f64, target: f64, amplitude: f64) -> Option<f64> {
    let agree = (theta - target).cos();
    match read_mode() {
        Mode::Count => Some(agree * amplitude.sqrt()),
        Mode::Agree => Some(agree),
        Mode::Band(turns) => {
            let apart = (theta - target).sin().atan2(agree).abs();
            (apart <= turns * TURN).then_some(agree)
        }
        Mode::Domain => Some(agree),
        Mode::Typed(_) => Some(agree * amplitude.sqrt()),
        Mode::Sure => {
            // amplitude is ln(1+n): recover n, then the saturating confidence.
            let n = amplitude.exp() - 1.0;
            Some(agree * (n / (n + 1.0)))
        }
        Mode::Bound(turns) => {
            let apart = (theta - target).sin().atan2(agree).abs();
            (apart <= turns * TURN).then_some(amplitude)
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Count,
    Agree,
    Band(f64),
    /// A relation has a DOMAIN and a RANGE, and a word sits in both at
    /// different angles (the user's framing, docs/198). Telling `a next b`
    /// pulls `a` as the thing REACHING and `b` as the thing REACHED, on two
    /// channels, so a word that is both a heavy predecessor and a heavy
    /// follower -- `the` -- no longer has one angle torn between the two.
    Domain,
    /// Domain/range channels, and a BAND: phase decides MEMBERSHIP (who may
    /// influence this read at all -- without a restriction of influence
    /// there are no groups, only one crowd), and among the members, which
    /// all agree well enough, confidence orders. A band is a coupling
    /// strength: wide enough to bind, narrow enough to leave things
    /// distinct (docs/198).
    Bound(f64),
    /// Domain/range channels, ordered by agreement times the phasor's own
    /// confidence n/(n+1) -- the step it already learns by, saturating at
    /// one, so a word said once cannot tie the top (as under `agree`) and a
    /// word said ten thousand times cannot buy it (as under `count`).
    Sure,
    /// TWO LAYERS (docs/199): the TYPE by the phasors -- membership is the
    /// domain/range read, which holds the truth 15 points more often than a
    /// frequency list of the same size -- and the THING within it by
    /// preference in associations, the old slot read. Who a word is matters
    /// less than what type is in play; within the type, association picks.
    Typed(f64),
}

pub(super) fn read_mode() -> Mode {
    use std::sync::OnceLock;
    static M: OnceLock<Mode> = OnceLock::new();
    *M.get_or_init(|| match std::env::var("PHI_READ").unwrap_or_default() {
        v if v == "agree" => Mode::Agree,
        v if v == "domain" => Mode::Domain,
        v if v == "sure" => Mode::Sure,
        v if v.starts_with("typed:") => v[6..].parse().map(Mode::Typed).unwrap_or(Mode::Typed(1.0)),
        v if v.starts_with("bound:") => v[6..].parse().map(Mode::Bound).unwrap_or(Mode::Domain),
        v if v.starts_with("band:") => v[5..].parse().map(Mode::Band).unwrap_or(Mode::Agree),
        _ => Mode::Count,
    })
}
