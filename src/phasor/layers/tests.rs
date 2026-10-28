#[allow(unused_imports)]
use super::*;

#[test]
fn test_hierarchical_phase_field() {
    let mut facet = Facet::new();
    facet.get_or_init("rust");
    facet.get_or_init("code");
    facet.get_or_init("memory");

    let mut field = HierarchicalPhaseField::new();
    field.build_hierarchy(&facet);

    assert_eq!(field.layers.len(), 4);
    assert!(!field.layers[1].clusters.is_empty());
    assert_eq!(field.layers[0].level, 0);

    // Query at a phase a word actually occupies. Probing a fixed angle only
    // worked while seeding was length-based and every short word landed on
    // the same handful of sectors.
    let probe = facet.get_phasor("rust").map(|p| p.phase).unwrap();
    let depth_res = field.resonate_depth(probe);
    assert!(!depth_res.is_empty(), "the layer containing a known word must resonate");

    // The hierarchy must now be usable for retrieval, not only display.
    let path = field.descend(probe);
    assert!(!path.is_empty(), "descent must produce a coarse-to-fine path");
    let near = field.candidates(&facet, probe, 3);
    assert!(near.iter().any(|(w, _)| w == "rust"), "the probe word should be found: {:?}", near);
}
