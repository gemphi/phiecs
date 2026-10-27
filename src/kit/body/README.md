# phikit::body — what a device can see of a person

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle; meaning is a side effect of pairs of words moving under a condition, and they move by the error of what was expected — toward what came, away from what did not.

The body is named the way the devices name it, in our words, and every frame of it is a sentence.

| module | what | ported from |
|---|---|---|
| `shape` | `Shape` (52), `Region` (its first word), `Side`, `Coefficients` | ARKit's blend-shape locations |
| `face` | `Face`: anchor, coefficients, geometry, each eye's transform, gaze; `said(tau)`, `sentence(tau)`; `rows(path, frames, tau)` tells a sequence as a paragraph | `ARFaceAnchor` |
| `pose` | `Joint` (23: nineteen planar, four spatial), `Group` read from the words, `Located`, `Pose` with `said(group, tau)` | Vision's body-pose joint names and groups |
| `skeleton` | `SkeletonJoint` (67), `Bone` (local and model transform, parent, confidence), `Skeleton` with `chain` to the root, `Body` | the binding's body skeleton |
| `hand` | `Finger`, `Chirality`, `HandJoint` (21, root to tip per finger), `HandSkeletonJoint` (27, with `parent` along the finger), `Hand`, `Skeleton` | Vision's hand-pose joints; the visionOS hand skeleton |

A frame says what it is sure of: `Face::sentence(0.3)` is `mouth smile left, mouth smile right, cheek squint left`; a frame saying nothing is `⟨rest⟩`. Told three times under one scene and asked what follows the smile, `phiplm::prose::Knowing` answers `eye` — the blink it was told (test `a_face_is_read_and_gone_on_from`).

Ties are real: `left elbow` is the pose's and the skeleton's, `thumb tip` the hand's and the skeleton's hands'; the module's word (`pose left elbow`, `skeleton left elbow`) or a word only one has (`thumb mp`, `index finger knuckle`, `left hand thumb tip`) settles a cast.
