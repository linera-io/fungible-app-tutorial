#![cfg(not(target_arch = "wasm32"))]

use linera_service::util::QuotedBashAndGraphQlScript;
use tokio::{process::Command, time::Duration};

#[tokio::test]
async fn test_script_in_readme() -> std::io::Result<()> {
    let script = QuotedBashAndGraphQlScript::from_markdown(
        format!("README.md"),
        Some(Duration::from_secs(3)),
    )?;

    // // Uncomment to keep the test file.
    // Command::new("cp")
    //     .arg(script.path())
    //     .arg("test.sh")
    //     .status()
    //     .await?;

    let status = Command::new("bash")
        .arg("-e")
        .arg("-x")
        .arg(script.path())
        .status()
        .await?;

    assert!(status.success());
    Ok(())
}
