
Macroquad-tiled-redux
===

This library integrates [rs-tiled](https://github.com/mapeditor/rs-tiled/)
with [macroquad](https://github.com/not-fl3/macroquad/).

This code is adapted from `macroquad-tiled` and `macroquad::experimental::animation`.

How to use
---

See [examples](./examples)

Limitations
---

* An unofficial fork of unofficial fork of `rs-tiled` is used. Very WIP.
* Only 2d orthogonal spritesheet-style tilesets are supported. But this is what you should use anyway.
* Only `Tileset` is implemented, not `Map`.

Plans:
* [x] Implement animations.
* [ ] Implement `<wangsets>`: https://doc.mapeditor.org/en/stable/manual/terrain/
* [ ] Implement `Map`.
* [ ] Implement `tile.terrain` and `tile.probability`.
* [ ] Find out what are these strange 0.1px [offsets in original macroquad-tiled](https://github.com/not-fl3/macroquad/blob/master/tiled/src/lib.rs#L70)

Non-plans yet:
* [ ] Parallelize `rs-tiled` parser
* [ ] "image collection tilesets" (https://github.com/mapeditor/rs-tiled/issues/113)
* [ ] Isometric maps
* [ ] Staggered maps
* [ ] Hexagonal maps
