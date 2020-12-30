use openslide_rs::{Address, DeepZoom, OpenSlide, Region, Size};

mod common;

#[test]
fn test_metadata() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();
    let dz = DeepZoom::new(&slide, 254, 1, false);

    assert_eq!(dz.level_count, 10);

    assert_eq!(
        dz.level_dimensions,
        vec![
            Size { w: 1, h: 1 },
            Size { w: 2, h: 1 },
            Size { w: 3, h: 2 },
            Size { w: 5, h: 4 },
            Size { w: 10, h: 8 },
            Size { w: 19, h: 16 },
            Size { w: 38, h: 32 },
            Size { w: 75, h: 63 },
            Size { w: 150, h: 125 },
            Size { w: 300, h: 250 }
        ]
    );

    assert_eq!(
        dz.level_tiles,
        vec![
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 1, h: 1 },
            Size { w: 2, h: 1 }
        ]
    );
}

// TODO: figure it out
// #[test]
// fn test_get_tile() {
//     let slide = OpenSlide::open(common::boxes_tiff()).unwrap();
//     let dz = DeepZoom::new(&slide, 254, 1, false);
//
//     // TODO: figure it out
//     assert_eq!(
//         dz.read_tile(9, Address { x: 1, y: 0 })
//             .unwrap()
//             .dimensions(),
//         (47, 250)
//     );
// }

#[test]
#[should_panic(expected = "Level 10 out of range")]
fn test_get_tile_bad_level() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();
    let dz = DeepZoom::new(&slide, 254, 1, false);

    dz.read_tile(10, Address { x: 0, y: 0 }).unwrap();
}

#[test]
fn test_get_tile_coordinates() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();
    let dz = DeepZoom::new(&slide, 254, 1, false);

    let expected = Region {
        address: Address { x: 253, y: 0 },
        level: 0,
        size: Size { w: 47, h: 250 },
    };
    assert_eq!(dz.tile_region(9, Address { x: 1, y: 0 }).unwrap(), expected);
}

// TODO: figure it out
// #[test]
// fn test_get_tile_dimensions() {
//     let slide = OpenSlide::open(common::boxes_tiff()).unwrap();
//     let dz = DeepZoom::new(&slide, 254, 1, false);
//
//     // TODO: figure it out
//     let expected = Size { w: 47, h: 250 };
//     assert_eq!(dz.tile_size(9, Address { x: 1, y: 0 }).unwrap(), expected);
// }
