use anyhow::{Context, Ok, Result, anyhow};
use chrono::NaiveDate;
use jotdown::Parser;
use jotdown::html::render_to_string;
use serde::{Deserialize, Serialize};
use std::fs;

use std::path::{Path, PathBuf};
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style};
use tera::{Context as TeraContext, Tera};

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub date: Option<NaiveDate>,
    #[serde(default = "default_status")]
    pub status: bool,
    pub js: Option<String>,
}

fn default_status() -> bool {
    false
}

#[derive(Debug, Serialize)]
pub struct Page {
    pub title: String,
    pub desc: String,
    pub date: String,
    pub content: String,
    pub inline_css: String,
    pub inline_js: String,
    pub assets_path: String,
}

pub fn process_djot_file(djot_path: &Path, src_dir: &Path, dist_dir: &Path) -> Result<()> {
    let content = fs::read_to_string(djot_path)
        .with_context(|| format!("failed to read file {djot_path:?}"))?;

    let (metadata, djot_content) = parse_front_matter(&content)?;

    if !metadata.status {
        return Ok(());
    }

    let title = get_default_title(djot_path, metadata.title);
    let html_content = djot_to_html(&djot_content)?;

    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let inline_css = css_for_theme_with_class_style(theme, ClassStyle::Spaced)?;
    let inline_js = metadata.js.unwrap_or_default();

    let page = create_page(
        title,
        metadata.desc,
        metadata.date,
        html_content,
        inline_css,
        inline_js,
    );

    let dest_path = get_dest_path(djot_path, src_dir, dist_dir)?;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {parent:?}"))?;
    }

    let site_root = src_dir
        .parent()
        .ok_or_else(|| anyhow!("could not determine site root"))?;
    let template_dir = site_root.join("templates");

    render_html_page(&page, &dest_path, &template_dir)?;

    Ok(())
}

pub fn parse_front_matter(content: &str) -> Result<(Metadata, String)> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        return Ok((Metadata::default(), content.to_string()));
    }

    let metadata_str = parts[1];
    let djot_content = parts[2].to_string();

    let metadata: Metadata =
        serde_yaml::from_str(metadata_str).context("failed to parse front matter YAML")?;

    Ok((metadata, djot_content))
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: None,
            desc: None,
            date: None,
            status: default_status(),
            js: None,
        }
    }
}

fn get_default_title(path: &Path, title: Option<String>) -> String {
    title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string()
    })
}

fn djot_to_html(content: &str) -> Result<String> {
    let parser = Parser::new(content);
    Ok(render_to_string(parser))
}

fn create_page(
    title: String,
    desc: Option<String>,
    date: Option<NaiveDate>,
    content: String,
    inline_css: String,
    inline_js: String,
) -> Page {
    Page {
        title,
        desc: desc.unwrap_or_default(),
        date: date.map_or("".to_string(), |d| d.format("%Y-%m-%d").to_string()),
        content,
        inline_css,
        inline_js,
        assets_path: "/assets".to_string(),
    }
}

fn get_dest_path(djot_path: &Path, src_dir: &Path, dist_dir: &Path) -> Result<PathBuf> {
    let relative_path = djot_path
        .strip_prefix(src_dir)
        .with_context(|| format!("failed to strip prefix {src_dir:?} from {djot_path:?}"))?;

    let html_path_buf = relative_path.with_extension("html");
    let html_file_name = html_path_buf
        .file_name()
        .ok_or_else(|| anyhow!("could not get file name for {:?}", relative_path))?;

    let dest_path = dist_dir
        .join(relative_path.parent().unwrap_or(Path::new("")))
        .join(html_file_name);

    Ok(dest_path)
}

fn render_html_page(page: &Page, dest_path: &Path, template_dir: &Path) -> Result<()> {
    if !template_dir.exists() {
        return Err(anyhow!(
            "template directory does not exist: {}",
            template_dir.display()
        ));
    }

    let template_pattern = template_dir.join("**/*.html");
    let tera = Tera::new(template_pattern.to_str().unwrap())?;

    let mut context = TeraContext::new();
    context.insert("page", page);

    let mut output_file = fs::File::create(dest_path)
        .with_context(|| format!("failed to create output file {dest_path:?}"))?;

    tera.render_to("layout.html", &context, &mut output_file)?;

    Ok(())
}
