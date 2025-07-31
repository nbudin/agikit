use include_dir::Dir;

use crate::{
    agi_version::AGIVersion,
    project::{Project, ProjectConfig},
    resources::{
        dirs::{ResourceDirDecodeOptions, ResourceDirs},
        resource_collection::{ResourceCollection, ResourceCollectionVersionData},
    },
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

pub fn uriquest() -> Project {
    Project::new(
        uriquest_dir(),
        Some(ProjectConfig::new(
            AGIVersion::new(2, 917),
            "URIQUEST".to_string(),
        )),
    )
}

pub fn kq4demo_dir<'a>() -> &'a Dir<'static> {
    TEST_DATA_DIR.get_dir("kq4demo").unwrap()
}

pub fn kq4demo_resources() -> ResourceCollection<Dir<'static>> {
    let file_provider = kq4demo_dir();
    let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
        file_provider,
        game_id: "DM".to_string(),
    })
    .unwrap();

    ResourceCollection::new(
        ResourceCollectionVersionData::AGI3("DM".to_string()),
        file_provider.clone(),
        dirs,
    )
}

pub fn kq4demo() -> Project {
    Project::new(
        kq4demo_dir(),
        Some(ProjectConfig::new(
            AGIVersion::new(3, 2102),
            "DM".to_string(),
        )),
    )
}
