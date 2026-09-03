use super::error::{VndbError, VndbResult};
use super::rate_limit::RateLimiter;
use crate::models::{
    VndbAuthInfo, VndbCharacter, VndbResponse, VndbSearchResult, VndbVnDetail,
};

pub const VNDB_API_BASE: &str = "https://api.vndb.org/kana";

pub fn search_body(query: &str) -> serde_json::Value {
    serde_json::json!({
        "filters": ["search", "=", query],
        "fields": "id, title, image.url, released, rating",
        "results": 10
    })
}

pub fn detail_body(vndb_id: &str) -> serde_json::Value {
    serde_json::json!({
        "filters": ["id", "=", vndb_id],
        "fields": "id, title, image.url, image.sexual, image.violence, released, rating, description, length, length_minutes, tags.id, tags.name, tags.rating, tags.spoiler, developers.id, developers.name, devstatus",
        "results": 1
    })
}

pub fn characters_body(vndb_id: &str) -> serde_json::Value {
    serde_json::json!({
        "filters": ["vn", "=", ["id", "=", vndb_id]],
        "fields": "id, name, original, aliases, image.url, image.sexual, image.violence, description, blood_type, height, weight, bust, waist, hips, cup, age, birthday, sex, vns.id, vns.role, vns.spoiler, traits.id, traits.name, traits.group_id, traits.group_name, traits.spoiler",
        "results": 50
    })
}

pub struct VndbClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
    limiter: RateLimiter,
}

pub fn user_vn_body(user_id: &str, vndb_id: &str) -> serde_json::Value {
    serde_json::json!({
        "user": user_id,
        "filters": ["id", "=", vndb_id],
        "fields": "id, vote, labels.id, labels.label, started, finished",
        "results": 1
    })
}

pub fn status_body(label_id: i32) -> serde_json::Value {
    let labels_unset: Vec<i32> = [1, 2, 3, 4, 5]
        .into_iter()
        .filter(|id| *id != label_id)
        .collect();
    serde_json::json!({
        "labels_set": [label_id],
        "labels_unset": labels_unset
    })
}

pub fn vote_body(vote: i32) -> serde_json::Value {
    serde_json::json!({ "vote": vote })
}

pub fn unvote_body() -> serde_json::Value {
    serde_json::json!({ "vote": null })
}

impl VndbClient {

    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("Poketto/0.1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            http,
            base_url: VNDB_API_BASE.to_string(),
            token: None,
            limiter: RateLimiter::vndb(),
        }
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.trim_end_matches('/').to_string();
        self
    }

    pub fn set_token(&mut self, token: Option<String>) {
        self.token = token;
    }

    #[tracing::instrument(skip(self))]
    pub async fn search(&self, query: &str) -> VndbResult<Vec<VndbSearchResult>> {
        let response: VndbResponse<VndbSearchResult> =
            self.post("vn", search_body(query)).await?;
        Ok(response.results)
    }

    #[tracing::instrument(skip(self))]
    pub async fn detail(&self, vndb_id: &str) -> VndbResult<VndbVnDetail> {
        let response: VndbResponse<VndbVnDetail> = self.post("vn", detail_body(vndb_id)).await?;
        response
            .results
            .into_iter()
            .next()
            .ok_or_else(|| VndbError::NotFound(vndb_id.to_string()))
    }

    #[tracing::instrument(skip(self))]
    pub async fn characters(&self, vndb_id: &str) -> VndbResult<Vec<VndbCharacter>> {
        let response: VndbResponse<VndbCharacter> =
            self.post("character", characters_body(vndb_id)).await?;
        Ok(response.results)
    }

    #[tracing::instrument(skip(self))]
    pub async fn auth_info(&self) -> VndbResult<VndbAuthInfo> {
        let token = self
            .token
            .clone()
            .ok_or_else(|| VndbError::AuthRequired("No VNDB token configured".to_string()))?;
        self.limiter.wait().await;
        let response = self
            .http
            .get(format!("{}/authinfo", self.base_url))
            .header("Authorization", format!("Token {token}"))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(VndbError::AuthRequired("Invalid token".to_string()));
        }
        response.json().await.map_err(VndbError::from)
    }

    fn token(&self) -> VndbResult<String> {
        self.token
            .clone()
            .ok_or_else(|| VndbError::AuthRequired("No VNDB token configured".to_string()))
    }

    #[tracing::instrument(skip(self))]
    pub async fn user_vn(
        &self,
        user_id: &str,
        vndb_id: &str,
    ) -> VndbResult<Option<crate::models::VndbUserListItem>> {
        let token = self.token()?;
        self.limiter.wait().await;
        let response = self
            .http
            .post(format!("{}/ulist", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Token {token}"))
            .json(&user_vn_body(user_id, vndb_id))
            .send()
            .await?;
        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(VndbError::Api(message));
        }
        let list: VndbResponse<crate::models::VndbUserListItem> =
            response.json().await.map_err(VndbError::from)?;
        Ok(list.results.into_iter().next())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_status(&self, vndb_id: &str, label_id: i32) -> VndbResult<()> {
        self.patch_ulist(vndb_id, status_body(label_id)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_vote(&self, vndb_id: &str, vote: i32) -> VndbResult<()> {
        self.patch_ulist(vndb_id, vote_body(vote)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove_vote(&self, vndb_id: &str) -> VndbResult<()> {
        self.patch_ulist(vndb_id, unvote_body()).await
    }

    #[tracing::instrument(skip(self, body))]
    async fn patch_ulist(&self, vndb_id: &str, body: serde_json::Value) -> VndbResult<()> {
        let token = self.token()?;
        self.limiter.wait().await;
        let response = self
            .http
            .patch(format!("{}/ulist/{vndb_id}", self.base_url))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Token {token}"))
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(VndbError::Api(message));
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, body))]
    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> VndbResult<T> {
        self.limiter.wait().await;
        let response = self
            .http
            .post(format!("{}/{}", self.base_url, path))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(VndbError::Api(message));
        }
        response.json().await.map_err(VndbError::from)
    }
}

impl Default for VndbClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_body_matches_legacy_query() {
        assert_eq!(
            search_body("muv-luv"),
            serde_json::json!({
                "filters": ["search", "=", "muv-luv"],
                "fields": "id, title, image.url, released, rating",
                "results": 10
            })
        );
    }

    #[test]
    fn detail_body_targets_single_id() {
        let body = detail_body("v17");
        assert_eq!(body["filters"], serde_json::json!(["id", "=", "v17"]));
        assert_eq!(body["results"], serde_json::json!(1));
        assert!(body["fields"].as_str().expect("fields").contains("developers.name"));
        assert!(body["fields"].as_str().expect("fields").contains("devstatus"));
    }

    #[test]
    fn characters_body_filters_by_vn() {
        let body = characters_body("v17");
        assert_eq!(
            body["filters"],
            serde_json::json!(["vn", "=", ["id", "=", "v17"]])
        );
        assert_eq!(body["results"], serde_json::json!(50));
    }

    #[test]
    fn search_response_fixture_parses() {
        let json = r#"{"results": [{"id": "v17", "title": "Muv-Luv", "image": {"url": "https://img.jpg"}, "released": "2003-02-28", "rating": 8.55}]}"#;
        let response: VndbResponse<VndbSearchResult> =
            serde_json::from_str(json).expect("fixture parses");
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Muv-Luv");
        assert_eq!(response.results[0].image.as_ref().expect("image").sexual, 0.0);
    }

    #[test]
    fn detail_response_fixture_parses() {
        let json = r#"{"results": [{"id": "v17", "title": "Muv-Luv", "description": "A story.", "length": 3, "length_minutes": 3000, "tags": [{"id": "g1", "name": "Drama", "rating": 8.0, "spoiler": 0}], "developers": [{"id": "p1", "name": "Age"}]}]}"#;
        let response: VndbResponse<VndbVnDetail> =
            serde_json::from_str(json).expect("fixture parses");
        let detail = &response.results[0];
        assert_eq!(detail.developers.as_ref().expect("devs")[0].name, "Age");
        assert_eq!(detail.tags.as_ref().expect("tags")[0].spoiler, 0);
        assert_eq!(detail.devstatus, None);
        let status_json = r#"{"results": [{"id": "v17", "title": "Muv-Luv", "devstatus": 1}]}"#;
        let status: VndbResponse<VndbVnDetail> =
            serde_json::from_str(status_json).expect("fixture parses");
        assert_eq!(status.results[0].devstatus, Some(1));
    }

    #[test]
    fn characters_response_fixture_parses() {
        let json = r#"{"results": [{"id": "c1", "name": "Meiya", "aliases": ["Mitsu"], "birthday": [7, 22], "sex": ["F"], "vns": [{"id": "v17", "role": "main", "spoiler": 0}], "traits": [{"id": "t1", "name": "Twintails", "spoiler": 0}]}]}"#;
        let response: VndbResponse<VndbCharacter> =
            serde_json::from_str(json).expect("fixture parses");
        let character = &response.results[0];
        assert_eq!(character.traits.as_ref().expect("traits")[0].name, "Twintails");
        assert_eq!(character.vns.as_ref().expect("vns")[0].role, "main");
    }

    #[tokio::test]
    async fn auth_info_without_token_fails_before_io() {
        let client = VndbClient::new().with_base_url("http://127.0.0.1:9");
        assert!(matches!(
            client.auth_info().await,
            Err(VndbError::AuthRequired(_))
        ));
    }

    #[test]
    fn user_vn_body_matches_legacy_query() {
        assert_eq!(
            user_vn_body("u1", "v17"),
            serde_json::json!({
                "user": "u1",
                "filters": ["id", "=", "v17"],
                "fields": "id, vote, labels.id, labels.label, started, finished",
                "results": 1
            })
        );
    }

    #[test]
    fn status_body_sets_one_label_and_unsets_rest() {
        assert_eq!(
            status_body(2),
            serde_json::json!({
                "labels_set": [2],
                "labels_unset": [1, 3, 4, 5]
            })
        );
    }

    #[test]
    fn vote_bodies_match_legacy_shapes() {
        assert_eq!(vote_body(8), serde_json::json!({ "vote": 8 }));
        assert_eq!(unvote_body(), serde_json::json!({ "vote": null }));
    }

    #[test]
    fn user_vn_response_fixture_parses() {
        let json = r#"{"results": [{"id": "v17", "vote": 8, "labels": [{"id": 2, "label": "Finished"}], "started": "2024-01-01", "finished": "2024-02-01"}]}"#;
        let response: VndbResponse<crate::models::VndbUserListItem> =
            serde_json::from_str(json).expect("fixture parses");
        let item = &response.results[0];
        assert_eq!(item.vote, Some(8));
        assert_eq!(item.labels.as_ref().expect("labels")[0].label, "Finished");
    }

    #[tokio::test]
    async fn vote_without_token_fails_before_io() {
        let client = VndbClient::new().with_base_url("http://127.0.0.1:9");
        assert!(matches!(
            client.set_vote("v17", 8).await,
            Err(VndbError::AuthRequired(_))
        ));
        assert!(matches!(
            client.user_vn("u1", "v17").await,
            Err(VndbError::AuthRequired(_))
        ));
    }
}
