use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn build_command_with_valid_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let src_dir = temp_dir.path().join("src");
    let templates_dir = temp_dir.path().join("templates");

    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&templates_dir)?;

    std::fs::write(
        src_dir.join("index.djot"),
        "---\nstatus: true\n---\n# Hello",
    )?;

    std::fs::write(
        templates_dir.join("layout.html"),
        "{{ page.content | safe }}",
    )?;

    let mut cmd = Command::cargo_bin("arrow")?;
    cmd.arg("build").arg("-e").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("build completed!"));

    assert!(temp_dir.path().join("dist").exists());
    assert!(temp_dir.path().join("dist").join("index.html").exists());

    Ok(())
}

#[test]
fn build_command_with_invalid_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let invalid_path = temp_dir.path().join("non_existent_dir");

    let mut cmd = Command::cargo_bin("arrow")?;
    cmd.arg("build").arg("-e").arg(&invalid_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "workspace '{}' not found in config",
            invalid_path.display()
        )));

    Ok(())
}

#[test]
fn build_command_missing_src_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    let mut cmd = Command::cargo_bin("arrow")?;
    cmd.arg("build").arg("-e").arg(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("missing `src` directory"));

    Ok(())
}

#[tokio::test]
async fn serve_command_works() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let src_dir = temp_dir.path().join("src");
    let templates_dir = temp_dir.path().join("templates");

    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&templates_dir)?;

    std::fs::write(
        src_dir.join("index.djot"),
        "---\nstatus: true\n---\n# Hello from serve",
    )?;

    std::fs::write(
        templates_dir.join("layout.html"),
        "{{ page.content | safe }}",
    )?;

    let mut cmd = Command::cargo_bin("arrow")?;
    cmd.arg("serve")
        .arg("-e")
        .arg(temp_dir.path())
        .arg("--port")
        .arg("8080");

    let mut child = cmd.spawn()?;

    sleep(Duration::from_secs(2));

    let resp = reqwest::get("http://127.0.0.1:8080/").await?;
    assert!(resp.status().is_success());
    let body = resp.text().await?;
    assert!(body.contains("Hello from serve"));

    child.kill()?;

    Ok(())
}

#[test]
fn status_command_displays_status() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        src_dir.join("file1.djot"),
        "---\nstatus: true\n---\n# File 1",
    )?;
    std::fs::write(
        src_dir.join("file2.djot"),
        "---\nstatus: false\n---\n# File 2",
    )?;
    std::fs::write(
        src_dir.join("file3.djot"),
        "---\nstatus: true\n---\n# File 3",
    )?;

    let mut cmd = Command::cargo_bin("arrow")?;
    cmd.arg("status").arg("-e").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("file1.djot").and(predicate::str::contains("true")))
        .stdout(predicate::str::contains("file2.djot").and(predicate::str::contains("false")))
        .stdout(predicate::str::contains("file3.djot").and(predicate::str::contains("true")));

    Ok(())
}
