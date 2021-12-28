
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
* Only 2d orthogonal spritesheet-style tilesets are supported.
* Only `Tileset` is implemented, not `Map`.

Plans:
* [x] Implement animations.
* [x] Implement `Map`.
* [ ] Animate `Map`.
* [ ] Clean up missing features in `Map`.
* [ ] Implement `<wangsets>`: https://doc.mapeditor.org/en/stable/manual/terrain/
* [ ] Implement all `rs-tiled` styles of constructors for `TileSet` and `Map`: from file/reader/str.
* [ ] Implement `tile.terrain` and `tile.probability`.
* [x] Find out what are these 1.0px and 0.1px [offsets in original macroquad-tiled](https://github.com/not-fl3/macroquad/blob/master/tiled/src/lib.rs#L70) - probably nothing.

Non-plans yet:
* [ ] Parallelize `rs-tiled` parser
* [ ] "image collection tilesets" (https://github.com/mapeditor/rs-tiled/issues/113)
* [ ] Isometric maps
* [ ] Staggered maps
* [ ] Hexagonal maps
