# phiecs — what is left to do

Paths are under `src/`. Diagrams: [ARCHITECTURE.md](ARCHITECTURE.md).

## Learning

- [ ] world: `Phasor::couple` is a running mean, with no error term (phasor/mod.rs). Make it error-driven, as `phient::lineup::learner` is, or keep it only as the counted contrast line (docs/203).
- [ ] phasor: a word's phasor has two forms side by side, `Phasor` (f64 slots, the world's) and `spectral::SpectralPhasor` (64 bytes plus evidence, the facet's lexicon). Both live here now; one should be the other's storage, not a second type, and the facet would read `Phasor` then.
- [ ] world: `read_mode` caches `PHI_READ` in a process-wide `OnceLock`, so the read mode cannot differ per world or per test. A bad `bound:` value falls back to `Domain`, while the other modes fall back to their own (world/relate/mode.rs:67, :74).

## Correctness

- [ ] world: `World::contains` tests the lexicon, not the roll, so it is still true for a word after `forget` (world/mod.rs:105).
- [ ] kit: because of that, the `world.contains(w)` guard in `Watch::word` is always true, and a forgotten shape word is never coined again (kit/watch.rs:38).
- [ ] row: `Row::weighted` allows a count from a strand, but `parse_tsv` clamps weights to 0..=1, so counted rows lose their counts on a TSV round trip (row.rs:24, :42).

## Dead or unwired

- [ ] aspect: `on_bind`/`on_unbind` are called by `Dense::insert`/`remove`, but no aspect overrides them (aspect.rs:15, :18).
- [ ] condition: the `Condition` aspect is never bound to a word outside its own test (condition/mod.rs:19).
- [ ] quest: `Quest.conditions` (a `ConditionSet`) is built in `Quest::new` and none of its fields is read (quest/mod.rs:55, :84).
- [ ] query: `With2`, `Related`, `World::with2`, `World::related`, `with_mut` and `count_with` have no callers in the workspace (query/pair.rs:7, :27, :56, :61; query/mod.rs:22, :27).
- [ ] relation: `DEFAULT_KINDS` is defined and re-exported, never used (relation/set.rs:27).
- [ ] resources: `World.resources` is never read or written; a quest keeps its budget and steps in its own fields (resources.rs:5, world/mod.rs:36).
- [ ] system: `FnSystem`, `FnSystem::at` and every non-default `Stage` are used only in tests; `kit::Watch` is the only `System` outside tests (system/mod.rs:29, :40).
- [ ] kit: no real device feeds the kit yet; frames and registry dumps are described as coming from outside (kit/body/face.rs:11, kit/device/README.md:13).

## Moved in

- [x] phasor: `facet` (the lexicon) and `wave` (its sentence waves, ray cast and sector index) came from phiano, and `relation::vocab` (RelationVocab) with them; phiano reads all three from here.
- [ ] phasor: phiano's `lib.rs` keeps `pub use phiecs::phasor::facet;` only because the read-only examples `flow` and `meaning` name `phiano::facet::Facet`. Once they can be edited, they name `phiecs::phasor::facet` and the re-export goes.
- [ ] phasor: phiano's `PhiConfig::sector_resolution` still checks `SECTOR_RESOLUTION` at run time; `facet/constants.rs` checks it at compile time, so the run-time check can read the constant and stop checking.

## Stale docs

- [ ] quest: the `Synthetic::truth` doc says `opposite`, but the kind it emits is `unlike` (quest/synthetic.rs:41).
- [ ] phasor: in `gemgum`, `bind`, `binding_strength` and `band_position` have no callers (phasor/gemgum.rs); in phiano they were hidden by an `#[allow(dead_code)]` on the module, which did not move.
- [ ] relation: the `MAX_KINDS` doc says it caps the kinds a language holds, but `declare` opens extension channels past it without bound (relation/mod.rs:15).
- [ ] kit: the body, device and track READMEs are still titled `phikit::…`, and the body README cites `phiplm::prose::Knowing` (now `phient::prose`) (kit/body/README.md:1, :15; kit/device/README.md:1; kit/track/README.md:1).
- [ ] README: "Nothing learns from counts here", but `couple` is a running mean.
