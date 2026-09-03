use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbSearchResult {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub released: Option<String>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbImage {
    pub url: String,
    #[serde(default)]
    pub sexual: f64,
    #[serde(default)]
    pub violence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbResponse<T> {
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbVnDetail {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub description: Option<String>,
    pub length: Option<i32>,
    pub length_minutes: Option<i32>,
    #[serde(default)]
    pub devstatus: Option<i32>,
    pub tags: Option<Vec<VndbTag>>,
    pub developers: Option<Vec<VndbProducer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbTag {
    pub id: String,
    pub name: String,
    pub rating: f64,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbProducer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbCharacter {
    pub id: String,
    pub name: String,
    pub original: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub image: Option<VndbImage>,
    pub description: Option<String>,
    pub blood_type: Option<String>,
    pub height: Option<i32>,
    pub weight: Option<i32>,
    pub bust: Option<i32>,
    pub waist: Option<i32>,
    pub hips: Option<i32>,
    pub cup: Option<String>,
    pub age: Option<i32>,
    pub birthday: Option<Vec<i32>>,
    pub sex: Option<Vec<String>>,
    pub vns: Option<Vec<VndbCharacterVn>>,
    pub traits: Option<Vec<VndbTrait>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbTrait {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbCharacterVn {
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbUserListItem {
    pub id: String,
    pub vote: Option<i32>,
    pub labels: Option<Vec<VndbLabel>>,
    pub started: Option<String>,
    pub finished: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbLabel {
    pub id: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbAuthInfo {
    pub id: String,
    pub username: String,
}
