# phikit::track — the space, frame by frame

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle with a count; meaning is a side effect of pairs of words moving under a condition.

A port of the Rust ARKit binding's `arkit` module (`anchor`, `camera`, `frame`, `session`, `raycast`, `geometry`, `mesh`, `scene_reconstruction`, `environment`, `geo`, `world`, `participants`, `collaboration`, `coaching`, `room_plan`), in our words and with the stubs made to work: a camera projects and unprojects, a raycast meets the floor, a world map survives its bytes, a session tells its events in order, a capture keeps rooms big enough.

| module | what |
|---|---|
| `anchor` | `Anchor` (id, name, transform, session, tracked), `Transform`, `Point`, `IDENTITY`, `Tracking`, `Reason`, `next_id` |
| `camera` | `Camera` with intrinsics, `project`, `unproject`, `field_of_view`; `VideoFormat`, `Orientation`, `Focus`, `Exposure`, `WhiteBalance` |
| `frame` | `Frame` (camera, anchors, light, image, depth, points, mapping, segmentation), `Tracked` — every kind of anchor — and `Kind` |
| `session` | `Session`: `run`, `update(frame)`, `add`, `remove`, `fail`, `interrupt`, `world_map`, `drain` → `Event`s; `Happening`, `RunOptions`, `SessionError` |
| `config` | `Configuration` by `Mode` (world, body, face, image, object, geo, positional, orientation), `PlaneDetection`, `WorldAlignment`, `Semantics` |
| `surface` | `Plane` (alignment, classification, status, extent, geometry), `Image` |
| `mesh` | `Mesh` (vertices, normals, texture coordinates, indices, classifications), `MeshAnchor`, `Classification`, `Reconstruction`, `Scene` |
| `depth` | `CapturedImage`, `Map<T>`, `SceneDepth`, `PointCloud`, `Segmentation`, `DepthQuality`, `DepthAccuracy` |
| `light` | `LightEstimate`, `DirectionalLight`, `Probe`, `EnvironmentTexture`, `Texturing` |
| `raycast` | `Query` (from a point or a pixel), `Result`, `Tracked`, `Target`, `TargetAlignment`, `Hit` |
| `geo` | `GeoAnchor` with `distance_to`, `GeoStatus` (`GeoState`, `GeoReason`, `GeoAccuracy`) |
| `map` | `WorldMap` (`serialize`, `deserialize`), `Mapping`, `MapError` |
| `share` | `Participant`, `Collaboration` (`Role`, `SharedState`, `Priority`), coaching `Goal` and `Coaching` state |
| `room` | `Room` (`Category`, `Dimensions`, `Surface`, `Opening`, `Object`, `Material`) with `sentence()` |
| `capture` | `Capture` (`CaptureSettings`, `Guidance`, `Export`, `CaptureError`): start, advance, stop |

Every module has a `told(path)` that tells its enumerations' cases and its structs' fields as rows under `track/<module>`; `track::told()` gathers them.
