use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq)]
pub enum StorySorting {
    Top,
    New,
    Best,
    Show,
    Ask,
    Job,
}

const TOP: &'static str = "top";
const BEST: &'static str = "best";
const NEW: &'static str = "new";
const SHOW: &'static str = "show";
const ASK: &'static str = "ask";
const JOB: &'static str = "job";

impl StorySorting {
    /// return all of the story sorting possible
    pub fn all() -> Vec<Self> {
        vec![
            StorySorting::Top,
            StorySorting::Best,
            StorySorting::New,
            StorySorting::Show,
            StorySorting::Ask,
            StorySorting::Job,
        ]
    }
    /// match url to StorySorting (supports both hash and path routing)
    pub fn from_url(url: &str) -> Option<Self> {
        let hash_url = url.strip_prefix("#").unwrap_or(url);
        Self::all().into_iter().find(|s| hash_url == s.to_str() || url == s.to_url())
    }
    /// return the str for assembling paths in warp
    pub fn to_str(&self) -> &str {
        match self {
            StorySorting::Top => TOP,
            StorySorting::Best => BEST,
            StorySorting::New => NEW,
            StorySorting::Show => SHOW,
            StorySorting::Ask => ASK,
            StorySorting::Job => JOB,
        }
    }

    pub fn to_url(&self) -> String {
        format!("#{}", self.to_str())
    }
}

impl Default for StorySorting {
    fn default() -> Self {
        StorySorting::Top
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StoryPageData {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub descendants: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    pub r#type: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

impl StoryPageData {
    /// derive a preview of this StoragePageData
    pub fn preview(&self) -> StoryItem {
        StoryItem {
            id: self.id,
            title: self.title.to_owned(),
            url: self.url.to_owned(),
            text: self.text.to_owned(),
            by: self.by.to_owned(),
            score: self.score,
            descendants: self.descendants,
            time: self.time.to_owned(),
            kids: self.kids.to_owned(),
            r#type: self.r#type.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Comment {
    pub id: i64,
    /// there will be no by field if the comment was deleted
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub text: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    #[serde(default)]
    pub sub_comments: Vec<Comment>,
    pub r#type: String,
}

impl Comment {
    /// attempt to extract comment id from url (supports both hash and path routing)
    pub fn id_from_url(url: &str) -> Option<i64> {
        let target_url = url.strip_prefix("#").unwrap_or(url);
        if target_url.starts_with("comment/") || url.starts_with("/comment") {
            let splinters = if target_url.starts_with("comment/") {
                target_url.split("/").collect::<Vec<_>>()
            } else {
                url.split("/").collect::<Vec<_>>()
            };
            if splinters.len() >= 2 {
                if target_url.starts_with("comment/") {
                    splinters[1].parse::<i64>().ok()
                } else {
                    splinters[2].parse::<i64>().ok()
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_url(comment_id: i64) -> String {
        format!("#comment/{}", comment_id)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StoryItem {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub descendants: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    pub r#type: String,
}

impl StoryItem {
    /// attempt to extract story id from url (supports both hash and path routing)
    pub fn id_from_url(url: &str) -> Option<i64> {
        let target_url = url.strip_prefix("#").unwrap_or(url);
        if target_url.starts_with("item/") || url.starts_with("/item") {
            let splinters = if target_url.starts_with("item/") {
                target_url.split("/").collect::<Vec<_>>()
            } else {
                url.split("/").collect::<Vec<_>>()
            };
            if splinters.len() >= 2 {
                if target_url.starts_with("item/") {
                    splinters[1].parse::<i64>().ok()
                } else {
                    splinters[2].parse::<i64>().ok()
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_url(story_id: i64) -> String {
        format!("#item/{}", story_id)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserData {
    pub id: String,
    pub karma: i64,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub submitted: Vec<i64>,
    #[serde(default)]
    pub stories: Vec<StoryItem>,
}

impl UserData {
    /// attempt to extract user id from url (supports both hash and path routing)
    pub fn id_from_url(url: &str) -> Option<String> {
        let target_url = url.strip_prefix("#").unwrap_or(url);
        if target_url.starts_with("user/") || url.starts_with("/user") {
            let splinters = if target_url.starts_with("user/") {
                target_url.split("/").collect::<Vec<_>>()
            } else {
                url.split("/").collect::<Vec<_>>()
            };
            if splinters.len() >= 2 {
                if target_url.starts_with("user/") {
                    Some(splinters[1].to_string())
                } else {
                    Some(splinters[2].to_string())
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn to_url(username: &str) -> String {
        format!("#user/{}", username)
    }
}
