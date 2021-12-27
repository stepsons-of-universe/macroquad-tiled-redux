
This library integrates [rs-tiled](https://github.com/mapeditor/rs-tiled/issues/113) with [macroquad](https://github.com/not-fl3/macroquad/).
---

This code is adapted from `macroquad-tiled` and `macroquad::experimental::animation`.

Limitations:
* An unofficial fork of unofficial fork of `rs-tiled` is used. Very WIP.
* Only spritesheet-style tilesets are supported. But this is what you should use anyway.
* Only `Tileset` is implemented, not `Map`.

Plans:
* [ ] Implement animations.
* [ ] Implement `Map`.
* [ ] Maybe implement "image collection tilesets" (https://github.com/mapeditor/rs-tiled/issues/113)
* [ ] Find out what are these strange 0.1 [offsets in original macroquad-tiled](https://github.com/not-fl3/macroquad/blob/master/tiled/src/lib.rs#L70)
˚