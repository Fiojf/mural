use std::fs;
use tempfile::tempdir;

#[test]
fn lists_supported_extensions_only() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.jpg"), b"x").unwrap();
    fs::write(dir.path().join("b.PNG"), b"x").unwrap();
    fs::write(dir.path().join("c.heic"), b"x").unwrap();
    fs::write(dir.path().join("ignore.txt"), b"x").unwrap();
    fs::write(dir.path().join("video.mp4"), b"x").unwrap();
    fs::create_dir(dir.path().join("nested")).unwrap();
    fs::write(dir.path().join("nested/deep.jpg"), b"x").unwrap();

    let items = mural_lib::scan::list_local(dir.path());
    let names: Vec<_> = items.iter().map(|i| i.name.clone()).collect();
    assert!(names.contains(&"a.jpg".to_string()));
    assert!(names.contains(&"b.PNG".to_string()));
    assert!(names.contains(&"c.heic".to_string()));
    assert!(names.contains(&"video.mp4".to_string()));
    assert!(!names.contains(&"ignore.txt".to_string()));
    assert!(
        !names.contains(&"deep.jpg".to_string()),
        "scan should be non-recursive"
    );
}

#[test]
fn classifies_video_vs_image() {
    use mural_lib::scan::{classify, Kind};
    use std::path::Path;

    assert!(matches!(classify(Path::new("/x/a.png")), Some(Kind::Image)));
    assert!(matches!(classify(Path::new("/x/a.MP4")), Some(Kind::Video)));
    assert!(matches!(classify(Path::new("/x/a.MOV")), Some(Kind::Video)));
    assert!(classify(Path::new("/x/readme.txt")).is_none());
}
