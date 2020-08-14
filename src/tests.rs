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
    let z5_tiles: Vec<Tile> = it.skip_while(|t| t.zoom < 5).take(1).collect();
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
    assert_eq!(
        t.bbox(),
        BBox::new(85.05112, -180., -85.05112, 180.).unwrap()
    );
}

#[test]
fn bbox_contains_point() {
    // triangle from London, to Bristol to Birmingham
    let tile = Tile::new(7, 63, 42).unwrap();
    let bbox = tile.bbox();
    let point1 = LatLon::new(51.75193, -1.25781).unwrap(); // oxford
    let point2 = LatLon::new(48.7997, 2.4218).unwrap(); // paris

    assert!(bbox.contains_point(&point1));
    assert!(!bbox.contains_point(&point2));

    // only the top and left borders are included in the bbox
    let nw_corner = tile.nw_corner();
    assert!(bbox.contains_point(&nw_corner));

    // Create  new point on the top edge along to the right from the NW corner
    let nw_right = LatLon::new(nw_corner.lat, nw_corner.lon + 0.001).unwrap();
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
    assert_eq!(
        Tile::all_to_zoom(5).size_hint(),
        (4_295_033_109, Some(4_295_033_109))
    );
    assert_eq!(
        Tile::all_to_zoom(6).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(7).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(8).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(9).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(10).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(11).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(12).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(13).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(14).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(15).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
    assert_eq!(
        Tile::all_to_zoom(16).size_hint(),
        (18_446_744_073_709_551_615, None)
    );
}

#[test]
fn all_sub_tiles_iter() {
    let mut it = Tile::new(4, 7, 5).unwrap().all_subtiles_iter();
    assert_eq!(it.next(), Tile::new(5, 14, 10));
    assert_eq!(it.next(), Tile::new(5, 15, 10));
    assert_eq!(it.next(), Tile::new(5, 14, 11));
    assert_eq!(it.next(), Tile::new(5, 15, 11));

    let z10tiles: Vec<Tile> = Tile::new(4, 7, 5)
        .unwrap()
        .all_subtiles_iter()
        .take_while(|t| t.zoom() < 11)
        .filter(|t| t.zoom() == 10)
        .collect();
    assert_eq!(z10tiles.len(), 4096);
    assert_eq!(z10tiles[0].zoom(), 10);
    assert_eq!(z10tiles[z10tiles.len() - 1].zoom(), 10);
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
    let tiles: Vec<Metatile> = it
        .take_while(|mt| mt.zoom < 11)
        .filter(|mt| mt.zoom == 10)
        .collect();
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
    assert_eq!(
        Metatile::new(8, 0, 0, 0).unwrap().tiles(),
        vec![(0, 0, 0)]
            .into_iter()
            .map(|c| Tile::new(c.0, c.1, c.2).unwrap())
            .collect::<Vec<Tile>>()
    );
    assert_eq!(
        Metatile::new(8, 1, 0, 0).unwrap().tiles(),
        vec![(1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)]
            .into_iter()
            .map(|c| Tile::new(c.0, c.1, c.2).unwrap())
            .collect::<Vec<Tile>>()
    );
    assert_eq!(
        Metatile::new(8, 2, 0, 0).unwrap().tiles(),
        vec![
            (2, 0, 0),
            (2, 0, 1),
            (2, 0, 2),
            (2, 0, 3),
            (2, 1, 0),
            (2, 1, 1),
            (2, 1, 2),
            (2, 1, 3),
            (2, 2, 0),
            (2, 2, 1),
            (2, 2, 2),
            (2, 2, 3),
            (2, 3, 0),
            (2, 3, 1),
            (2, 3, 2),
            (2, 3, 3),
        ]
        .into_iter()
        .map(|c| Tile::new(c.0, c.1, c.2).unwrap())
        .collect::<Vec<Tile>>()
    );
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

#[cfg(feature = "world_file")]
#[test]
fn world_file() {
    let t = Tile::new(6, 33, 21).unwrap();
    let wf = t.world_file();
    assert_eq!(
        format!("{}", wf),
        "2445.98490512564\n0\n0\n-2445.98490512564\n626172.1357121654\n6887893.4928338025\n"
    );
}

#[test]
fn bbox_tiles() {
    let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();

    macro_rules! assert_bbox {
        ($bbox:expr, $zoom:expr, $coords:expr ) => {{
            let output: Vec<Tile> = $bbox.tiles_for_zoom($zoom).collect();
            let expected: Vec<Tile> = $coords
                .into_iter()
                .map(|xy| Tile::new($zoom, xy.0, xy.1).unwrap())
                .collect();
            assert_eq!(output, expected);
        }};
    }

    assert_bbox!(&ie_bbox, 0, vec![(0, 0)]);
    assert_bbox!(&ie_bbox, 1, vec![(0, 0)]);
    assert_bbox!(&ie_bbox, 2, vec![(1, 1)]);
    assert_bbox!(&ie_bbox, 3, vec![(3, 2)]);
    assert_bbox!(&ie_bbox, 4, vec![(7, 5)]);
}

mod metatiles {
    use super::*;

    mod modtiles {
        use super::*;

        #[test]
        fn simple1() {
            let mt_meta = ModTileMetatile::new(0, 0, 0);
            assert!(mt_meta.is_some());
            let mt_meta = mt_meta.unwrap();
            assert_eq!(mt_meta.path("png"), "0/0/0/0/0/0.png");
        }

        #[test]
        fn simple2() {
            let mt = ModTileMetatile::new(0, 0, 0).unwrap();
            assert_eq!(mt.tiles(), vec![Tile::new(0, 0, 0).unwrap()]);
        }
    }
}
