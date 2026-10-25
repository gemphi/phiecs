# phikit::device — the registry, a tree of conditions found in a kernel

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle with a count; meaning is a side effect of pairs of words moving under a condition.

A port of the IOKit kernel registry (`IORegistryEntry.h`, `IOService.h`, `IOKitKeys.h` under `gemphi/phi/apple-source/ruskit/Kernel.framework`) in our words. Every piece of hardware and every driver is an `Entry` with a name, a class and a property table; entries hang from one another in `Plane`s — service, power, device tree, audio, FireWire, USB — and the same entry has a different parent under each plane. That is X ⇒ Y under c, the paradigm's shape.

| module | what |
|---|---|
| `registry` | `Registry` (`add`, `attach(child, plane, parent)`, `parents`, `children`, `path`, `roots`, `rows`), `Entry`, `Plane`, `Value` |
| `service` | `Services`: `register` (publish, match the catalogue's `Personality`s by provider class and name, probe by score, start the best as a client in the service plane), `terminate` (clients with it), `State` bits |
| `keys` | `Key` (81): the property keys in our words, with `reported()` — the string a registry prints (`IOProviderClass`, `boot-args`) — and `from_reported` |

`Registry::rows(path)` reads a registry into the kit: each entry `within` its device-tree path, `is-a` its class, `has` each key it carries, `child-of:<plane>` its parent in every plane. A real registry dump (`ioreg -l`) becomes rows this way; none is read here.
