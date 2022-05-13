//! Abstractions and functions for working with OpenStreetMap (etc.) tiles
//!
//! # Examples
//! ```
//! use slippy_map_tiles::Tile;
//!
//! let t = Tile::new(6, 35, 23).unwrap();
//!
//! ```
//!
//! You cannot create invalid tiles
//! ```
//! # use slippy_map_tiles::Tile;
//! assert!(Tile::new(0, 3, 3).is_none());
//! ```
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[cfg(feature = "world_file")]
extern crate world_image_file;

use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::ops::Deref;
use std::str::FromStr;

#[cfg(feature = "world_file")]
use world_image_file::WorldFile;

#[cfg(test)]
mod tests;

/// A single tile.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct Tile {
    zoom: u8,
    x: u32,
    y: u32,
}

impl Tile {
    /// Constucts a Tile with the following zoom, x and y values.
    ///
    /// Returns None if the x/y are invalid for that zoom level, or if the zoom is >= 100.
    /// # Examples
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// assert!(Tile::new(0, 3, 3).is_none());
    /// ```
    pub fn new(zoom: u8, x: u32, y: u32) -> Option<Tile> {
        if zoom >= 100 {
            None
        } else if x < 2u32.pow(zoom as u32) && y < 2u32.pow(zoom as u32) {
            Some(Tile {
                zoom: zoom,
                x: x,
                y: y,
            })
        } else {
            None
        }
    }

    /// zoom of this tile
    pub fn zoom(&self) -> u8 {
        self.zoom
    }

    /// X value of this tile
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Y value of tile
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Constucts a Tile with the following zoom, x and y values based on a TMS URL.
    /// Returns None if the TMS url is invalid, or those
    ///
    /// # Examples
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// let t = Tile::from_tms("/10/547/380.png");
    /// assert_eq!(t, Tile::new(10, 547, 380));
    /// assert_eq!(Tile::from_tms("foobar"), None);
    /// ```
    pub fn from_tms(tms: &str) -> Option<Tile> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "/?(?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})(\\.[a-zA-Z]{3,4})?$"
            )
            .unwrap();
        }

        let caps = RE.captures(tms)?;

        let zoom = caps.name("zoom");
        let x = caps.name("x");
        let y = caps.name("y");
        if zoom.is_none() || x.is_none() || y.is_none() {
            return None;
        }
        let zoom = zoom.unwrap().as_str().parse();
        let x = x.unwrap().as_str().parse();
        let y = y.unwrap().as_str().parse();

        if zoom.is_err() || x.is_err() || y.is_err() {
            return None;
        }
        let zoom: u8 = zoom.unwrap();
        let x: u32 = x.unwrap();
        let y: u32 = y.unwrap();

        Tile::new(zoom, x, y)
    }

    // TODO Add from_tc to parse the directory hiearchy so we can turn a filename in to a tile.
    // TODO Add from_ts to parse the directory hiearchy so we can turn a filename in to a tile.

    /// Returns the parent tile for this tile, i.e. the tile at the `zoom-1` that this tile is
    /// inside.
    ///
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// assert_eq!(Tile::new(1, 0, 0).unwrap().parent(), Tile::new(0, 0, 0));
    /// ```
    /// None if there is no parent, which is at zoom 0.
    ///
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// assert_eq!(Tile::new(0, 0, 0).unwrap().parent(), None);
    /// ```
    pub fn parent(&self) -> Option<Tile> {
        match self.zoom {
            0 => {
                // zoom 0, no parent
                None
            }
            _ => Tile::new(self.zoom - 1, self.x / 2, self.y / 2),
        }
    }

    /// Returns the subtiles (child) tiles for this tile. The 4 tiles at zoom+1 which cover this
    /// tile. Returns None if this is at the maximum permissable zoom level, and hence there are no
    /// subtiles.
    ///
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// let t = Tile::new(0, 0, 0).unwrap();
    /// let subtiles: [Tile; 4] = t.subtiles().unwrap();
    /// assert_eq!(subtiles[0], Tile::new(1, 0, 0).unwrap());
    /// assert_eq!(subtiles[1], Tile::new(1, 1, 0).unwrap());
    /// assert_eq!(subtiles[2], Tile::new(1, 0, 1).unwrap());
    /// assert_eq!(subtiles[3], Tile::new(1, 1, 1).unwrap());
    /// ```
    pub fn subtiles(&self) -> Option<[Tile; 4]> {
        match self.zoom {
            std::u8::MAX => None,
            _ => {
                let z = self.zoom + 1;
                let x = 2 * self.x;
                let y = 2 * self.y;
                Some([
                    Tile {
                        zoom: z,
                        x: x,
                        y: y,
                    },
                    Tile {
                        zoom: z,
                        x: x + 1,
                        y: y,
                    },
                    Tile {
                        zoom: z,
                        x: x,
                        y: y + 1,
                    },
                    Tile {
                        zoom: z,
                        x: x + 1,
                        y: y + 1,
                    },
                ])
            }
        }
    }

    /// Iterate on all child tiles of this tile
    pub fn all_subtiles_iter(&self) -> AllSubTilesIterator {
        AllSubTilesIterator::new_from_tile(&self)
    }

    /// Returns the LatLon for the centre of this tile.
    pub fn centre_point(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32) + 0.5, (self.y as f32) + 0.5)
    }

    /// Returns the LatLon for the centre of this tile.
    pub fn center_point(&self) -> LatLon {
        self.centre_point()
    }

    /// Returns the LatLon of the top left, i.e. north west corner, of this tile.
    pub fn nw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, self.x as f32, self.y as f32)
    }

    /// Returns the LatLon of the top right, i.e. north east corner, of this tile.
    pub fn ne_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32) + 1.0, self.y as f32)
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this tile.
    pub fn sw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, self.x as f32, (self.y as f32) + 1.0)
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this tile.
    pub fn se_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32) + 1.0, (self.y as f32) + 1.0)
    }

    pub fn top(&self) -> f32 {
        self.nw_corner().lat
    }
    pub fn bottom(&self) -> f32 {
        self.sw_corner().lat
    }
    pub fn left(&self) -> f32 {
        self.nw_corner().lon
    }
    pub fn right(&self) -> f32 {
        self.se_corner().lon
    }

    /// Returns the TC (TileCache) path for storing this tile.
    pub fn tc_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let tc = xy_to_tc(self.x, self.y);
        format!(
            "{}/{}/{}/{}/{}/{}/{}.{}",
            self.zoom, tc[0], tc[1], tc[2], tc[3], tc[4], tc[5], ext
        )
    }

    /// Returns the MP (MapProxy) path for storing this tile.
    pub fn mp_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let mp = xy_to_mp(self.x, self.y);
        format!(
            "{}/{}/{}/{}/{}.{}",
            self.zoom, mp[0], mp[1], mp[2], mp[3], ext
        )
    }

    /// Returns the TS (TileStash safe) path for storing this tile.
    pub fn ts_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let ts = xy_to_ts(self.x, self.y);
        format!(
            "{}/{}/{}/{}/{}.{}",
            self.zoom, ts[0], ts[1], ts[2], ts[3], ext
        )
    }

    /// Returns the Z/X/Y representation of this tile
    pub fn zxy(&self) -> String {
        format!("{}/{}/{}", self.zoom, self.x, self.y)
    }

    /// Returns the ZXY path for storing this tile.
    pub fn zxy_path<T: std::fmt::Display>(&self, ext: T) -> String {
        format!("{}/{}/{}.{}", self.zoom, self.x, self.y, ext)
    }

    /// Returns the ModTileMetatile path for storing this tile
    pub fn mt_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let tc = xy_to_mt(self.x, self.y);
        format!(
            "{}/{}/{}/{}/{}/{}.{}",
            self.zoom, tc[0], tc[1], tc[2], tc[3], tc[4], ext
        )
    }

    /// Returns an iterator that yields all the tiles possible, starting from `0/0/0`. Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    ///
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// let mut all_tiles_iter = Tile::all();
    /// ```
    pub fn all() -> AllTilesIterator {
        AllTilesIterator {
            next_zoom: 0,
            next_zorder: 0,
        }
    }

    /// Returns an iterator that yields all the tiles from zoom 0 down to, and including, all the
    /// tiles at `max_zoom` zoom level.  Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    pub fn all_to_zoom(max_zoom: u8) -> AllTilesToZoomIterator {
        AllTilesToZoomIterator {
            max_zoom: max_zoom,
            next_zoom: 0,
            next_x: 0,
            next_y: 0,
        }
    }

    /// The BBox for this tile.
    pub fn bbox(&self) -> BBox {
        let nw = self.nw_corner();
        let se = self.se_corner();

        BBox::new_from_points(&nw, &se)
    }

    pub fn metatile(&self, scale: u8) -> Option<Metatile> {
        Metatile::new(scale, self.zoom(), self.x(), self.y())
    }

    pub fn modtile_metatile(&self) -> Option<ModTileMetatile> {
        ModTileMetatile::new(self.zoom(), self.x(), self.y())
    }

    #[cfg(feature = "world_file")]
    /// Return the World File (in EPSG:3857 / Web Mercator SRID) for this tile
    pub fn world_file(&self) -> WorldFile {
        let total_merc_width = 20037508.342789244;
        let tile_merc_width = (2. * total_merc_width) / 2f64.powi(self.zoom as i32);
        let scale = tile_merc_width / 256.;

        WorldFile {
            x_scale: scale,
            y_scale: -scale,

            x_skew: 0.,
            y_skew: 0.,

            x_coord: tile_merc_width * (self.x as f64) - total_merc_width,
            y_coord: -tile_merc_width * (self.y as f64) + total_merc_width,
        }
    }
}

impl FromStr for Tile {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref TILE_RE: Regex =
                Regex::new("^(?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})$")
                    .unwrap();
        }

        let caps = TILE_RE.captures(s);

        if caps.is_none() {
            return Err("Tile Z/X/Y regex didn't match");
        }
        let caps = caps.unwrap();

        // If the regex matches, then none of these should fail, right?
        let zoom = caps.name("zoom").unwrap().as_str().parse().unwrap();
        let x = caps.name("x").unwrap().as_str().parse().unwrap();
        let y = caps.name("y").unwrap().as_str().parse().unwrap();

        match Tile::new(zoom, x, y) {
            None => {
                // Invalid x or y for the zoom
                Err("Invalid X or Y for this zoom")
            }
            Some(t) => Ok(t),
        }
    }
}

/// Iterates over all the tiles in the world.
pub struct AllTilesIterator {
    next_zoom: u8,
    next_zorder: u64,
}

impl Iterator for AllTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let zoom = self.next_zoom;
        let (x, y) = zorder_to_xy(self.next_zorder);
        let tile = Tile::new(zoom, x, y);

        let max_tile_no = 2u32.pow(zoom as u32) - 1;
        if x == max_tile_no && y == max_tile_no {
            // we're at the end
            self.next_zoom = zoom + 1;
            self.next_zorder = 0;
        } else {
            self.next_zorder += 1;
        }

        tile
    }
}

/// Iterates over all the tiles from 0/0/0 up to, and including, `max_zoom`.
pub struct AllTilesToZoomIterator {
    max_zoom: u8,
    next_zoom: u8,
    next_x: u32,
    next_y: u32,
}

fn remaining_in_this_zoom(next_zoom: u8, next_x: u32, next_y: u32) -> Option<usize> {
    if next_zoom == 0 && next_x == 0 && next_y == 0 {
        return Some(1);
    }

    let max_tile_no = 2u32.pow(next_zoom as u32);
    let remaining_in_column = max_tile_no - next_y;
    let remaining_in_column = remaining_in_column as usize;
    let remaining_rows = max_tile_no - next_x - 1;
    let remaining_rows = remaining_rows as usize;

    let remaining_after_this_column = remaining_rows.checked_mul(max_tile_no as usize)?;

    remaining_in_column.checked_add(remaining_after_this_column)
}

impl Iterator for AllTilesToZoomIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.next_zoom > self.max_zoom {
            return None;
        }
        let tile = Tile::new(self.next_zoom, self.next_x, self.next_y);
        let max_tile_no = 2u32.pow(self.next_zoom as u32) - 1;
        if self.next_y < max_tile_no {
            self.next_y += 1;
        } else if self.next_x < max_tile_no {
            self.next_x += 1;
            self.next_y = 0;
        } else if self.next_zoom < std::u8::MAX {
            self.next_zoom += 1;
            self.next_x = 0;
            self.next_y = 0;
        }

        tile
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.next_zoom > self.max_zoom {
            return (0, Some(0));
        }

        let remaining_in_this_level =
            remaining_in_this_zoom(self.next_zoom, self.next_x, self.next_y);
        if remaining_in_this_level.is_none() {
            return (std::usize::MAX, None);
        }
        let remaining_in_this_level = remaining_in_this_level.unwrap();

        let mut total: usize = remaining_in_this_level as usize;
        for i in (self.next_zoom + 1)..(self.max_zoom + 1) {
            let tiles_this_zoom = num_tiles_in_zoom(i);
            if tiles_this_zoom.is_none() {
                return (std::usize::MAX, None);
            }

            let tiles_this_zoom = tiles_this_zoom.unwrap();

            let new_total = total.checked_add(tiles_this_zoom);
            if new_total.is_none() {
                return (std::usize::MAX, None);
            }
            total = new_total.unwrap();
        }

        // If we've got to here, we know how big it is
        (total, Some(total))
    }
}

pub struct AllSubTilesIterator {
    _tiles: Vec<Tile>,
}

impl AllSubTilesIterator {
    pub fn new_from_tile(base_tile: &Tile) -> Self {
        let new_tiles = match base_tile.subtiles() {
            None => Vec::new(),
            Some(t) => vec![t[0], t[1], t[2], t[3]],
        };
        AllSubTilesIterator { _tiles: new_tiles }
    }
}

impl Iterator for AllSubTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self._tiles.is_empty() {
            return None;
        }
        let next = self._tiles.remove(0);
        if let Some(subtiles) = next.subtiles() {
            self._tiles.extend_from_slice(&subtiles);
        }
        Some(next)
    }
}

/// Metatiles are NxN tiles
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct Metatile {
    scale: u8,
    zoom: u8,
    x: u32,
    y: u32,
}

impl Metatile {
    pub fn new(scale: u8, zoom: u8, x: u32, y: u32) -> Option<Self> {
        if !scale.is_power_of_two() {
            return None;
        }
        if zoom >= 100 {
            None
        } else if x < 2u32.pow(zoom as u32) && y < 2u32.pow(zoom as u32) {
            let s = scale as u32;
            let x = (x / s) * s;
            let y = (y / s) * s;
            Some(Metatile {
                scale: scale,
                zoom: zoom,
                x: x,
                y: y,
            })
        } else {
            None
        }
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn zoom(&self) -> u8 {
        self.zoom
    }

    /// What is the width or height of this metatile. For small zoom numbers (e.g. z1), there will
    /// not be the full `scale` tiles across.
    pub fn size(&self) -> u8 {
        let num_tiles_in_zoom = 2u32.pow(self.zoom as u32);
        if num_tiles_in_zoom < (self.scale as u32) {
            num_tiles_in_zoom as u8
        } else {
            self.scale
        }
    }

    /// Returns the LatLon for the centre of this metatile.
    pub fn centre_point(&self) -> LatLon {
        tile_nw_lat_lon(
            self.zoom,
            (self.x as f32) + (self.size() as f32) / 2.,
            (self.y as f32) + (self.size() as f32) / 2.,
        )
    }

    /// Returns the LatLon for the centre of this metatile.
    pub fn center_point(&self) -> LatLon {
        self.centre_point()
    }

    /// Returns the LatLon of the top left, i.e. north west corner, of this metatile.
    pub fn nw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, self.x as f32, self.y as f32)
    }

    /// Returns the LatLon of the top right, i.e. north east corner, of this metatile.
    pub fn ne_corner(&self) -> LatLon {
        tile_nw_lat_lon(
            self.zoom,
            (self.x + self.size() as u32) as f32,
            self.y as f32,
        )
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this metatile.
    pub fn sw_corner(&self) -> LatLon {
        tile_nw_lat_lon(
            self.zoom,
            self.x as f32,
            (self.y + self.size() as u32) as f32,
        )
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this metatile.
    pub fn se_corner(&self) -> LatLon {
        tile_nw_lat_lon(
            self.zoom,
            (self.x + self.size() as u32) as f32,
            (self.y + self.size() as u32) as f32,
        )
    }

    /// X value of this metatile
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Y value of metatile
    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn tiles(&self) -> Vec<Tile> {
        let size = self.size() as u32;
        (0..(size * size))
            .map(|n| {
                // oh for a divmod
                let (i, j) = (n / size, n % size);
                // being cheeky and skipping the usuall Tile::new checks here, since we know it's valid
                Tile {
                    zoom: self.zoom,
                    x: self.x + i,
                    y: self.y + j,
                }
            })
            .collect()
    }

    pub fn all(scale: u8) -> MetatilesIterator {
        assert!(scale.is_power_of_two());
        MetatilesIterator::all(scale)
    }
}

impl FromStr for Metatile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref METATILE_RE: Regex = Regex::new(
                "^(?P<scale>[0-9]+) (?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})$"
            )
            .unwrap();
        }

        let caps = METATILE_RE.captures(s);

        if caps.is_none() {
            return Err(());
        }
        let caps = caps.unwrap();

        // If the regex matches, then none of these should fail, right?
        let scale = caps.name("scale").unwrap().as_str().parse().unwrap();
        let zoom = caps.name("zoom").unwrap().as_str().parse().unwrap();
        let x = caps.name("x").unwrap().as_str().parse().unwrap();
        let y = caps.name("y").unwrap().as_str().parse().unwrap();

        match Metatile::new(scale, zoom, x, y) {
            None => {
                // Invalid x or y for the zoom
                Err(())
            }
            Some(mt) => Ok(mt),
        }
    }
}

/// Iterates over all the metatiles in the world.
#[derive(Debug)]
pub struct MetatilesIterator {
    scale: u8,
    curr_zoom: u8,
    maxzoom: u8,
    curr_zorder: u64,
    bbox: Option<BBox>,

    // In metatile coords, i.e. x/scale
    curr_zoom_width_height: Option<(u32, u32)>,
    curr_zoom_start_xy: Option<(u32, u32)>,

    // If we're reading from a file
    total: Option<usize>,
    tile_list_file: Option<BufReader<File>>,
}

impl MetatilesIterator {
    pub fn all(scale: u8) -> Self {
        MetatilesIterator {
            scale: scale,
            curr_zoom: 0,
            curr_zorder: 0,
            bbox: None,
            maxzoom: 32,
            curr_zoom_width_height: None,
            curr_zoom_start_xy: None,
            total: None,
            tile_list_file: None,
        }
    }

    pub fn new_for_bbox(scale: u8, bbox: &BBox) -> Self {
        MetatilesIterator::new_for_bbox_zoom(scale, &Some(bbox.clone()), 0, 32)
    }

    /// `None` for bbox means 'whole world'
    pub fn new_for_bbox_zoom(scale: u8, bbox: &Option<BBox>, minzoom: u8, maxzoom: u8) -> Self {
        let mut it = MetatilesIterator {
            scale: scale,
            curr_zoom: minzoom,
            curr_zorder: 0,
            bbox: bbox.clone(),
            maxzoom: maxzoom,
            curr_zoom_width_height: None,
            curr_zoom_start_xy: None,
            total: None,
            tile_list_file: None,
        };
        it.set_zoom_width_height();
        it.set_zoom_start_xy();

        it
    }

    pub fn new_from_filelist(filename: String) -> Self {
        let mut file = BufReader::new(File::open(&filename).unwrap());
        file.seek(SeekFrom::Start(0)).unwrap();

        // we're intentionally ignore usize overflow. If you have that many lines in a file,
        // you're probably doing something wrong.
        let total = file.lines().count();

        let file = BufReader::new(File::open(filename).unwrap());

        MetatilesIterator {
            scale: 0,
            curr_zoom: 0,
            curr_zorder: 0,
            bbox: None,
            maxzoom: 0,
            curr_zoom_width_height: None,
            curr_zoom_start_xy: None,
            total: Some(total),
            tile_list_file: Some(file),
        }
    }

    /// Update the `self.curr_zoom_width_height` variable with the correct value for this zoom
    /// (`self.curr_zoom`)
    fn set_zoom_width_height(&mut self) {
        if let Some(ref bbox) = self.bbox {
            let scale = self.scale as u32;
            let zoom = self.curr_zoom;
            // TODO is this x/y lat/lon the right way around?
            let (x1, y1) = lat_lon_to_tile(bbox.top, bbox.left, zoom);
            let (x1, y1) = (x1 / scale, y1 / scale);
            let (x2, y2) = lat_lon_to_tile(bbox.bottom, bbox.right, zoom);
            let (x2, y2) = (x2 / scale, y2 / scale);

            let width = x2 - x1 + 1;
            let height = y2 - y1 + 1;

            self.curr_zoom_width_height = Some((width, height));
        }
    }

    fn set_zoom_start_xy(&mut self) {
        if self.bbox.is_none() {
            return;
        }

        let top = match self.bbox {
            None => 90.,
            Some(ref b) => b.top,
        };
        let left = match self.bbox {
            None => -180.,
            Some(ref b) => b.left,
        };
        // TODO is this x/y lat/lon the right way around?
        let (x1, y1) = lat_lon_to_tile(top, left, self.curr_zoom);
        self.curr_zoom_start_xy = Some((x1 / self.scale as u32, y1 / self.scale as u32));
    }

    fn next_from_zorder(&mut self) -> Option<Metatile> {
        // have to set a value, but we're never going to read it
        #[allow(unused_assignments)]
        let mut zoom = 0;
        #[allow(unused_assignments)]
        let mut x = 0;
        #[allow(unused_assignments)]
        let mut y = 0;

        let scale = self.scale as u32;

        loop {
            if self.curr_zoom > self.maxzoom {
                // We're finished
                return None;
            }

            zoom = self.curr_zoom;
            let (width, height) = match self.curr_zoom_width_height {
                None => {
                    let max_num = 2u32.pow(zoom as u32);
                    let mut max = max_num / scale;
                    if max_num % scale > 0 {
                        max += 1
                    }
                    (max, max)
                }
                Some((width, height)) => (width, height),
            };

            let max_zorder_for_zoom = xy_to_zorder(width - 1, height - 1);

            let (i, j) = zorder_to_xy(self.curr_zorder);
            let bits = match self.curr_zoom_start_xy {
                None => (i, j),
                Some(start) => (start.0 + i, start.1 + j),
            };
            x = bits.0;
            y = bits.1;

            if self.curr_zorder > max_zorder_for_zoom {
                // Next zoom
                // we're at the end
                self.curr_zoom = zoom + 1;
                self.curr_zorder = 0;
                self.set_zoom_start_xy();
                self.set_zoom_width_height();
            } else if i > width || j > height {
                // If the bbox is non-square, there will be X (or Y) tiles which are outside
                // the bbox. Rather than go to the next zoom level, we want to contine to look at
                // the next tile in order, and keep going until we get a tile that's inside the
                // bbox.  to the next tile
                self.curr_zorder += 1;
                continue;
            } else {
                // This z order is OK
                self.curr_zorder += 1;
                break;
            }
        }

        let (x, y) = (x * scale, y * scale);
        Metatile::new(self.scale, zoom, x, y)
    }

    fn next_from_file(&mut self) -> Option<Metatile> {
        let mut s = String::new();
        if let Some(ref mut file) = self.tile_list_file {
            file.read_line(&mut s).unwrap();
        }
        // remove trailing newline
        let s = s.trim_end();

        s.parse().ok()
    }

    pub fn total(&self) -> Option<usize> {
        self.total
    }
}

impl Iterator for MetatilesIterator {
    type Item = Metatile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tile_list_file.is_some() {
            self.next_from_file()
        } else {
            self.next_from_zorder()
        }
    }
}

/// Metatiles as found by mod_tile, always 8x8
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ModTileMetatile {
    inner: Metatile,
}

impl ModTileMetatile {
    pub fn new(zoom: u8, x: u32, y: u32) -> Option<Self> {
        match Metatile::new(8, zoom, x, y) {
            None => None,
            Some(inner) => Some(ModTileMetatile { inner: inner }),
        }
    }

    /// Returns the mod_tile path for storing this metatile
    pub fn path<T: std::fmt::Display>(&self, ext: T) -> String {
        let mt = xy_to_mt(self.inner.x, self.inner.y);
        format!(
            "{}/{}/{}/{}/{}/{}.{}",
            self.inner.zoom, mt[0], mt[1], mt[2], mt[3], mt[4], ext
        )
    }

    /// X value of this metatile
    pub fn x(&self) -> u32 {
        self.inner.x
    }

    /// Y value of metatile
    pub fn y(&self) -> u32 {
        self.inner.y
    }

    /// Zoom value of metatile
    pub fn zoom(&self) -> u8 {
        self.inner.zoom
    }

    /// What is the width or height of this metatile. For small zoom numbers (e.g. z1), there will
    /// not be the full `scale` tiles across.
    pub fn size(self) -> u8 {
        self.inner.size()
    }
}

impl From<ModTileMetatile> for Metatile {
    fn from(mt: ModTileMetatile) -> Self {
        mt.inner
    }
}

impl TryFrom<Metatile> for ModTileMetatile {
    type Error = &'static str;

    fn try_from(mt: Metatile) -> Result<Self, Self::Error> {
        if mt.scale == 8 {
            Ok(ModTileMetatile { inner: mt })
        } else {
            Err("Can only convert scale 8 metatiles into ModTileMetatile")
        }
    }
}

impl Borrow<Metatile> for ModTileMetatile {
    fn borrow(&self) -> &Metatile {
        &self.inner
    }
}

impl Deref for ModTileMetatile {
    type Target = Metatile;

    fn deref(&self) -> &Metatile {
        &self.inner
    }
}

fn tile_nw_lat_lon(zoom: u8, x: f32, y: f32) -> LatLon {
    let n: f32 = 2f32.powi(zoom as i32);
    let lon_deg: f32 = (x as f32) / n * 360f32 - 180f32;
    let lat_rad: f32 = ((1f32 - 2f32 * (y as f32) / n) * std::f32::consts::PI)
        .sinh()
        .atan();
    let lat_deg: f32 = lat_rad * 180f32 * std::f32::consts::FRAC_1_PI;

    // FIXME figure out the unwrapping here....
    // Do we always know it's valid?
    LatLon::new(lat_deg, lon_deg).unwrap()
}

/// Return the x,y of a tile which has this lat/lon for this zoom level
pub fn lat_lon_to_tile(lat: f32, lon: f32, zoom: u8) -> (u32, u32) {
    // TODO do this at compile time?
    #[allow(non_snake_case)]
    let MAX_LAT: f64 = std::f64::consts::PI.sinh().atan();

    let lat: f64 = lat as f64;
    let lat = lat.to_radians();

    let lon: f64 = lon as f64;

    // Clip the latitude to the max & min (~85.0511)
    let lat = if lat > MAX_LAT {
        MAX_LAT
    } else if lat < -MAX_LAT {
        -MAX_LAT
    } else {
        lat
    };

    let n: f64 = 2f64.powi(zoom as i32);
    let xtile: u32 = (n * ((lon + 180.) / 360.)).trunc() as u32;
    let ytile: u32 = (n * (1. - ((lat.tan() + (1. / lat.cos())).ln() / std::f64::consts::PI)) / 2.)
        .trunc() as u32;

    (xtile, ytile)
}

/// Return the x,y of a tile which (for this zoom) has this web mercator 3857 x/y, and then the x,y
/// of the pixel within that image (presuming a 256x256 image)
pub fn merc_location_to_tile_coords(x: f64, y: f64, zoom: u8) -> ((u32, u32), (u32, u32)) {
    let num_tiles = 2u32.pow(zoom as u32) as f64;
    let global_extent = 20_037_508.342789244;
    let tile_width = (2. * global_extent) / num_tiles;

    (
        // location within the tile
        (
            ((x + global_extent) / tile_width) as u32,
            ((y + global_extent) / tile_width) as u32,
        ),
        // Tile x/y
        (
            (((x + global_extent) % tile_width) / tile_width * 256.) as u32,
            (num_tiles - ((y + global_extent) % tile_width) / tile_width * 256. - 1.) as u32,
        ),
    )
}

/// How many tiles does this bbox cover at this zoom
/// If there is an overflow for usize, `None` is returned, if not, a `Some(...)`
pub fn size_bbox_zoom(bbox: &BBox, zoom: u8) -> Option<usize> {
    let top_left_tile = lat_lon_to_tile(bbox.top(), bbox.left(), zoom);
    let bottom_right_tile = lat_lon_to_tile(bbox.bottom(), bbox.right(), zoom);
    let height = (bottom_right_tile.0 - top_left_tile.0) as usize + 1;
    let width = (bottom_right_tile.1 - top_left_tile.1) as usize + 1;

    height.checked_mul(width)
}

/// How many metatiles, of this scale, does this bbox cover at this zoom
/// If there is an overflow for usize, `None` is returned, if not, a `Some(...)`
/// This is less likely to overflow than `size_bbox_zoom` because metatiles are larger
pub fn size_bbox_zoom_metatiles(bbox: &BBox, zoom: u8, metatile_scale: u8) -> Option<usize> {
    let metatile_scale = metatile_scale as u32;
    let top_left_tile = lat_lon_to_tile(bbox.top(), bbox.left(), zoom);
    let bottom_right_tile = lat_lon_to_tile(bbox.bottom(), bbox.right(), zoom);
    let bottom = (bottom_right_tile.0 / metatile_scale) * metatile_scale;
    let top = (top_left_tile.0 / metatile_scale) * metatile_scale;
    let left = (top_left_tile.1 / metatile_scale) * metatile_scale;
    let right = (bottom_right_tile.1 / metatile_scale) * metatile_scale;

    let height = ((bottom - top) / metatile_scale as u32) as usize + 1;
    let width = ((right - left) / metatile_scale as u32) as usize + 1;

    height.checked_mul(width)
}

/// A single point in the world.
///
/// Since OSM uses up to 7 decimal places, this stores the lat/lon as `f32` which is enough
/// precision of that
#[derive(PartialEq, Debug, Clone)]
pub struct LatLon {
    lat: f32,
    lon: f32,
}

impl LatLon {
    /// Constructs a LatLon from a given `lat` and `lon`. Returns `None` if the lat or lon is
    /// invalid, e.g. a lat of 100.
    pub fn new(lat: f32, lon: f32) -> Option<LatLon> {
        if (-90f32 ..= 90f32).contains(&lat) && (-180f32 ..= 180.).contains(&lon) {
            Some(LatLon { lat: lat, lon: lon })
        } else {
            None
        }
    }

    /// Latitude
    pub fn lat(&self) -> f32 {
        self.lat
    }
    /// Longitude
    pub fn lon(&self) -> f32 {
        self.lon
    }

    /// Convert to Web Mercator format (SRID 3857)
    pub fn to_3857(&self) -> (f32, f32) {
        let x = self.lon() * 20037508.34 / 180.;
        let pi = std::f32::consts::PI;
        let y = ((90. + self.lat()) * pi / 360.).tan().ln() / (pi / 180.);
        let y = y * 20037508.34 / 180.;

        (x, y)
    }

    /// What tile is this point at on this zoom level
    pub fn tile(&self, zoom: u8) -> Tile {
        let (x, y) = lat_lon_to_tile(self.lat, self.lon, zoom);
        Tile::new(zoom, x, y).unwrap()
    }
}

/// A Bounding box
#[derive(PartialEq, Debug, Clone)]
pub struct BBox {
    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
}

impl BBox {
    /// Construct a new BBox from the given max and min latitude and longitude. Returns `None` if
    /// the lat or lon is invalid, e.g. a lon of 200
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Option<BBox> {
        //let top = if top > bottom { top } else { bottom };
        //let bottom = if top > bottom { bottom } else { top };
        //let left = if right > left { left } else { right };
        //let right = if right > left { right } else { left };

        if  (-90. ..=90.).contains(&top)
          &&(-90. ..=90.).contains(&bottom) 
          &&(-180. ..=180.).contains(&left) 
          &&(-180. ..=180.).contains(&right) 
        {
            Some(BBox {
                top,
                left,
                bottom,
                right,
            })
        } else {
            None
        }
    }

    /// Given two points, return the bounding box specified by those 2 points
    pub fn new_from_points(topleft: &LatLon, bottomright: &LatLon) -> BBox {
        BBox {
            top: topleft.lat,
            left: topleft.lon,
            bottom: bottomright.lat,
            right: bottomright.lon,
        }
    }

    /// Construct a BBox from a tile
    pub fn new_from_tile(tile: &Tile) -> Self {
        tile.bbox()
    }

    /// Return true iff this point is in this bbox
    pub fn contains_point(&self, point: &LatLon) -> bool {
        point.lat <= self.top
            && point.lat > self.bottom
            && point.lon >= self.left
            && point.lon < self.right
    }

    /// Returns true iff this bbox and `other` share at least one point
    pub fn overlaps_bbox(&self, other: &BBox) -> bool {
        // FXME check top & left edges
        self.left < other.right
            && self.right > other.left
            && self.top > other.bottom
            && self.bottom < other.top
    }

    /// Iterate over all the tiles from z0 onwards that this bbox is in
    pub fn tiles(&self) -> BBoxTilesIterator {
        BBoxTilesIterator::new(self)
    }

    /// Iterate over all the metatiles from z0 onwards that this bbox is in
    pub fn metatiles(&self, scale: u8) -> MetatilesIterator {
        let bbox: BBox = (*self).clone();
        MetatilesIterator {
            curr_zoom: 0,
            maxzoom: 32,
            bbox: Some(bbox),
            curr_zorder: 0,
            scale: scale,
            curr_zoom_width_height: None,
            curr_zoom_start_xy: None,
            total: None,
            tile_list_file: None,
        }
    }

    /// Return the top value of this bbox
    pub fn top(&self) -> f32 {
        self.top
    }

    /// Return the bottom value of this bbox
    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    /// Return the left value of this bbox
    pub fn left(&self) -> f32 {
        self.left
    }

    /// Return the right value of this bbox
    pub fn right(&self) -> f32 {
        self.right
    }

    /// For this zoom level, return all the tiles that cover this bbox
    pub fn tiles_for_zoom(&self, zoom: u8) -> impl Iterator<Item = Tile> {
        let top_left_tile = lat_lon_to_tile(self.top, self.left, zoom);
        let bottom_right_tile = lat_lon_to_tile(self.bottom, self.right, zoom);

        (top_left_tile.0..=bottom_right_tile.0)
            .flat_map(move |x| {
                (top_left_tile.1..=bottom_right_tile.1)
                    .map(move |y| (x, y))
            })
            .map(move |(x, y)| Tile::new(zoom, x, y).unwrap())
    }

    /// Returns the LatLon for the centre of this bbox
    pub fn centre_point(&self) -> LatLon {
        LatLon::new((self.top + self.bottom) / 2., (self.left + self.right) / 2.).unwrap()
    }

    /// Returns the LatLon for the centre of this bbox
    pub fn center_point(&self) -> LatLon {
        self.centre_point()
    }

    /// Returns the LatLon of the top left, i.e. north west corner, of this bbot
    pub fn nw_corner(&self) -> LatLon {
        LatLon::new(self.top, self.left).unwrap()
    }

    /// Returns the LatLon of the top right, i.e. north east corner, of this bbox
    pub fn ne_corner(&self) -> LatLon {
        LatLon::new(self.top, self.right).unwrap()
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this bbox
    pub fn sw_corner(&self) -> LatLon {
        LatLon::new(self.bottom, self.left).unwrap()
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this bbox.
    pub fn se_corner(&self) -> LatLon {
        LatLon::new(self.bottom, self.right).unwrap()
    }
}

impl FromStr for BBox {
    type Err = &'static str;

    /// Given a string like "$MINLON $MINLAT $MAXLON $MAXLAT" parse that into a BBox. Returns None
    /// if there is no match.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            //static ref num_regex: &'static str = r"-?[0-9]{1,3}(\.[0-9]{1,10})?";
            static ref SIMPLE_COPY_SPACE: Regex = Regex::new(r"^(?P<minlon>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<minlat>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<maxlon>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<maxlat>-?[0-9]{1,3}(\.[0-9]{1,10})?)$").unwrap();
            static ref SIMPLE_COPY_COMMA: Regex = Regex::new(r"^(?P<minlon>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<minlat>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<maxlon>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<maxlat>-?[0-9]{1,3}(\.[0-9]{1,10})?)$").unwrap();
        }
        let caps = SIMPLE_COPY_SPACE
            .captures(string)
            .or_else(|| SIMPLE_COPY_COMMA.captures(string));
        if caps.is_none() {
            return Err("regex not match");
        }
        let caps = caps.unwrap();

        let minlat = caps.name("minlat");
        let maxlat = caps.name("maxlat");
        let minlon = caps.name("minlon");
        let maxlon = caps.name("maxlon");

        if minlat.is_none() || maxlat.is_none() || minlon.is_none() || maxlon.is_none() {
            return Err("bad lat/lon");
        }

        let minlat = minlat.unwrap().as_str().parse();
        let maxlat = maxlat.unwrap().as_str().parse();
        let minlon = minlon.unwrap().as_str().parse();
        let maxlon = maxlon.unwrap().as_str().parse();

        if minlat.is_err() || maxlat.is_err() || minlon.is_err() || maxlon.is_err() {
            return Err("bad lat/lon");
        }

        let minlat = minlat.unwrap();
        let maxlat = maxlat.unwrap();
        let minlon = minlon.unwrap();
        let maxlon = maxlon.unwrap();

        BBox::new(maxlat, minlon, minlat, maxlon).ok_or("bad lat/lon")
    }
}

pub struct BBoxTilesIterator<'a> {
    bbox: &'a BBox,
    tiles: Vec<Tile>,
    tile_index: usize,
}

impl<'a> BBoxTilesIterator<'a> {
    pub fn new(bbox: &'a BBox) -> BBoxTilesIterator<'a> {
        // Everything is in 0/0/0, so start with that.
        BBoxTilesIterator {
            bbox: bbox,
            tiles: vec![Tile::new(0, 0, 0).unwrap()],
            tile_index: 0,
        }
    }
}

impl<'a> Iterator for BBoxTilesIterator<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.tile_index >= self.tiles.len() {
            // We've sent off all the existing tiles, so start looking at the children
            let mut new_tiles: Vec<Tile> = Vec::with_capacity(self.tiles.len() * 4);
            for t in self.tiles.iter() {
                match t.subtiles() {
                    None => {}
                    Some(sub) => {
                        if self.bbox.overlaps_bbox(&sub[0].bbox()) {
                            new_tiles.push(sub[0]);
                        }
                        if self.bbox.overlaps_bbox(&sub[1].bbox()) {
                            new_tiles.push(sub[1]);
                        }
                        if self.bbox.overlaps_bbox(&sub[2].bbox()) {
                            new_tiles.push(sub[2]);
                        }
                        if self.bbox.overlaps_bbox(&sub[3].bbox()) {
                            new_tiles.push(sub[3]);
                        }
                    }
                }
            }

            new_tiles.shrink_to_fit();
            self.tiles = new_tiles;
            self.tile_index = 0;
        }

        let tile = self.tiles[self.tile_index];
        self.tile_index += 1;
        Some(tile)
    }
}

/// Convert x & y to a TileCache (tc) directory parts
fn xy_to_tc(x: u32, y: u32) -> [String; 6] {
    [
        format!("{:03}", x / 1_000_000),
        format!("{:03}", (x / 1_000) % 1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y / 1_000_000),
        format!("{:03}", (y / 1_000) % 1_000),
        format!("{:03}", y % 1_000),
    ]
}

/// Convert x & y to a MapProxy (mp) directory parts
fn xy_to_mp(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:04}", x / 10_000),
        format!("{:04}", x % 10_000),
        format!("{:04}", y / 10_000),
        format!("{:04}", y % 10_000),
    ]
}

/// Convert x & y to a TileStash (ts) safe directory parts
fn xy_to_ts(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:03}", x / 1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y / 1_000),
        format!("{:03}", y % 1_000),
    ]
}

/// Convert x & y to a ModTile metatile directory parts
fn xy_to_mt(x: u32, y: u32) -> [String; 5] {
    // /[Z]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy].png
    // i.e. /[Z]/a/b/c/d/e.png

    let mut x = x;
    let mut y = y;

    let e = (((x & 0x0f) << 4) | (y & 0x0f)) as u8;
    x >>= 4;
    y >>= 4;

    let d = (((x & 0x0f) << 4) | (y & 0x0f)) as u8;
    x >>= 4;
    y >>= 4;

    let c = (((x & 0b000_1111_u32) << 4) | (y & 0b000_1111_u32)) as u8;
    x >>= 4;
    y >>= 4;

    let b = (((x & 0b000_1111_u32) << 4) | (y & 0b000_1111_u32)) as u8;
    x >>= 4;
    y >>= 4;

    let a = (((x & 0b000_1111_u32) << 4) | (y & 0b000_1111_u32)) as u8;
    //x >>= 4;
    //y >>= 4;

    [
        format!("{}", a),
        format!("{}", b),
        format!("{}", c),
        format!("{}", d),
        format!("{}", e),
    ]
}

/// How many times are in this soom level? Returns None if there would be a usize overflow
fn num_tiles_in_zoom(zoom: u8) -> Option<usize> {
    // From experience it looks like you can't calc above zoom >= 6
    if zoom == 0 {
        // Special case of known value
        Some(1)
    } else if zoom <= 5 {
        Some(2u64.pow(2u32.pow(zoom as u32)) as usize)
    } else {
        None
    }
}

pub fn xy_to_zorder(x: u32, y: u32) -> u64 {
    let mut res: u64 = 0;
    for i in 0..32 {
        let x_set: bool = (x >> i) & 1 == 1;
        let y_set: bool = (y >> i) & 1 == 1;
        if x_set {
            res |= 1 << (i * 2);
        }
        if y_set {
            res |= 1 << (i * 2) + 1;
        }
    }

    res
}

pub fn zorder_to_xy(zorder: u64) -> (u32, u32) {
    let mut x: u32 = 0;
    let mut y: u32 = 0;

    for i in 0..32 {
        let x_bit_set = (zorder >> (i * 2)) & 1 == 1;
        let y_bit_set = (zorder >> ((i * 2) + 1)) & 1 == 1;

        if x_bit_set {
            x |= 1 << i;
        }
        if y_bit_set {
            y |= 1 << i;
        }
    }

    (x, y)
}
