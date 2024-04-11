use ::assert_matches::assert_matches;
use pickup::filemanager::{
    cache::{init, refresh},
    options::CollectionOptions,
};

const NUM_FILES: usize = 6;

#[test]
fn test_refresh_and_load() {
    let options = CollectionOptions {
        dir: String::from("../music"),
        ignores: None,
    };

    let result = refresh(options.clone());

    assert_matches!(result, Ok(_));
    let files = result.unwrap();
    assert_eq!(NUM_FILES, files.len());

    let load_result = init(options);

    assert_matches!(load_result, Ok(_));
    let loaded_files = load_result.unwrap();
    assert_eq!(files, loaded_files);
}
