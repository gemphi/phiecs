# phiecs — the Entity Component System and the phasor engine

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle with a count; meaning is a side effect of pairs of words moving under a condition.

Everything told to this world is one shape: a `Row { about, kind, other, weight }`, and `World::tell` is the one loop. Nothing learns from counts here — the engine measures the DIFFERENCE under conditions and moves phasors: telling `a next b` couples `a` toward one TURN before `b` on the `next` channel, sequentially and directed, with a step of `max(rate, 1/(evidence+1))` so one-shot facts are readable at once. Reading (`World::reads`) comes FROM the phasors: the words nearest one TURN past the asked word on the kind's channel, scored by phase agreement times √amplitude.

A condition is a CHANNEL: beyond the 64 base kinds, extension channels open lazily at seeded angles, so a construction family ("have been", "does not") becomes its own channel the moment it is named. An answer has a SLOT: 16 answer slots per kind, a word read on a slot only where it has itself spoken, the plain channel only when no slot spoke. Source evidence (`spoken`) is kept apart from sink evidence, so the world cannot fabricate speakers out of words that were only spoken about.

`quest/` carries the oracle seam (`Spawned::ollama(model)` on a machine that has it; `Scripted` in the sandbox), so a student in this world can ask a teacher for more uses of what it is unsure of.

Measured (the record is [phiano](https://github.com/gemphi/phiano)'s docs/186–194): this engine leads the arena on glosses, 32.9% next-word vs counts' 29.9%, and still trails counts on prose, 13.9% vs 21.4% — stated straight, both numbers.

Part of the [phinet](https://github.com/gemphi/phinet) workspace. MIT OR Apache-2.0.
