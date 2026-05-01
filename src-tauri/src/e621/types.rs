use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostsResponse {
    pub posts: Vec<Post>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Post {
    pub id: u64,
    pub file: PostFile,
    #[serde(default)]
    pub preview: PostPreview,
    #[serde(default)]
    pub sample: PostSample,
    #[serde(default)]
    pub tags: PostTags,
    #[serde(default)]
    pub is_favorited: bool,
    #[serde(default)]
    pub score: PostScore,
    #[serde(default)]
    pub fav_count: i64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PostScore {
    #[serde(default)]
    pub up: i64,
    #[serde(default)]
    pub down: i64,
    #[serde(default)]
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credentials {
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostFile {
    pub ext: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PostPreview {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PostSample {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PostTags {
    #[serde(default)]
    pub artist: Vec<String>,
    #[serde(default)]
    pub copyright: Vec<String>,
    #[serde(default)]
    pub character: Vec<String>,
    #[serde(default)]
    pub species: Vec<String>,
    #[serde(default)]
    pub general: Vec<String>,
    #[serde(default)]
    pub meta: Vec<String>,
    #[serde(default)]
    pub lore: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub post_count: u64,
    pub category: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TagsResponse {
    List(Vec<Tag>),
    Empty { tags: Vec<Tag> },
}
