use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWithAliases {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub details: Option<String>,
    pub base_image: Option<String>,
    pub is_builtin: bool,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInput {
    pub name: String,
    pub description: Option<String>,
    pub details: Option<String>,
    pub base_image: Option<String>,
    pub aliases: Vec<String>,
}
