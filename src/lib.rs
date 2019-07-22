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
#[macro_use] extern crate lazy_static;
extern crate regex;

#[cfg(feature="world_file")]
extern crate world_image_file;

use regex::Regex;
use std::str::FromStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

#[cfg(feature="world_file")]
use world_image_file::WorldFile;

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
            Some(Tile { zoom: zoom, x: x, y: y })
        } else {
            None
        }
    }

    /// zoom of this tile
    pub fn zoom(&self) -> u8 { self.zoom }

    /// X value of this tile
    pub fn x(&self) -> u32 { self.x }

    /// Y value of tile
    pub fn y(&self) -> u32 { self.y }

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
            static ref RE: Regex = Regex::new("/?(?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})(\\.[a-zA-Z]{3,4})?$").unwrap();
        }

        let caps = RE.captures(tms);
        if caps.is_none() {
            return None;
        }
        let caps = caps.unwrap();

        let zoom = caps.name("zoom");
        let x = caps.name("x");
        let y = caps.name("y");
        if zoom.is_none() || x.is_none() || y.is_none() {
            return None;
        }
        let zoom = zoom.unwrap();
        let x = x.unwrap();
        let y = y.unwrap();

        let zoom = zoom.parse();
        let x = x.parse();
        let y = y.parse();
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
            },
            _ => {
                Tile::new(self.zoom-1, self.x/2, self.y/2)
            }
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
            std::u8::MAX => {
                None
            },
            _ => {
                let z = self.zoom+1;
                let x = 2*self.x;
                let y = 2*self.y;
                Some([Tile{zoom:z, x:x, y:y}, Tile{zoom:z, x:x+1, y:y}, Tile{zoom:z, x:x, y:y+1}, Tile{zoom:z, x:x+1, y:y+1}])
            }
        }
    }

    /// Iterate on all child tiles of this tile
    pub fn all_subtiles_iter(&self) -> AllSubTilesIterator {
        AllSubTilesIterator::new_from_tile(&self)
    }

    /// Returns the LatLon for the centre of this tile.
    pub fn centre_point(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32)+0.5, (self.y as f32)+0.5)
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
        tile_nw_lat_lon(self.zoom, (self.x as f32)+1.0, self.y as f32)
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this tile.
    pub fn sw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, self.x as f32, (self.y as f32)+1.0)
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this tile.
    pub fn se_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32)+1.0, (self.y as f32)+1.0)
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
        format!("{}/{}/{}/{}/{}/{}/{}.{}", self.zoom, tc[0], tc[1], tc[2], tc[3], tc[4], tc[5], ext)
    }

    /// Returns the MP (MapProxy) path for storing this tile.
    pub fn mp_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let mp = xy_to_mp(self.x, self.y);
        format!("{}/{}/{}/{}/{}.{}", self.zoom, mp[0], mp[1], mp[2], mp[3], ext)
    }

    /// Returns the TS (TileStash safe) path for storing this tile.
    pub fn ts_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let ts = xy_to_ts(self.x, self.y);
        format!("{}/{}/{}/{}/{}.{}", self.zoom, ts[0], ts[1], ts[2], ts[3], ext)
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
        format!("{}/{}/{}/{}/{}/{}/{}.{}", self.zoom, tc[0], tc[1], tc[2], tc[3], tc[4], tc[5], ext)
    }

    /// Returns an iterator that yields all the tiles possible, starting from `0/0/0`. Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    ///
    /// ```
    /// # use slippy_map_tiles::Tile;
    /// let mut all_tiles_iter = Tile::all();
    /// ```
    pub fn all() -> AllTilesIterator {
        AllTilesIterator{ next_zoom: 0, next_zorder: 0 }
    }

    /// Returns an iterator that yields all the tiles from zoom 0 down to, and including, all the
    /// tiles at `max_zoom` zoom level.  Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    pub fn all_to_zoom(max_zoom: u8) -> AllTilesToZoomIterator {
        AllTilesToZoomIterator{ max_zoom: max_zoom, next_zoom: 0, next_x: 0, next_y: 0}
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

    #[cfg(feature="world_file")]
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
            static ref TILE_RE: Regex = Regex::new("^(?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})$").unwrap();
        }

        let caps = TILE_RE.captures(s);

        if caps.is_none() {
            return Err("Tile Z/X/Y regex didn't match");
        }
        let caps = caps.unwrap();

        // If the regex matches, then none of these should fail, right?
        let zoom = caps.name("zoom").unwrap().parse().unwrap();
        let x = caps.name("x").unwrap().parse().unwrap();
        let y = caps.name("y").unwrap().parse().unwrap();
        
        match Tile::new(zoom, x, y) {
            None => {
                // Invalid x or y for the zoom
                Err("Invalid X or Y for this zoom")
            },
            Some(t) => {
                Ok(t)
            }
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
        let zoom =  self.next_zoom;
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
    let remaining_rows = max_tile_no - next_x -1;
    let remaining_rows = remaining_rows as usize;

    let remaining_after_this_column = remaining_rows.checked_mul(max_tile_no as usize);
    if remaining_after_this_column.is_none() {
        return None;
    }
    let remaining_after_this_column = remaining_after_this_column.unwrap();


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
        } else  if self.next_zoom < std::u8::MAX {
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

        let remaining_in_this_level = remaining_in_this_zoom(self.next_zoom, self.next_x, self.next_y);
        if remaining_in_this_level.is_none() {
            return (std::usize::MAX, None);
        }
        let remaining_in_this_level = remaining_in_this_level.unwrap();


        let mut total: usize = remaining_in_this_level as usize;
        for i in (self.next_zoom+1)..(self.max_zoom+1) {
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
        AllSubTilesIterator{ _tiles: new_tiles }
    }

}

impl Iterator for AllSubTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self._tiles.len() == 0 {
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
        if ! scale.is_power_of_two() {
            return None;
        }
        if zoom >= 100 {
            None
        } else if x < 2u32.pow(zoom as u32) && y < 2u32.pow(zoom as u32) {
            let s = scale as u32;
            let x = (x / s) * s;
            let y = (y / s) * s;
            Some(Metatile { scale: scale, zoom: zoom, x: x, y: y })
        } else {
            None
        }
    }

    pub fn scale(&self) -> u8 { self.scale }

    pub fn zoom(&self) -> u8 { self.zoom }

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
        tile_nw_lat_lon(self.zoom, (self.x as f32)+(self.size() as f32)/2., (self.y as f32)+(self.size() as f32)/2.)
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
        tile_nw_lat_lon(self.zoom, (self.x+self.size() as u32) as f32, self.y as f32)
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this metatile.
    pub fn sw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, self.x as f32, (self.y+self.size() as u32) as f32)
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this metatile.
    pub fn se_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x+self.size() as u32) as f32, (self.y+self.size() as u32) as f32)
    }

    /// X value of this metatile
    pub fn x(&self) -> u32 { self.x }

    /// Y value of metatile
    pub fn y(&self) -> u32 { self.y }

    pub fn tiles(&self) -> Vec<Tile> {
        let size = self.size() as u32;
        (0..(size*size)).map(|n| {
            // oh for a divmod
            let (i, j) = (n / size, n % size);
            // being cheeky and skipping the usuall Tile::new checks here, since we know it's valid
            Tile{ zoom: self.zoom, x: self.x+i, y: self.y+j }
        }).collect()
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
            static ref METATILE_RE: Regex = Regex::new("^(?P<scale>[0-9]+) (?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})$").unwrap();
        }

        let caps = METATILE_RE.captures(s);

        if caps.is_none() {
            return Err(());
        }
        let caps = caps.unwrap();

        // If the regex matches, then none of these should fail, right?
        let scale = caps.name("scale").unwrap().parse().unwrap();
        let zoom = caps.name("zoom").unwrap().parse().unwrap();
        let x = caps.name("x").unwrap().parse().unwrap();
        let y = caps.name("y").unwrap().parse().unwrap();
        
        match Metatile::new(scale, zoom, x, y) {
            None => {
                // Invalid x or y for the zoom
                Err(())
            },
            Some(mt) => {
                Ok(mt)
            }
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
        MetatilesIterator{ scale: scale, curr_zoom: 0, curr_zorder: 0, bbox: None, maxzoom: 32, curr_zoom_width_height: None, curr_zoom_start_xy: None, total: None, tile_list_file: None }
    }
    
    pub fn new_for_bbox(scale: u8, bbox: &BBox) -> Self {
        MetatilesIterator::new_for_bbox_zoom(scale, &Some(bbox.clone()), 0, 32)
    }

    /// `None` for bbox means 'whole world'
    pub fn new_for_bbox_zoom(scale: u8, bbox: &Option<BBox>, minzoom: u8, maxzoom: u8) -> Self {
        let mut it = MetatilesIterator{ scale: scale, curr_zoom: minzoom, curr_zorder: 0, bbox: bbox.clone(), maxzoom: maxzoom, curr_zoom_width_height: None, curr_zoom_start_xy: None, total: None, tile_list_file: None };
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

        MetatilesIterator{ scale: 0, curr_zoom: 0, curr_zorder: 0, bbox: None, maxzoom: 0, curr_zoom_width_height: None, curr_zoom_start_xy: None, total: Some(total), tile_list_file: Some(file) }
    }

    /// Update the `self.curr_zoom_width_height` variable with the correct value for this zoom
    /// (`self.curr_zoom`)
    fn set_zoom_width_height(&mut self) {
        if let Some(ref bbox) = self.bbox {
            let scale = self.scale as u32;
            let zoom = self.curr_zoom;
            // TODO is this x/y lat/lon the right way around?
            let (x1, y1) = lat_lon_to_tile(bbox.top, bbox.left, zoom);
            let (x1, y1) = (x1/scale, y1/scale);
            let (x2, y2) = lat_lon_to_tile(bbox.bottom, bbox.right, zoom);
            let (x2, y2) = (x2/scale, y2/scale);

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
        self.curr_zoom_start_xy = Some((x1/self.scale as u32, y1/self.scale as u32));
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
                    let mut max = max_num/scale;
                    if max_num % scale > 0 {
                        max += 1
                    }
                    (max, max)
                },
                Some((width, height)) => (width, height),
            };

            let max_zorder_for_zoom = xy_to_zorder(width-1, height-1);

            let (i, j) = zorder_to_xy(self.curr_zorder);
            let bits = match self.curr_zoom_start_xy {
                None => (i, j),
                Some(start) => (start.0+i, start.1+j),
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

        let (x, y) = (x*scale, y*scale);
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
    inner: Metatile
}

impl ModTileMetatile {
    pub fn new(zoom: u8, x: u32, y: u32) -> Option<Self> {
        match Metatile::new(8, zoom, x, y) {
            None => None,
            Some(inner) => Some(ModTileMetatile{ inner: inner })
        }
    }

    /// Returns the mod_tile path for storing this metatile
    pub fn path<T: std::fmt::Display>(&self, ext: T) -> String {
        let mt = xy_to_mt(self.inner.x, self.inner.y);
        format!("{}/{}/{}/{}/{}/{}.{}", self.inner.zoom, mt[0], mt[1], mt[2], mt[3], mt[4], ext)
    }

    /// X value of this metatile
    pub fn x(&self) -> u32 { self.inner.x }

    /// Y value of metatile
    pub fn y(&self) -> u32 { self.inner.y }

    /// Y value of metatile
    pub fn size(self) -> u8 { self.inner.size() }

}


fn tile_nw_lat_lon(zoom: u8, x: f32, y: f32) -> LatLon {
    let n: f32 = 2f32.powi(zoom as i32);
    let lon_deg: f32 = (x as f32) / n * 360f32 - 180f32;
    let lat_rad: f32 = ((1f32 - 2f32 * (y as f32) / n) * std::f32::consts::PI).sinh().atan();
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
    let lat = if lat > MAX_LAT { MAX_LAT } else if lat < -MAX_LAT { -MAX_LAT } else { lat };

    let n: f64 = 2f64.powi(zoom as i32);
    let xtile: u32 = (n * ((lon + 180.) / 360.)).trunc() as u32;
    let ytile: u32 = (n * (1. - ((lat.tan() + (1. / lat.cos())).ln() / std::f64::consts::PI)) / 2.).trunc() as u32;

    (xtile, ytile)
}

/// Return the x,y of a tile which (for this zoom) has this web mercator 3857 x/y, and then the x,y
/// of the pixel within that image (presuming a 256x256 image)
pub fn merc_location_to_tile_coords(x: f64, y: f64, zoom: u8) -> ((u32, u32), (u32, u32)) {
    let num_tiles = 2u32.pow(zoom as u32) as f64;
    let global_extent = 20_037_508.342789244;
    let tile_width = (2.*global_extent) / num_tiles;

    (
        // location within the tile
        (
            ((x+global_extent)/tile_width) as u32,
            ((y+global_extent)/tile_width) as u32,
        ),
    
        // Tile x/y
        (
            (((x+global_extent)%tile_width)/tile_width*256.) as u32,
            (num_tiles - ((y+global_extent)%tile_width)/tile_width*256. - 1.) as u32,
        )
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
    let bottom = (bottom_right_tile.0 / metatile_scale)*metatile_scale;
    let top = (top_left_tile.0 / metatile_scale)*metatile_scale;
    let left = (top_left_tile.1 / metatile_scale)*metatile_scale;
    let right = (bottom_right_tile.1 / metatile_scale)*metatile_scale;

    let height = ((bottom - top)/metatile_scale as u32) as usize + 1;
    let width = ((right - left)/metatile_scale as u32) as usize + 1;
    
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
        if lat <= 90f32 && lat >= -90f32 && lon <= 180f32 && lon >= -180f32 {
            Some(LatLon{ lat: lat, lon: lon })
        } else {
            None
        }
    }

    /// Latitude
    pub fn lat(&self) -> f32 { self.lat }
    /// Longitude
    pub fn lon(&self) -> f32 { self.lon }

    /// Convert to Web Mercator format (SRID 3857)
    pub fn to_3857(&self) -> (f32, f32) {
        let x = self.lon() * 20037508.34 / 180.;
        let pi = std::f32::consts::PI;
        let y = ((90. + self.lat()) * pi / 360.).tan().ln() / (pi / 180.);
        let y = y * 20037508.34 / 180.;
        
        (x, y)
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

        if top <= 90. && top >= -90. && bottom <= 90. && bottom >= -90.
             && left <= 180. && left >= -180. && right <= 180. && right >= -180. {
             Some(BBox{ top: top, left: left, bottom: bottom, right: right })
        } else {
            None
        }
    }

    /// Given two points, return the bounding box specified by those 2 points
    pub fn new_from_points(topleft: &LatLon, bottomright: &LatLon) -> BBox {
        BBox{ top: topleft.lat, left: topleft.lon, bottom: bottomright.lat, right: bottomright.lon }
    }

    /// Construct a BBox from a tile
    pub fn new_from_tile(tile: &Tile) -> Self {
        tile.bbox()
    }

    /// Return true iff this point is in this bbox
    pub fn contains_point(&self, point: &LatLon) -> bool {
        (point.lat <= self.top && point.lat > self.bottom && point.lon >= self.left && point.lon < self.right)
    }


    /// Returns true iff this bbox and `other` share at least one point
    pub fn overlaps_bbox(&self, other: &BBox) -> bool {
        // FXME check top & left edges
        (self.left < other.right && self.right > other.left && self.top > other.bottom && self.bottom < other.top)
    }

    /// Iterate over all the tiles from z0 onwards that this bbox is in
    pub fn tiles(&self) -> BBoxTilesIterator {
        BBoxTilesIterator::new(&self)
    }

    /// Iterate over all the metatiles from z0 onwards that this bbox is in
    pub fn metatiles(&self, scale: u8) -> MetatilesIterator {
        let bbox: BBox = (*self).clone();
        MetatilesIterator{ curr_zoom: 0, maxzoom: 32, bbox: Some(bbox), curr_zorder: 0, scale: scale, curr_zoom_width_height: None, curr_zoom_start_xy: None, total: None, tile_list_file: None }
    }

    /// Return the top value of this bbox
    pub fn top(&self) -> f32 { self.top }

    /// Return the bottom value of this bbox
    pub fn bottom(&self) -> f32 { self.bottom }

    /// Return the left value of this bbox
    pub fn left(&self) -> f32 { self.left }

    /// Return the right value of this bbox
    pub fn right(&self) -> f32 { self.right }
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
        let caps = SIMPLE_COPY_SPACE.captures(string).or_else(|| { SIMPLE_COPY_COMMA.captures(string) } );
        if caps.is_none() {
            return Err("regex not match")
        }
        let caps = caps.unwrap();

        let minlat = caps.name("minlat");
        let maxlat = caps.name("maxlat");
        let minlon = caps.name("minlon");
        let maxlon = caps.name("maxlon");

        if minlat.is_none() || maxlat.is_none() || minlon.is_none() || maxlon.is_none() {
            return Err("bad lat/lon");
        }

        let minlat = minlat.unwrap().parse();
        let maxlat = maxlat.unwrap().parse();
        let minlon = minlon.unwrap().parse();
        let maxlon = maxlon.unwrap().parse();

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
        BBoxTilesIterator{ bbox: bbox, tiles: vec![Tile::new(0, 0, 0).unwrap()], tile_index: 0 }
    }
}

impl<'a> Iterator for BBoxTilesIterator<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.tile_index >= self.tiles.len() {
            // We've sent off all the existing tiles, so start looking at the children
            let mut new_tiles: Vec<Tile> = Vec::with_capacity(self.tiles.len()*4);
            for t in self.tiles.iter() {
                match t.subtiles() {
                    None => { },
                    Some(sub) => {
                        if self.bbox.overlaps_bbox(&sub[0].bbox()) { new_tiles.push(sub[0]); }
                        if self.bbox.overlaps_bbox(&sub[1].bbox()) { new_tiles.push(sub[1]); }
                        if self.bbox.overlaps_bbox(&sub[2].bbox()) { new_tiles.push(sub[2]); }
                        if self.bbox.overlaps_bbox(&sub[3].bbox()) { new_tiles.push(sub[3]); }
                    }
                }
            }

            new_tiles.shrink_to_fit();
            self.tiles = new_tiles;
            self.tile_index = 0;
        }

        let tile = self.tiles[self.tile_index].clone();
        self.tile_index += 1;
        Some(tile)
    }
}


/// Convert x & y to a TileCache (tc) directory parts
fn xy_to_tc(x: u32, y: u32) -> [String; 6] {
    [
        format!("{:03}", x/1_000_000),
        format!("{:03}", (x / 1_000) % 1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y/1_000_000),
        format!("{:03}", (y / 1_000) % 1_000),
        format!("{:03}", y % 1_000),
    ]
}

/// Convert x & y to a MapProxy (mp) directory parts
fn xy_to_mp(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:04}", x/10_000),
        format!("{:04}", x % 10_000),
        format!("{:04}", y/10_000),
        format!("{:04}", y % 10_000),
    ]
}

/// Convert x & y to a TileStash (ts) safe directory parts
fn xy_to_ts(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:03}", x/1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y/1_000),
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

    let c = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
    x >>= 4;
    y >>= 4;

    let b = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
    x >>= 4;
    y >>= 4;

    let a = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
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
        if x_set  {
            res |= 1 << i*2;
        }
        if y_set {
            res |= 1 << (i*2)+1;
        }

    }

    res
}

pub fn zorder_to_xy(zorder: u64) -> (u32, u32) {
    let mut x: u32 = 0;
    let mut y: u32 = 0;

    for i in 0..32 {
        let x_bit_set = (zorder >> i*2) & 1 == 1;
        let y_bit_set = (zorder >> (i*2)+1) & 1 == 1;

        if x_bit_set {
            x |= 1 << i;
        }
        if y_bit_set {
            y |= 1 << i;
        }
    }

    (x, y)
}


// TODO do mod_tile tile format

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tc() {
        let res = xy_to_tc(3, 4);
        assert_eq!(res[0], "000");
        assert_eq!(res[1], "000");
        assert_eq!(res[2], "003");
        assert_eq!(res[3], "000");
        assert_eq!(res[4], "000");
        assert_eq!(res[5], "004");
    }

    #[test]
    fn mp() {
        let res = xy_to_mp(3, 4);
        assert_eq!(res[0], "0000");
        assert_eq!(res[1], "0003");
        assert_eq!(res[2], "0000");
        assert_eq!(res[3], "0004");
    }

    #[test]
    fn ts() {
        let res = xy_to_ts(656, 1582);
        assert_eq!(res[0], "000");
        assert_eq!(res[1], "656");
        assert_eq!(res[2], "001");
        assert_eq!(res[3], "582");
    }

    #[test]
    fn tiles_parsing() {
        let tile = Tile::new(1, 5, 5);
        assert!(tile.is_none());

        assert!(Tile::new(4, 8, 9).is_some());

        let tile = Tile::new(1, 0, 0);
        assert!(tile.is_some());

        assert!(Tile::new(100, 0, 0).is_none());
    }

    #[test]
    fn tiles() {
        let tile = Tile::new(1, 0, 0);

        assert!(tile.is_some());
        let tile = tile.unwrap();

        assert_eq!(tile.zoom(), 1);
        assert_eq!(tile.x(), 0);
        assert_eq!(tile.y(), 0);

        let parent = tile.parent();
        assert!(parent.is_some());
        let parent = parent.unwrap();
        assert_eq!(parent, Tile::new(0, 0, 0).unwrap());

        assert_eq!(parent.centre_point(), LatLon::new(0f32, 0f32).unwrap());
        assert_eq!(parent.nw_corner(), LatLon::new(85.05112, -180.0).unwrap());
        assert_eq!(parent.ne_corner(), LatLon::new(85.05112, 180.0).unwrap());
        assert_eq!(parent.sw_corner(), LatLon::new(-85.05112, -180.0).unwrap());
        assert_eq!(parent.se_corner(), LatLon::new(-85.05112, 180.0).unwrap());

        assert_eq!(parent.top(), 85.05112);
        assert_eq!(parent.bottom(), -85.05112);
        assert_eq!(parent.left(), -180.0);
        assert_eq!(parent.right(), 180.0);


        assert_eq!(parent.tc_path("png"), "0/000/000/000/000/000/000.png");
        assert_eq!(parent.mp_path("png"), "0/0000/0000/0000/0000.png");
        assert_eq!(parent.ts_path("png"), "0/000/000/000/000.png");
        assert_eq!(parent.zxy(), "0/0/0");
        assert_eq!(parent.zxy_path("png"), "0/0/0.png");

        let children = parent.subtiles();
        assert_eq!(children.is_none(), false);
        let children: [Tile; 4] = children.unwrap();
        assert_eq!(children[0], Tile::new(1, 0, 0).unwrap());
        assert_eq!(children[0].tc_path("png"), "1/000/000/000/000/000/000.png");

        assert_eq!(children[1], Tile::new(1, 1, 0).unwrap());
        assert_eq!(children[1].tc_path("png"), "1/000/000/001/000/000/000.png");

        assert_eq!(children[2], Tile::new(1, 0, 1).unwrap());
        assert_eq!(children[2].tc_path("png"), "1/000/000/000/000/000/001.png");

        assert_eq!(children[3], Tile::new(1, 1, 1).unwrap());
        assert_eq!(children[3].tc_path("png"), "1/000/000/001/000/000/001.png");
        assert_eq!(children[3].zxy_path("png"), "1/1/1.png");
        assert_eq!(children[3].zxy(), "1/1/1");
        
    }

    #[test]
    fn tile_from_tms() {
        fn known_good(tms: &str, zoom: u8, x: u32, y: u32) {
            let tile = Tile::from_tms(tms);
            assert!(tile.is_some());
            let tile = tile.unwrap();
            assert_eq!(tile.zoom, zoom);
            assert_eq!(tile.x, x);
            assert_eq!(tile.y, y);
        }

        fn known_bad(tms: &str) {
            let tile = Tile::from_tms(tms);
            assert!(tile.is_none());
        }

        known_good("/0/0/0.png", 0, 0, 0);
        known_good("/17/1/1234.png", 17, 1, 1234);
        known_good("17/1/1234", 17, 1, 1234);
        known_good("17/1/1234.jpeg", 17, 1, 1234);
        known_good("/17/1/1234.jpeg", 17, 1, 1234);

        known_bad("foo");
        known_bad("/17/1/1234.jpegz");
        known_bad("/17z/1/1234.jpegz");
        known_bad("/0/1/1.png");
        known_bad("/100/1/1.png");

        known_good("http://tile.example.org/17/1/1234", 17, 1, 1234);
        known_good("http://tile.example.org/17/1/1234.png", 17, 1, 1234);
        known_bad("http://tile.example.org/17/1");
        known_bad("http://tile.example.org/17");
        known_bad("http://tile.example.org/17/1/1234.png/foo/bar");
    }

    #[test]
    fn all_tiles() {

        let mut it = Tile::all();

        assert_eq!(it.next(), Tile::new(0, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 1, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 1));
        assert_eq!(it.next(), Tile::new(1, 1, 1));
        assert_eq!(it.next(), Tile::new(2, 0, 0));
        assert_eq!(it.next(), Tile::new(2, 1, 0));
        assert_eq!(it.next(), Tile::new(2, 0, 1));
        assert_eq!(it.next(), Tile::new(2, 1, 1));
        assert_eq!(it.next(), Tile::new(2, 2, 0));

        let it = Tile::all();
        let z5_tiles: Vec<Tile> = it.skip_while(|t| { t.zoom < 5 }).take(1).collect();
        assert_eq!(z5_tiles[0], Tile::new(5, 0, 0).unwrap());

    }

    #[test]
    fn latlon_create() {

        let p1 = LatLon::new(54.9, 5.5).unwrap();
        assert_eq!(p1.lat(), 54.9);
        assert_eq!(p1.lon(), 5.5);

        assert_eq!(p1.to_3857(), (612257.20, 7342480.5));
    }

    #[test]
    fn bbox_create() {

        // left=5.53 bottom=47.23 right=15.38 top=54.96
        let b1: Option<BBox> = BBox::new(54.9, 5.5, 47.2, 15.38);
        assert!(b1.is_some());
        let b1 = b1.unwrap();
        assert_eq!(b1.top, 54.9);

        let p1 = LatLon::new(54.9, 5.5).unwrap();
        let p2 = LatLon::new(47.2, 15.38).unwrap();
        let b2: BBox = BBox::new_from_points(&p1, &p2);
        assert_eq!(b1, b2);
    }

    #[test]
    fn bbox_from_string() {

        let bbox = "10 20 30 40".parse().ok();
        assert!(bbox.is_some());
        let bbox: BBox = bbox.unwrap();
        assert_eq!(bbox.top(), 40.);
        assert_eq!(bbox.left(), 10.);
        assert_eq!(bbox.bottom(), 20.);
        assert_eq!(bbox.right(), 30.);

        let bbox = "10,20,30,40".parse().ok();
        assert!(bbox.is_some());
        let bbox: BBox = bbox.unwrap();
        assert_eq!(bbox.top(), 40.);
        assert_eq!(bbox.left(), 10.);
        assert_eq!(bbox.bottom(), 20.);
        assert_eq!(bbox.right(), 30.);

        let bbox = "71.6,-25.93,35.55,48.9".parse().ok();
        assert!(bbox.is_some());
        let bbox: BBox = bbox.unwrap();
        assert_eq!(bbox.top(), 48.9);
        assert_eq!(bbox.left(), 71.6);
        assert_eq!(bbox.bottom(), -25.93);
        assert_eq!(bbox.right(), 35.55);

        fn known_bad(s: &str) {
            assert!(BBox::from_str(s).is_err());
        }
        known_bad("foo");
        known_bad("1.1.1.1");
        known_bad("1  1  1  1");
    }

    #[test]
    fn bbox_tile() {
        let t = Tile::new(0, 0, 0).unwrap();
        assert_eq!(t.bbox(), BBox::new(85.05112, -180., -85.05112, 180.).unwrap());
    }

    #[test]
    fn bbox_contains_point(){
        // triangle from London, to Bristol to Birmingham
        let tile = Tile::new(7, 63, 42).unwrap();
        let bbox = tile.bbox();
        let point1 = LatLon::new(51.75193, -1.25781).unwrap();  // oxford
        let point2 = LatLon::new(48.7997, 2.4218).unwrap();     // paris

        assert!(bbox.contains_point(&point1));
        assert!(!bbox.contains_point(&point2));

        // only the top and left borders are included in the bbox
        let nw_corner = tile.nw_corner();
        assert!(bbox.contains_point(&nw_corner));

        // Create  new point on the top edge along to the right from the NW corner
        let nw_right = LatLon::new(nw_corner.lat, nw_corner.lon+0.001).unwrap();
        assert!(bbox.contains_point(&nw_right));

        
        assert!(!bbox.contains_point(&tile.sw_corner()));
        assert!(!bbox.contains_point(&tile.ne_corner()));
        assert!(!bbox.contains_point(&tile.se_corner()));
    }

    #[test]
    fn bbox_overlaps() {

        let tile = Tile::new(7, 63, 42).unwrap();
        let parent_tile = tile.parent().unwrap();

        assert!(parent_tile.bbox().overlaps_bbox(&tile.bbox()));

        let tile2 = Tile::new(7, 63, 43).unwrap();
        assert!(!tile.bbox().overlaps_bbox(&tile2.bbox()));
    }

    #[test]
    fn bbox_tile_iter() {

        // left=-11.32 bottom=51.11 right=-4.97 top=55.7
        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        let mut tiles = ie_bbox.tiles();
        assert_eq!(tiles.next(), Tile::new(0, 0, 0));
        assert_eq!(tiles.next(), Tile::new(1, 0, 0));
        assert_eq!(tiles.next(), Tile::new(2, 1, 1));
        assert_eq!(tiles.next(), Tile::new(3, 3, 2));
        assert_eq!(tiles.next(), Tile::new(4, 7, 5));
        assert_eq!(tiles.next(), Tile::new(5, 14, 10));
        assert_eq!(tiles.next(), Tile::new(5, 15, 10));
        assert_eq!(tiles.next(), Tile::new(6, 29, 20));
        assert_eq!(tiles.next(), Tile::new(6, 29, 21));

    }

    #[test]
    fn test_num_tiles_in_zoom() {

        assert_eq!(num_tiles_in_zoom(0), Some(1));
        assert_eq!(num_tiles_in_zoom(1), Some(4));
        assert_eq!(num_tiles_in_zoom(2), Some(16));
        assert_eq!(num_tiles_in_zoom(3), Some(256));
        assert_eq!(num_tiles_in_zoom(4), Some(65_536));
        assert_eq!(num_tiles_in_zoom(5), Some(4_294_967_296));

        assert_eq!(num_tiles_in_zoom(6), None);

        // Can't do these because the integers overflow
        //assert_eq!(num_tiles_in_zoom(17), 17_179_869_184);
        //assert_eq!(num_tiles_in_zoom(18), 68_719_476_736);
        //assert_eq!(num_tiles_in_zoom(19), 274_877_906_944);
    }

    #[test]
    fn test_remaining_in_zoom() {

        assert_eq!(remaining_in_this_zoom(0, 0, 0), Some(1));

        assert_eq!(remaining_in_this_zoom(1, 0, 0), Some(4));
        assert_eq!(remaining_in_this_zoom(1, 0, 1), Some(3));
        assert_eq!(remaining_in_this_zoom(1, 1, 0), Some(2));
        assert_eq!(remaining_in_this_zoom(1, 1, 1), Some(1));

        assert_eq!(remaining_in_this_zoom(2, 0, 0), Some(16));
    }

    #[test]
    fn all_tiles_to_zoom_iter() {

        let mut it = Tile::all_to_zoom(1);

        assert_eq!(it.next(), Tile::new(0, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 1));
        assert_eq!(it.next(), Tile::new(1, 1, 0));
        assert_eq!(it.next(), Tile::new(1, 1, 1));
        assert_eq!(it.next(), None);


        assert_eq!(Tile::all_to_zoom(0).count(), 1);
        assert_eq!(Tile::all_to_zoom(1).count(), 5);
        assert_eq!(Tile::all_to_zoom(2).count(), 21);
        assert_eq!(Tile::all_to_zoom(3).count(), 85);

        assert_eq!(Tile::all_to_zoom(2).last(), Tile::new(2, 3, 3));

        // check the size hints
        assert_eq!(Tile::all_to_zoom(0).size_hint(), (1, Some(1)));

        let mut it = Tile::all_to_zoom(1);
        assert_eq!(it.size_hint(), (5, Some(5)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (4, Some(4)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());

        assert_eq!(Tile::all_to_zoom(2).size_hint(), (21, Some(21)));

        assert_eq!(Tile::all_to_zoom(3).size_hint(), (277, Some(277)));
        assert_eq!(Tile::all_to_zoom(4).size_hint(), (65_813, Some(65_813)));
        assert_eq!(Tile::all_to_zoom(5).size_hint(), (4_295_033_109, Some(4_295_033_109)));
        assert_eq!(Tile::all_to_zoom(6).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(7).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(8).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(9).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(10).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(11).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(12).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(13).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(14).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(15).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(16).size_hint(), (18_446_744_073_709_551_615, None));

    }

    #[test]
    fn all_sub_tiles_iter() {
        let mut it = Tile::new(4, 7, 5).unwrap().all_subtiles_iter();
        assert_eq!(it.next(), Tile::new(5, 14, 10));
        assert_eq!(it.next(), Tile::new(5, 15, 10));
        assert_eq!(it.next(), Tile::new(5, 14, 11));
        assert_eq!(it.next(), Tile::new(5, 15, 11));

        let z10tiles: Vec<Tile> = Tile::new(4, 7, 5).unwrap().all_subtiles_iter().take_while(|t| t.zoom() < 11).filter(|t| t.zoom() == 10).collect();
        assert_eq!(z10tiles.len(), 4096);
        assert_eq!(z10tiles[0].zoom(), 10);
        assert_eq!(z10tiles[z10tiles.len()-1].zoom(), 10);
        
    }

    #[test]
    fn test_xy_to_zorder() {
        assert_eq!(xy_to_zorder(0, 0), 0);
        assert_eq!(xy_to_zorder(1, 0), 1);
        assert_eq!(xy_to_zorder(0, 1), 2);
        assert_eq!(xy_to_zorder(1, 1), 3);
    }

    #[test]
    fn test_zorder_to_xy() {
        assert_eq!(zorder_to_xy(0), (0, 0));
        assert_eq!(zorder_to_xy(1), (1, 0));
    }

    #[test]
    fn test_metatile() {

        let mt = Metatile::new(8, 0, 0, 0);
        assert!(mt.is_some());
        let mt = mt.unwrap();
        assert_eq!(mt.scale(), 8);
        assert_eq!(mt.zoom, 0);
        assert_eq!(mt.x, 0);
        assert_eq!(mt.y, 0);

        let mt = Metatile::new(8, 3, 3, 2);
        assert!(mt.is_some());
        let mt = mt.unwrap();
        assert_eq!(mt.zoom, 3);
        assert_eq!(mt.x, 0);
        assert_eq!(mt.y, 0);

        let t = Tile::new(3, 3, 2).unwrap();
        assert_eq!(t.metatile(8), Some(mt));

    }

    #[test]
    fn test_metatile_all() {

        let mut it = Metatile::all(8);

        assert_eq!(it.next(), Metatile::new(8, 0, 0, 0));
        assert_eq!(it.next(), Metatile::new(8, 1, 0, 0));
        assert_eq!(it.next(), Metatile::new(8, 2, 0, 0));
        assert_eq!(it.next(), Metatile::new(8, 3, 0, 0));

        assert_eq!(it.next(), Metatile::new(8, 4, 0, 0));
        assert_eq!(it.next(), Metatile::new(8, 4, 8, 0));
        assert_eq!(it.next(), Metatile::new(8, 4, 0, 8));
        assert_eq!(it.next(), Metatile::new(8, 4, 8, 8));

        assert_eq!(it.next(), Metatile::new(8, 5, 0, 0));

        let it = Metatile::all(8);
        let tiles: Vec<Metatile> = it.take_while(|mt| mt.zoom < 11).filter(|mt| mt.zoom == 10).collect();
        assert_eq!(tiles.len(), 16384);
        assert_eq!(tiles[1], Metatile::new(8, 10, 8, 0).unwrap());
    }

    #[test]
    fn test_metatile_bbox() {

        assert_eq!(Metatile::new(8, 0, 0, 0).unwrap().size(), 1);
        assert_eq!(Metatile::new(8, 1, 0, 0).unwrap().size(), 2);
        assert_eq!(Metatile::new(8, 2, 0, 0).unwrap().size(), 4);
        assert_eq!(Metatile::new(8, 3, 0, 0).unwrap().size(), 8);
        assert_eq!(Metatile::new(8, 4, 0, 0).unwrap().size(), 8);
        assert_eq!(Metatile::new(8, 5, 0, 0).unwrap().size(), 8);

        let mt = Metatile::new(8, 2, 0, 0).unwrap();

        assert_eq!(mt.centre_point(), LatLon::new(0f32, 0f32).unwrap());
        assert_eq!(mt.nw_corner(), LatLon::new(85.05112, -180.0).unwrap());
        assert_eq!(mt.ne_corner(), LatLon::new(85.05112, 180.0).unwrap());
        assert_eq!(mt.sw_corner(), LatLon::new(-85.05112, -180.0).unwrap());
        assert_eq!(mt.se_corner(), LatLon::new(-85.05112, 180.0).unwrap());

    }

    #[test]
    fn test_metatile_subtiles() {

        assert_eq!(Metatile::new(8, 0, 0, 0).unwrap().tiles(), vec![(0, 0, 0)].into_iter().map(|c| Tile::new(c.0, c.1, c.2).unwrap()).collect::<Vec<Tile>>());
        assert_eq!(Metatile::new(8, 1, 0, 0).unwrap().tiles(), vec![(1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)].into_iter().map(|c| Tile::new(c.0, c.1, c.2).unwrap()).collect::<Vec<Tile>>());
        assert_eq!(Metatile::new(8, 2, 0, 0).unwrap().tiles(), vec![
                   (2, 0, 0), (2, 0, 1), (2, 0, 2), (2, 0, 3),
                   (2, 1, 0), (2, 1, 1), (2, 1, 2), (2, 1, 3),
                   (2, 2, 0), (2, 2, 1), (2, 2, 2), (2, 2, 3),
                   (2, 3, 0), (2, 3, 1), (2, 3, 2), (2, 3, 3),
                   ].into_iter().map(|c| Tile::new(c.0, c.1, c.2).unwrap()).collect::<Vec<Tile>>());

    }


    #[test]
    fn test_metatile_subtiles_bbox1() {

        // left=-11.32 bottom=51.11 right=-4.97 top=55.7
        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        let mut metatiles = ie_bbox.metatiles(8);
        assert_eq!(metatiles.next(), Metatile::new(8, 0, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 1, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 2, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 3, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 4, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 5, 8, 8));

        assert_eq!(metatiles.next(), Metatile::new(8, 6, 24, 16));

        assert_eq!(metatiles.next(), Metatile::new(8, 7, 56, 40));

        assert_eq!(metatiles.next(), Metatile::new(8, 8, 112, 80));
        assert_eq!(metatiles.next(), Metatile::new(8, 8, 120, 80));

        assert_eq!(metatiles.next(), Metatile::new(8, 9, 232, 160));
        assert_eq!(metatiles.next(), Metatile::new(8, 9, 240, 160));
        assert_eq!(metatiles.next(), Metatile::new(8, 9, 232, 168));
        assert_eq!(metatiles.next(), Metatile::new(8, 9, 240, 168));
    }

    #[test]
    fn test_metatile_subtiles_bbox2() {

        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        let mut metatiles = MetatilesIterator::new_for_bbox_zoom(8, &Some(ie_bbox), 0, 5);
        assert_eq!(metatiles.next(), Metatile::new(8, 0, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 1, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 2, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 3, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 4, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 5, 8, 8));
        assert_eq!(metatiles.next(), None);
    }


    #[test]
    fn test_metatile_subtiles_bbox3() {

        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        let mut metatiles = MetatilesIterator::new_for_bbox_zoom(8, &Some(ie_bbox), 5, 5);
        assert_eq!(metatiles.next(), Metatile::new(8, 5, 8, 8));
        assert_eq!(metatiles.next(), None);

    }

    #[test]
    fn test_metatile_subtiles_bbox4() {
        let ie_d_bbox = BBox::new(53.61, -6.66, 53.08, -5.98).unwrap();
        let mut metatiles = MetatilesIterator::new_for_bbox_zoom(8, &Some(ie_d_bbox), 0, 10);
        assert_eq!(metatiles.next(), Metatile::new(8, 0, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 1, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 2, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 3, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 4, 0, 0));
        assert_eq!(metatiles.next(), Metatile::new(8, 5, 8, 8));
        assert_eq!(metatiles.next(), Metatile::new(8, 6, 24, 16));
        assert_eq!(metatiles.next(), Metatile::new(8, 7, 56, 40));

        assert_eq!(metatiles.next(), Metatile::new(8, 8, 120, 80));

        assert_eq!(metatiles.next(), Metatile::new(8, 9, 240, 160));

        assert_eq!(metatiles.next(), Metatile::new(8, 10, 488, 328));

        assert_eq!(metatiles.next(), None);
    }
    
    #[test]
    fn test_lat_lon_to_tile1() {

        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 18), (130981, 87177));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 17), (65490, 43588));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 16), (32745, 21794));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 15), (16372, 10897));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 14), (8186, 5448));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 13), (4093, 2724));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 11), (1023, 681));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 10), (511, 340));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 9), (255, 170));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 8), (127, 85));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 7), (63, 42));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 6), (31, 21));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 5), (15, 10));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 4), (7, 5));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 3), (3, 2));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 2), (1, 1));
        assert_eq!(lat_lon_to_tile(51.50101, -0.12418, 0), (0, 0));
    }

    #[test]
    fn test_lat_lon_to_tile2() {
        assert_eq!(lat_lon_to_tile(53.61, -6.66, 9), (246, 165));
        assert_eq!(lat_lon_to_tile(53.08, -5.98, 9), (247, 166));

        assert_eq!(lat_lon_to_tile(53.61, -6.66, 10), (493, 330));
        assert_eq!(lat_lon_to_tile(53.08, -5.98, 10), (494, 333));
    }

    #[test]
    fn mod_tile_path() {
        let res = xy_to_mt(0, 0);
        assert_eq!(res[0], "0");
        assert_eq!(res[1], "0");
        assert_eq!(res[2], "0");
        assert_eq!(res[3], "0");
        assert_eq!(res[4], "0");

        let res = xy_to_mt(1, 1);
        assert_eq!(res[0], "0");
        assert_eq!(res[1], "0");
        assert_eq!(res[2], "0");
        assert_eq!(res[3], "0");
        assert_eq!(res[4], "17");
    }

    #[test]
    fn test_mod_tile_metatile() {
        let mt_meta = ModTileMetatile::new(0, 0, 0);
        assert!(mt_meta.is_some());
        let mt_meta = mt_meta.unwrap();
        assert_eq!(mt_meta.path("png"), "0/0/0/0/0/0.png");
    }
    
    #[test]
    fn size_bbox_zoom1() {
        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        assert_eq!(size_bbox_zoom(&ie_bbox, 0), Some(1));
        assert_eq!(size_bbox_zoom(&ie_bbox, 1), Some(1));
        assert_eq!(size_bbox_zoom(&ie_bbox, 2), Some(1));
        assert_eq!(size_bbox_zoom(&ie_bbox, 3), Some(1));
        assert_eq!(size_bbox_zoom(&ie_bbox, 4), Some(1));
        assert_eq!(size_bbox_zoom(&ie_bbox, 5), Some(2));
        assert_eq!(size_bbox_zoom(&ie_bbox, 6), Some(6));
        assert_eq!(size_bbox_zoom(&ie_bbox, 7), Some(12));
        assert_eq!(size_bbox_zoom(&ie_bbox, 8), Some(36));
        assert_eq!(size_bbox_zoom(&ie_bbox, 9), Some(120));
        assert_eq!(size_bbox_zoom(&ie_bbox, 10), Some(437));
        assert_eq!(size_bbox_zoom(&ie_bbox, 11), Some(1665));
        assert_eq!(size_bbox_zoom(&ie_bbox, 12), Some(6497));
        assert_eq!(size_bbox_zoom(&ie_bbox, 13), Some(25520));
        assert_eq!(size_bbox_zoom(&ie_bbox, 14), Some(102080));
        assert_eq!(size_bbox_zoom(&ie_bbox, 15), Some(407037));
        assert_eq!(size_bbox_zoom(&ie_bbox, 16), Some(1625585));
        assert_eq!(size_bbox_zoom(&ie_bbox, 17), Some(6494904));
        assert_eq!(size_bbox_zoom(&ie_bbox, 18), Some(25959136));
    }

    #[test]
    fn size_bbox_zoom2() {
        let bbox = BBox::new(1e-5, -1e-5, -1e-5, 1e-5).unwrap();
        assert_eq!(size_bbox_zoom(&bbox, 0), Some(1));
        assert_eq!(size_bbox_zoom(&bbox, 1), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 2), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 3), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 4), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 5), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 6), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 7), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 8), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 9), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 10), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 11), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 12), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 13), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 14), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 15), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 16), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 17), Some(4));
        assert_eq!(size_bbox_zoom(&bbox, 18), Some(4));
    }

    #[test]
    fn size_bbox_zoom_metatiles1() {
        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 0, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 1, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 2, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 3, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 4, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 5, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 6, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 7, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 8, 8), Some(2));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 9, 8), Some(6));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 10, 8), Some(12));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 11, 8), Some(36));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 12, 8), Some(120));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 13, 8), Some(437));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 14, 8), Some(1665));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 15, 8), Some(6497));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 16, 8), Some(25520));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 17, 8), Some(102080));
        assert_eq!(size_bbox_zoom_metatiles(&ie_bbox, 18, 8), Some(407037));
    }

    #[test]
    fn size_bbox_zoom_metatiles2() {
        let ie_d_bbox = BBox::new(53.61, -6.66, 53.08, -5.98).unwrap();
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 9, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 10, 8), Some(1));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 11, 8), Some(2));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 12, 8), Some(4));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 13, 8), Some(8));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 14, 8), Some(24));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 15, 8), Some(88));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 16, 8), Some(336));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 17, 8), Some(1344));
        assert_eq!(size_bbox_zoom_metatiles(&ie_d_bbox, 18, 8), Some(5166));
    }

    #[test]
    fn parse_metatile1() {
        assert_eq!("8 3/0/0".parse().ok(), Metatile::new(8, 3, 0, 0));
        assert_eq!("8 0/0/0".parse().ok(), Metatile::new(8, 0, 0, 0));
        assert_eq!("0 0/0/0".parse::<Metatile>().ok(), None);
        assert_eq!("8 0/10/10".parse::<Metatile>().ok(), None);
        assert_eq!("8 4/1/1".parse().ok(), Metatile::new(8, 4, 0, 0));
    }

    #[cfg(feature="world_file")]
    #[test]
    fn world_file() {
        let t = Tile::new(6, 33, 21).unwrap();
        let wf = t.world_file();
        assert_eq!(format!("{}", wf), "2445.98490512564\n0\n0\n-2445.98490512564\n626172.1357121654\n6887893.4928338025\n");
    }

}
