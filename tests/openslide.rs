use math::round;
use openslide_rs::{Address, OpenSlide, Region, Size};
use std::path::Path;

mod common;

#[test]
fn test_detect_format() {
    let missing_file = common::missing_file();
    let expected = Some(format!("Missing image file: {}", missing_file.display()));
    assert_eq!(OpenSlide::detect_vendor(missing_file).err(), expected);

    let unsupported_file = common::unsupported_file();
    let expected = Some(format!(
        "Unsupported image file: {}",
        unsupported_file.display()
    ));
    assert_eq!(OpenSlide::detect_vendor(unsupported_file).err(), expected);

    let boxes_tiff_path = common::boxes_tiff();
    assert_eq!(
        "generic-tiff",
        OpenSlide::detect_vendor(boxes_tiff_path).unwrap()
    );
}

#[test]
fn test_open_errors() {
    let missing_file = common::missing_file();
    let expected = Some(format!("Missing image file: {}", missing_file.display()));
    assert_eq!(OpenSlide::open(missing_file).err(), expected);

    let unsupported_file = common::unsupported_file();
    let expected = Some(format!(
        "Unsupported image file: {}",
        unsupported_file.display()
    ));
    assert_eq!(OpenSlide::open(unsupported_file).err(), expected);

    let unopenable_tiff = common::unopenable_tiff();
    let expected = Some(String::from("Unsupported TIFF compression: 52479"));
    assert_eq!(OpenSlide::open(unopenable_tiff).err(), expected);
}

#[test]
fn test_basic_metadata() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();

    assert_eq!(slide.level_count().unwrap(), 4);

    assert_eq!(slide.level_dimensions(0).unwrap(), Size { w: 300, h: 250 });
    assert_eq!(slide.level_dimensions(1).unwrap(), Size { w: 150, h: 125 });
    assert_eq!(slide.level_dimensions(2).unwrap(), Size { w: 75, h: 62 });
    assert_eq!(slide.level_dimensions(3).unwrap(), Size { w: 37, h: 31 });
    assert_eq!(slide.dimensions().unwrap(), Size { w: 300, h: 250 });

    assert_eq!(slide.level_downsample(0).unwrap(), 1.);
    assert_eq!(slide.level_downsample(1).unwrap(), 2.);
    assert_eq!(round::floor(slide.level_downsample(2).unwrap(), 0), 4.);
    assert_eq!(round::floor(slide.level_downsample(3).unwrap(), 0), 8.);

    assert_eq!(slide.best_level_for_downsample(0.5).unwrap(), 0);
    assert_eq!(slide.best_level_for_downsample(3.).unwrap(), 1);
    assert_eq!(slide.best_level_for_downsample(37.).unwrap(), 3);
}

#[test]
fn test_properties() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();

    assert_eq!(
        slide.properties.get("openslide.vendor").unwrap(),
        "generic-tiff"
    );
}

#[test]
fn test_read_region() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();

    let tile = slide
        .read_region(Region {
            address: Address { x: 0, y: 0 },
            level: 1,
            size: Size { w: 400, h: 200 },
        })
        .unwrap();
    assert_eq!(tile.dimensions(), (400, 200));

    tile.save(Path::new("tests/artifacts/test_read_region.png"))
        .unwrap();
}

#[test]
fn test_thumbnail() {
    let slide = OpenSlide::open(common::boxes_tiff()).unwrap();

    let thumbnail = slide.thumbnail(Size { w: 100, h: 100 }).unwrap();
    assert_eq!(thumbnail.dimensions(), (100, 83));

    thumbnail
        .save(Path::new("tests/artifacts/test_thumbnail.png"))
        .unwrap();
}

#[test]
#[should_panic(expected = "Associated image __missing does not exist")]
fn test_associated_images() {
    let slide = OpenSlide::open(common::small_svs()).unwrap();

    assert_eq!(
        slide.associated_image("thumbnail").unwrap().dimensions(),
        (16, 16)
    );
    slide.associated_image("__missing").unwrap();
}

#[test]
#[should_panic(expected = "Corrupt JPEG data: 226 extraneous bytes before marker 0xd9")]
fn test_read_bad_region() {
    let slide = OpenSlide::open(common::unreadable_svs()).unwrap();

    slide
        .read_region(Region {
            address: Address { x: 0, y: 0 },
            level: 0,
            size: Size { w: 16, h: 16 },
        })
        .unwrap();
}

#[test]
#[should_panic(expected = "TIFFRGBAImageGet failed")]
fn test_read_bad_associated_image() {
    let slide = OpenSlide::open(common::unreadable_svs()).unwrap();
    slide.associated_image("thumbnail").unwrap();
}
