use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureRoot { items: Vec<MemoryItem> }

/// Abstraction to allow swapping fixture/JSON‑RPC backends.
#[allow(async_fn_in_trait)]
pub trait MemoryAgentClient {
    async fn search(&self, q: &str, k: u32) -> anyhow::Result<Vec<MemoryItem>>;
    async fn neighbors(&self, _id: &str, _depth: u8) -> anyhow::Result<Vec<MemoryItem>> {
        Ok(Vec::new())
    }
}

/// Fixture‑backed client (Phase‑1 offline path). Reads CONTEXT_MCP_FIXTURE JSON.
pub struct FixtureMemoryClient {
    pub fixture_path: String,
}

#[async_trait::async_trait]
impl MemoryAgentClient for FixtureMemoryClient {
    async fn search(&self, _q: &str, _k: u32) -> anyhow::Result<Vec<MemoryItem>> {
        let data = std::fs::read(&self.fixture_path)?;
        let root: FixtureRoot = serde_json::from_slice(&data)?;
        Ok(root.items)
    }
}

