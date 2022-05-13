## Unreleased

* Internal refactoring
* `slippy_map_tiles::all_tiles()`

<a name="v0.16.0"></a>
## v0.16.0 (2018-07-20)

* `ModTileMetatiles` can be deref'ed to `Metatile`
* Internal refactoring, formatting & lints
* Updated dependencies

<a name="v0.15.0"></a>
## v0.15.0 (2018-07-20)

### Breaking Changes

* `BBox::new_from_string` has been replaced with a `FromStr` implemention

### New Features

* `merc_location_to_tile_coords` in which tile is a 3857/web mercator point
* `world_file` optional feature, and `Tile::world_file()` to generate a [world
  file](https://en.wikipedia.org/wiki/World_file) to georeference this tile.

<a name="v0.14.0"></a>
## v0.14.0 (2018-02-28)


#### Features

*   Can read MetaTile's from a file and iterate over that ([6bcbc0ee](6bcbc0ee))
*   Convience .x/y/scale methods to MetaTile's ([9bfc466b](9bfc466b))
*   Add FromStr for Metatile (in "scale Z/X/Y") format ([a300aa97](a300aa97))
*   Add FromStr for Tile (in "Z/X/Y" format) ([b7256fe1](b7256fe1))



<a name="v0.13.0"></a>
## v0.13.0 (2018-02-18)


#### Bug Fixes

*   Bugs with previous MetatileIterator returning too many tiles for a bbox ([42715d66](42715d66))

#### Features

*   BBox fix ups ([fed9db10](fed9db10))



<a name="v0.12.0"></a>
## v0.12.0 (2018-02-10)


#### Bug Fixes

*   Clip lattitude ([526c3cd8](526c3cd8))

#### Features

*   BBox fix ups ([cf21cbcb](cf21cbcb))
*   Easy way to get a generic metatile into modtilemetatile ([d70d2d58](d70d2d58))
*   Add .zxy() method to quickly get this string rep ([1dc51302](1dc51302))



<a name="v0.11.0"></a>
## v0.11.0 (2017-04-12)


#### Features

*   Tiles are generated in Z order now ([05521c61](05521c61))



<a name="v0.10.0"></a>
## v0.10.0 (2017-02-10)


#### Bug Fixes

*   Set LatLon costructor public ([23553c53](23553c53))

#### Features

*   Support simple Z/X/Y paths ([d20635a0](d20635a0))



<a name="v0.9.0"></a>
## v0.9.0 (2016-12-17)


#### Features

*   Iterate on all child tiles ([b2ce8759](b2ce8759))
*   Tiles can be hashed. Can use as keys in Hashmap ([3ad16113](3ad16113))



<a name="v0.8.0"></a>
## v0.8.0 (2016-09-28)


#### Features

*   Add a TileStash safe output ([48f7d138](48f7d138))

<a name="v0.7.0"></a>
## v0.7.0 (2016-06-14)


#### Features

*   Change licence to GNU GPL v3 (or later)



<a name="v0.6.0"></a>
## v0.6.0 (2016-03-18)


#### Bug Fixes

*   Correct BBox::new_from_string order it's TLBR ([29db0639](29db0639))

#### Features

*   Add BBox::new_from_string ([7553bd1e](7553bd1e))
*   LatLon now have .lat()/.lon() as getters ([e64472a8](e64472a8))
*   Tile::from_tms can now take a full URL ([c2ccdbba](c2ccdbba))



<a name="v0.5.0"></a>
## v0.5.0 (2016-03-17)


#### Features

*   Regex for TMS parsing now statically created ([ce9fa5f4](ce9fa5f4))



<a name="v0.4.0"></a>
## v0.4.0 (2016-03-07)


#### Bug Fixes

*   Make num_tiles_in_zoom more robust & return Option ([d340bb0d](d340bb0d))

#### Features

*   Add size_hint to AllTilesToZoomIterator ([048632b2](048632b2))
*   Add iterator that stops after a certain zoom ([621454e6](621454e6))
*   Add num_tiles_in_zoom function ([b8890c28](b8890c28))



<a name="v0.3.1"></a>
### v0.3.1 (2016-03-07)


#### Bug Fixes

*   Make Tile.{zoom/x/y} pub so we can access them ([6dca0ac3](6dca0ac3))



<a name="v0.3.0"></a>
## v0.3.0 (2016-03-03)


#### Bug Fixes

*   Reject zoom >= 100 ([1127a422](1127a422))

#### Features

*   Add Tile::from_tms to parse a TMS string ([1bf8accb](1bf8accb))



