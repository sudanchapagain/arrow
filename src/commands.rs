use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

use crate::config::Config;
use crate::djot;
use crate::fs;

fn resolve_entry_path(entry: &str) -> Result<PathBuf> {
    let entry_path = PathBuf::from(entry);
    if entry_path.is_dir() {
        Ok(entry_path)
    } else {
        let config = Config::load_config().context("failed to load configuration")?;
        config.get_path(entry).map(|p| p.to_path_buf())
    }
}

pub async fn build_command(entry: &str) -> Result<()> {
    let build_path = resolve_entry_path(entry)?;

    let src_dir = build_path.join("src");

    if !src_dir.exists() {
        anyhow::bail!(
            "error: missing `src` directory. specify a valid project with the --entry flag."
        );
    }

    let dist_dir = build_path.join("dist");
    let assets_dir = src_dir.join("assets");

    println!("starting build...");
    println!("source: {src_dir:?}");
    println!("destination: {dist_dir:?}");

    fs::prepare_directories(&dist_dir).context("error preparing directories")?;

    if let Err(e) = fs::copy_assets(&assets_dir, &dist_dir.join("assets")) {
        eprintln!("warning: error copying assets: {e}");
    }

    let files = fs::collect_djot_files(&src_dir).context("error collecting Djot files")?;

    files.par_iter().for_each(|file| {
        if let Err(e) = djot::process_djot_file(file, &src_dir, &dist_dir) {
            eprintln!("error processing file {file:?}: {e}");
        }
    });

    println!("build completed!");

    Ok(())
}

pub fn new_command(entry: &str) -> Result<()> {
    use chrono::Local;
    use inquire::{Confirm, Text};
    use std::io::Write;

    let workspace_path = resolve_entry_path(entry)?;
    let src_path = workspace_path.join("src");

    let file_name = Text::new("enter file name (without extension):")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "file name cannot be empty.".into(),
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()
        .context("failed to get file name")?;

    let description = Text::new("enter description (optional):")
        .prompt()
        .context("failed to get description")?;

    let file_path = src_path.join(format!("{}.djot", file_name.trim()));

    if file_path.exists() {
        return Err(anyhow::anyhow!(
            "error: a file with this name already exists: {:?}",
            file_path
        ));
    }

    let mut file = std::fs::File::create(&file_path)
        .with_context(|| format!("error creating file: {file_path:?}"))?;

    let date = Local::now().format("%Y-%m-%d").to_string();
    let front_matter = format!(
        "---\ntitle: {}\ndesc: {}\ndate: {}\nstatus: false\n---\n",
        file_name.trim(),
        description.trim(),
        date
    );

    file.write_all(front_matter.as_bytes())
        .context("error writing front matter to file")?;

    println!("new entry created at: {file_path:?}");

    let open_in_editor = Confirm::new("open file in editor?")
        .with_default(true)
        .prompt()
        .context("failed to get editor confirmation")?;

    if open_in_editor {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let mut command = std::process::Command::new(editor);
        command.arg(&file_path);
        command
            .spawn()
            .with_context(|| format!("error opening editor for file: {file_path:?}"))?
            .wait()
            .context("error waiting for editor to close")?;
    }

    Ok(())
}

pub fn status_command(entry: &str) -> Result<()> {
    use colored::Colorize;

    let workspace_path = resolve_entry_path(entry)?;
    let src_dir = workspace_path.join("src");

    let files = fs::collect_djot_files(&src_dir).context("error collecting Djot files")?;

    let mut entries: Vec<(PathBuf, bool)> = Vec::new();

    for file in files {
        let content = std::fs::read_to_string(&file)
            .with_context(|| format!("failed to read file {file:?}"))?;
        let (metadata, _) = djot::parse_front_matter(&content)?;
        entries.push((file, metadata.status));
    }

    entries.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{:<60} {:<10}", "file Path", "status");
    println!("{}", "-".repeat(75));

    for (path, status) in entries {
        let relative_path = path
            .strip_prefix(&src_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        let status_str = if status {
            "true".green()
        } else {
            "false".red()
        };
        println!("{relative_path:<60} {status_str}");
    }

    Ok(())
}

pub async fn serve_command(port: u16, entry: &str) -> Result<()> {
    use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::convert::Infallible;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
    };
    use std::time::Duration;
    use tokio::signal;
    use warp::Filter;

    println!("building initial site...");
    build_command(entry).await.context("initial build failed")?;

    let build_path = resolve_entry_path(entry)?;

    let src_dir = build_path.join("src");
    let dist_dir = build_path.join("dist");

    let server_port = if port == 0 {
        Config::load_config().map(|c| c.server.port).unwrap_or(8000)
    } else {
        port
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_watcher = shutdown.clone();

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .context("failed to create file watcher")?;

    watcher
        .watch(&src_dir, RecursiveMode::Recursive)
        .context("failed to start watching source directory")?;

    let dist_dir_clone = dist_dir.clone();
    let dist_dir_canon = dist_dir.clone().canonicalize().unwrap();

    let extensionless_route = warp::path::full().and_then(move |path: warp::path::FullPath| {
        let dist_dir = dist_dir_clone.clone();
        async move {
            let path_str = path.as_str();

            if path_str.contains('.') {
                return Err(warp::reject::not_found());
            }

            let html_path = if path_str == "/" {
                dist_dir.join("index.html")
            } else {
                let clean_path = path_str.trim_start_matches('/');
                dist_dir.join(format!("{clean_path}.html"))
            };

            if html_path.exists() {
                match tokio::fs::read(html_path).await {
                    Ok(contents) => Ok(warp::reply::html(
                        String::from_utf8_lossy(&contents).to_string(),
                    )),
                    Err(_) => Err(warp::reject::not_found()),
                }
            } else {
                Err(warp::reject::not_found())
            }
        }
    });

    let dir_route = warp::fs::dir(dist_dir.clone());

    let routes = extensionless_route
        .or(dir_route)
        .recover(move |_: warp::Rejection| {
            let dist_dir = dist_dir.clone();
            async move {
                let not_found_path = dist_dir.join("404.html");
                if not_found_path.exists() {
                    match tokio::fs::read(not_found_path).await {
                        Ok(contents) => Ok::<_, Infallible>(warp::reply::with_status(
                            warp::reply::html(String::from_utf8_lossy(&contents).to_string()),
                            warp::http::StatusCode::NOT_FOUND,
                        )),
                        Err(_) => Ok::<_, Infallible>(warp::reply::with_status(
                            warp::reply::html("404 - Page Not Found".to_string()),
                            warp::http::StatusCode::NOT_FOUND,
                        )),
                    }
                } else {
                    Ok::<_, Infallible>(warp::reply::with_status(
                        warp::reply::html("404 - Page Not Found".to_string()),
                        warp::http::StatusCode::NOT_FOUND,
                    ))
                }
            }
        });

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], server_port), async {
            shutdown_rx.await.ok();
        });

    println!("serving at http://{addr}/");

    let server_handle = tokio::spawn(server);

    let entry_clone = entry.to_string();
    let mut last_rebuild = Instant::now();
    let debounce_duration = Duration::from_millis(200);

    let _ = std::thread::spawn(move || {
        let dist_dir = dist_dir_canon;
        while !shutdown_watcher.load(Ordering::SeqCst) {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if shutdown_watcher.load(Ordering::SeqCst) {
                        break;
                    }

                    let should_rebuild = matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    );

                    if should_rebuild {
                        let has_relevant_files = event.paths.iter().any(|path| {
                            if let Ok(abs_path) = path.canonicalize() {
                                if abs_path.starts_with(&dist_dir) {
                                    return false;
                                }
                            }
                            if let Some(file_name) = path.file_name() {
                                let file_str = file_name.to_string_lossy();
                                file_str.ends_with(".djot")
                                    || file_str.ends_with(".html")
                                    || file_str.ends_with(".css")
                                    || file_str.ends_with(".js")
                            } else {
                                false
                            }
                        });

                        if has_relevant_files && last_rebuild.elapsed() > debounce_duration {
                            println!("change detected. Rebuilding...");
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            if let Err(e) = rt.block_on(build_command(&entry_clone)) {
                                eprintln!("rebuild failed: {e}");
                            }
                            last_rebuild = Instant::now();
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("watch error: {e}"),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(e) => {
                    eprintln!("watcher channel error: {e}");
                    break;
                }
            }
        }
    });

    signal::ctrl_c().await?;

    println!("Received shutdown signal, exiting...");
    shutdown.store(true, Ordering::SeqCst);
    let _ = shutdown_tx.send(());

    server_handle.await.context("server task failed")?;

    Ok(())
}
