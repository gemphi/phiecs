# phiecs — architecture

## The modules and what they use

```mermaid
graph TB
    subgraph crate["phiecs"]
        m_aspect["aspect"]
        m_condition["condition"]
        m_entity["entity<br/>level 0 · Entity · Names"]
        m_kit["kit<br/>body · device · track · eye"]
        m_node["node<br/>levels 1 and 2 · component · state · tree · graph"]
        m_persona["persona"]
        m_phasor["phasor<br/>spectral · gemgum · facet + storage + helix · wave<br/>layers · flow · attention · oscillator"]
        m_prelude["prelude"]
        m_query["query"]
        m_quest["quest"]
        m_relation["relation<br/>register · edge · vocab"]
        m_resources["resources"]
        m_row["row"]
        m_scope["scope"]
        m_speak["speak"]
        m_store["store"]
        m_system["system"]
        m_word["word"]
        m_world["world"]
    end
    m_aspect --> m_word
    m_condition --> m_aspect
    m_condition --> m_word
    m_entity --> m_word
    m_kit --> m_prelude
    m_node --> m_entity
    m_node --> m_aspect
    m_node --> m_store
    m_node --> m_relation
    m_kit --> m_quest
    m_kit --> m_scope
    m_kit --> m_speak
    m_kit --> m_system
    m_phasor --> m_aspect
    m_phasor --> m_relation
    m_query --> m_aspect
    m_query --> m_relation
    m_query --> m_word
    m_query --> m_world
    m_quest --> m_condition
    m_quest --> m_persona
    m_quest --> m_phasor
    m_quest --> m_relation
    m_quest --> m_row
    m_quest --> m_world
    m_relation --> m_word
    m_scope --> m_quest
    m_speak --> m_quest
    m_speak --> m_scope
    m_store --> m_aspect
    m_store --> m_word
    m_system --> m_world
    m_word --> m_store
    m_world --> m_aspect
    m_world --> m_condition
    m_world --> m_phasor
    m_world --> m_relation
    m_world --> m_resources
    m_world --> m_row
    m_world --> m_store
    m_world --> m_system
    m_world --> m_word
    X_phient["phient (crate)"]
    m_kit -.-> X_phient
    X_phistd["phistd (crate)"]
    m_phasor -.-> X_phistd
    m_world -.-> X_phistd
    X_phiton["phiton (crate)"]
    m_phasor -.-> X_phiton
    X_phical["phical (crate)"]
    m_phasor -.->|"helix: Codon"| X_phical
```

## How a told row moves the world, and how it is read

```mermaid
graph LR
    R["Row { about, kind, other, weight }"]
    T["World::tell<br/>world/rows.rs"]
    D["declare(kind)<br/>a kind is a channel;<br/>past 64, an extension channel"]
    REL["World::relate_by<br/>world/relate.rs"]
    C["Phasor::couple<br/>angle → toward partner ± TURN<br/>step = max(rate, 1/(n+1))<br/>evidence += 1"]
    S["answer SLOT kind#s<br/>(16 per kind, by the answer's id)"]
    DR["domain / range channels<br/>kind and kind&gt; (PHI_READ=domain/sure)"]
    RD["World::reads<br/>nearest one TURN past the word,<br/>weighed by PHI_READ: count · agree ·<br/>band · domain · sure · typed"]
    R --> T --> D --> REL --> C
    REL --> S
    REL --> DR
    C --> RD
    S --> RD
    DR --> RD
```

## Modules, one diagram each

- [aspect](../../docs/phiecs/aspect.md)
- [condition](../../docs/phiecs/condition.md)
- [entity · node](../../docs/phiecs/node.md)
- [kit](../../docs/phiecs/kit.md)
- [kit · eye](../../docs/phiecs/eye.md)
- [persona](../../docs/phiecs/persona.md)
- [phasor](../../docs/phiecs/phasor.md)
- [phasor · facet](../../docs/phiecs/facet.md)
- [phasor · relational](../../docs/phiecs/relational.md)
- [phasor · wave](../../docs/phiecs/wave.md)
- [phasor · facet · storage](../../docs/phiecs/storage.md) · [helix](../../docs/phiecs/helix.md)
- [phasor · layers](../../docs/phiecs/layers.md) · [flow](../../docs/phiecs/flow.md) · [attention](../../docs/phiecs/attention.md) · [oscillator](../../docs/phiecs/oscillator.md)
- [prelude](../../docs/phiecs/prelude.md)
- [query](../../docs/phiecs/query.md)
- [quest](../../docs/phiecs/quest.md)
- [relation](../../docs/phiecs/relation.md)
- [resources](../../docs/phiecs/resources.md)
- [row](../../docs/phiecs/row.md)
- [scope](../../docs/phiecs/scope.md)
- [speak](../../docs/phiecs/speak.md)
- [store](../../docs/phiecs/store.md)
- [system](../../docs/phiecs/system.md)
- [word](../../docs/phiecs/word.md)
- [world](../../docs/phiecs/world.md)

Notes and tables: [docs/notes.md](../../docs/phiecs/notes.md) · What is left to do: [TODO.md](TODO.md)
