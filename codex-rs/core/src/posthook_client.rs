use tokio::task::JoinHandle;

pub fn spawn_posthook_client(server: String, params: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        use tokio::process::Command;
        let mut cmd = Command::new("codex-mcp-client");
        cmd.arg("--server")
            .arg(server)
            .arg("--connect-timeout-ms")
            .arg("650")
            .arg("--call-timeout-ms")
            .arg("700")
            .arg("--params")
            .arg(params)
            .kill_on_drop(true);
        let _ = cmd.spawn();
    })
}

#[cfg(test)]
mod tests {

    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    async fn run_posthook_client_once(server: String, params: String) {
        use tokio::process::Command;
        let mut cmd = Command::new("codex-mcp-client");
        cmd.arg("--server")
            .arg(server)
            .arg("--connect-timeout-ms")
            .arg("650")
            .arg("--call-timeout-ms")
            .arg("700")
            .arg("--params")
            .arg(params)
            .kill_on_drop(true);
        let _ = cmd.status().await;
    }

    #[tokio::test]
    async fn spawn_writes_params_via_fake_client() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("codex-mcp-client");
        let out = dir.path().join("params.json");
        let script = format!(
            "#!/usr/bin/env bash\necho \"$@\" > {}\nexit 0\n",
            out.display()
        );
        fs::write(&bin, script).unwrap();
        let mut perms = fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bin, perms).unwrap();

        let orig_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", dir.path().display(), orig_path);
        unsafe {
            std::env::set_var("PATH", &new_path);
        }

        let server = "stdio:/bin/true".to_string();
        let params =
            serde_json::json!({"tool":"codex.posthook.record","args":{"turn_id":"t"}}).to_string();
        run_posthook_client_once(server, params).await;

        let body = fs::read_to_string(&out).unwrap_or_default();
        assert!(!body.is_empty(), "expected fake client to write arguments");
    }
}
