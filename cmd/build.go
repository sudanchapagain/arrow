package cmd

import (
	"log"
	"path/filepath"
	"sync"

	"arrow/config"
	"arrow/core/fs"
	"arrow/core/md"
)

func Build(entry string) {
	cfg, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("Error loading config: %v", err)
	}

	buildPath, err := cfg.GetPath(entry)
	if err != nil {
		log.Fatalf("Error getting path for entry '%s': %v", entry, err)
	}

	srcDir := filepath.Join(buildPath, "src")
	distDir := filepath.Join(buildPath, "dist")
	assetsDir := filepath.Join(srcDir, "assets")

	log.Println("Starting build...")
	log.Printf("Source: %s", srcDir)
	log.Printf("Destination: %s", distDir)

	if err := fs.PrepareDirectories(distDir); err != nil {
		log.Fatalf("Error preparing directories: %v", err)
	}

	if err := fs.CopyAssets(assetsDir, filepath.Join(distDir, "assets")); err != nil {
		log.Printf("Warning: Error copying assets: %v", err)
	}

	files, err := fs.CollectMarkdownFiles(srcDir)
	if err != nil {
		log.Fatalf("Error collecting markdown files: %v", err)
	}

	processFilesConcurrently(files, srcDir, distDir)
	log.Println("Build completed!")
}

func processFilesConcurrently(files []string, srcDir, distDir string) {
	var wg sync.WaitGroup
	for _, file := range files {
		wg.Add(1)
		go func(file string) {
			defer wg.Done()
			if err := md.ProcessMarkdownFile(file, srcDir, distDir); err != nil {
				log.Printf("Error processing file %s: %v", file, err)
			}
		}(file)
	}
	wg.Wait()
}
