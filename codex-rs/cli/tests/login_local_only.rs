use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn login_blocked_in_local_only() {
    // No flags: triggers ChatGPT/device login path which we block in local-only.
    let mut cmd = Command::cargo_bin("codex").expect("binary");
    cmd.env("CODEX_LOCAL_ONLY", "1").arg("login");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Login is disabled: local-only mode is active",
    ));
}
