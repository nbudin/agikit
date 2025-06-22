use include_dir::Dir;

use crate::resources::{
    dirs::{ResourceDirDecodeOptions, ResourceDirs},
    resource_collection::{ResourceCollection, ResourceCollectionVersionData},
};

static TEST_DATA_DIR: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/test_data");

pub fn contest2_template_dir<'a>() -> &'a Dir<'static> {
    TEST_DATA_DIR.get_dir("AGI_Contest_2_Template").unwrap()
}

pub fn contest2_template_resources() -> ResourceCollection<Dir<'static>> {
    let file_provider = contest2_template_dir();
    let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();

    ResourceCollection::new(
        ResourceCollectionVersionData::AGI2,
        file_provider.clone(),
        dirs,
    )
}

pub fn uriquest_dir<'a>() -> &'a Dir<'static> {
    TEST_DATA_DIR.get_dir("uriquest").unwrap()
}

pub fn uriquest_resources() -> ResourceCollection<Dir<'static>> {
    let file_provider = uriquest_dir();
    let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();

    ResourceCollection::new(
        ResourceCollectionVersionData::AGI2,
        file_provider.clone(),
        dirs,
    )
}

pub fn v_the_graphical_adventure_dir<'a>() -> &'a Dir<'static> {
    TEST_DATA_DIR.get_dir("VTheGraphicalAdventureDemo").unwrap()
}

pub fn v_the_graphical_adventure_resources() -> ResourceCollection<Dir<'static>> {
    let file_provider = v_the_graphical_adventure_dir();
    let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
        file_provider,
        game_id: "V".to_string(),
    })
    .unwrap();

    ResourceCollection::new(
        ResourceCollectionVersionData::AGI3("V".to_string()),
        file_provider.clone(),
        dirs,
    )
}
