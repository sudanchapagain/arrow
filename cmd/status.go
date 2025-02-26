package cmd

import (
	"bytes"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"arrow/config"

	"github.com/adrg/frontmatter"
)

type Metadata struct {
	Title  string    `yaml:"title"`
	Desc   string    `yaml:"desc"`
	Date   time.Time `yaml:"date"`
	Status bool      `yaml:"status"`
	JS     string    `yaml:"js"`
}

type FileEntry struct {
	Path   string
	Status bool
}

func Status(entry string) {
	cfg, err := config.LoadConfig()
	if err != nil {
		log.Println("Config file not found.")
		os.Exit(1)
	}

	workspace, err := cfg.GetPath(entry)
	if err != nil {
		log.Fatalf("Error getting path for entry '%s': %v", entry, err)
	}

	srcDir := filepath.Join(workspace, "src")

	files, err := collectMarkdownFilesInDir(srcDir)
	if err != nil {
		log.Println("Error collecting markdown files:", err)
		os.Exit(1)
	}

	var entries []FileEntry

	for _, file := range files {
		relativePath := strings.TrimPrefix(file, srcDir+"/")

		metadata, err := readMetadata(file)
		if err != nil {
			log.Printf("Error reading metadata for %s: %v", file, err)
			continue
		}

		entries = append(entries, FileEntry{
			Path:   relativePath,
			Status: metadata.Status,
		})
	}

	sort.SliceStable(entries, func(i, j int) bool {
		return entries[i].Status && !entries[j].Status
	})

	fmt.Printf("%-60s %-10s\n", "File Path", "Status")
	fmt.Println(strings.Repeat("-", 75))

	for _, entry := range entries {
		color := determineStatusColor(entry.Status)
		printStatus(entry.Path, entry.Status, color)
	}
}

func collectMarkdownFilesInDir(srcDir string) ([]string, error) {
	var files []string

	fmt.Println("workspace: ", srcDir)

	err := filepath.Walk(srcDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() {
			if strings.HasPrefix(path, filepath.Join(srcDir, "assets")) {
				return filepath.SkipDir
			}
			return nil
		}

		if filepath.Base(path) == "layout.html" {
			return nil
		}

		if strings.HasSuffix(path, ".md") {
			files = append(files, path)
		}

		return nil
	})

	return files, err
}

func readMetadata(filePath string) (Metadata, error) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		return Metadata{}, fmt.Errorf("failed to read file %s: %w", filePath, err)
	}

	var metadata Metadata
	_, err = frontmatter.Parse(bytes.NewReader(content), &metadata)
	if err != nil {
		return Metadata{}, fmt.Errorf("failed to parse front matter for %s: %w", filePath, err)
	}

	return metadata, nil
}

func determineStatusColor(isLive bool) string {
	if isLive {
		return "\033[32m"
	}
	return "\033[31m"
}

func printStatus(filePath string, isLive bool, color string) {
	fmt.Printf("%-60s %s%v\033[0m\n", filePath, color, isLive)
}
