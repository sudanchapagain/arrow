package md

import (
	"bytes"
	"fmt"
	"html/template"
	"log"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/adrg/frontmatter"
	"github.com/alecthomas/chroma/formatters/html"
	"github.com/alecthomas/chroma/styles"
	"github.com/yuin/goldmark"
	highlighting "github.com/yuin/goldmark-highlighting"
	"github.com/yuin/goldmark/extension"
	gmhtml "github.com/yuin/goldmark/renderer/html"
)

type Metadata struct {
	Title  string    `yaml:"title"`
	Desc   string    `yaml:"desc"`
	Date   time.Time `yaml:"date"`
	Status bool      `yaml:"status"`
	JS     string    `yaml:"js"`
}

type Page struct {
	Title      string
	Desc       string
	Date       time.Time
	Content    template.HTML
	InlineCSS  template.HTML
	InlineJS   template.HTML
	AssetsPath string
}

var markdownParser = goldmark.New(
	goldmark.WithExtensions(
		extension.GFM,
		highlighting.NewHighlighting(
			highlighting.WithStyle("monokailight"),
			highlighting.WithFormatOptions(html.WithLineNumbers(false)),
		),
	),
	goldmark.WithRendererOptions(gmhtml.WithUnsafe()),
)

func ProcessMarkdownFile(mdPath, srcDir, distDir string) error {
	content, err := os.ReadFile(mdPath)
	if err != nil {
		return fmt.Errorf("failed to read file %s: %w", mdPath, err)
	}

	var metadata Metadata
	body, err := frontmatter.Parse(bytes.NewReader(content), &metadata)
	if err != nil {
		return fmt.Errorf("failed to parse front matter for %s: %w", mdPath, err)
	}

	if !metadata.Status {
		return nil
	}

	metadata.Title = getDefaultTitle(mdPath, metadata.Title)
	htmlContent, err := markdownToHTML(body)
	if err != nil {
		return fmt.Errorf("failed to convert markdown to HTML for %s: %w", mdPath, err)
	}

	inlineCSS, err := generateHighlightCSS("monokailight")
	if err != nil {
		return fmt.Errorf("failed to generate CSS: %w", err)
	}

	inlineJS := template.HTML(metadata.JS)
	page := createPage(metadata, htmlContent, inlineCSS, inlineJS)

	destPath := getDestPath(mdPath, srcDir, distDir)

	if err := os.MkdirAll(filepath.Dir(destPath), os.ModePerm); err != nil {
		return fmt.Errorf("failed to create directory %s: %w", destPath, err)
	}

	return renderHTMLPage(page, srcDir, destPath)
}

func getDefaultTitle(mdPath, title string) string {
	if title == "" {
		return strings.TrimSuffix(filepath.Base(mdPath), ".md")
	}
	return title
}

func markdownToHTML(content []byte) (template.HTML, error) {
	var buf bytes.Buffer
	err := markdownParser.Convert(content, &buf)
	if err != nil {
		return "", fmt.Errorf("failed to convert markdown to HTML: %w", err)
	}
	return template.HTML(buf.String()), nil
}

func generateHighlightCSS(styleName string) (template.HTML, error) {
	style := styles.Get(styleName)
	if style == nil {
		return "", fmt.Errorf("style %s not found", styleName)
	}

	var buf bytes.Buffer
	err := html.New().WriteCSS(&buf, style)
	if err != nil {
		return "", fmt.Errorf("error writing CSS: %w", err)
	}

	return template.HTML(buf.String()), nil
}

func createPage(metadata Metadata, content template.HTML, inlineCSS template.HTML, inlineJS template.HTML) Page {
	return Page{
		Title:      metadata.Title,
		Desc:       metadata.Desc,
		Date:       metadata.Date,
		Content:    content,
		InlineCSS:  inlineCSS,
		InlineJS:   inlineJS,
		AssetsPath: "/assets",
	}
}

func getDestPath(mdPath, srcDir, distDir string) string {
	relPath, err := filepath.Rel(srcDir, mdPath)
	if err != nil {
		log.Fatalf("failed to get relative path for %s: %v", mdPath, err)
	}

	relPath = strings.TrimSuffix(relPath, ".md") + ".html"
	destPath := filepath.Join(distDir, relPath)
	return destPath
}

func renderHTMLPage(page Page, srcPath string, destPath string) error {
	layoutTemplatePath := filepath.Join(filepath.Dir(srcPath), "src", "layout.html")
	layoutTemplate, err := template.ParseFiles(layoutTemplatePath)
	if err != nil {
		return fmt.Errorf("failed to parse layout template: %w", err)
	}

	outputFile, err := os.Create(destPath)
	if err != nil {
		return fmt.Errorf("failed to create output file %s: %w", destPath, err)
	}
	defer outputFile.Close()

	return layoutTemplate.Execute(outputFile, page)
}
