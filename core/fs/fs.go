package fs

import (
	"fmt"
	"os"
	"io"
	"strings"
	"path/filepath"
)

func PrepareDirectories(distDir string) error {
	if err := os.RemoveAll(distDir); err != nil {
		return fmt.Errorf("failed to clear destination directory %s: %w", distDir, err)
	}
	if err := os.MkdirAll(distDir, os.ModePerm); err != nil {
		return fmt.Errorf("failed to create destination directory %s: %w", distDir, err)
	}
	return nil
}

func CollectMarkdownFiles(srcDir string) ([]string, error) {
	var files []string
	err := filepath.Walk(srcDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if strings.HasSuffix(path, ".md") {
			files = append(files, path)
		}
		return nil
	})
	return files, err
}

func CopyAssets(src, dest string) error {
	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		destPath := filepath.Join(dest, strings.TrimPrefix(path, src))
		if info.IsDir() {
			return os.MkdirAll(destPath, os.ModePerm)
		}
		return copyFile(path, destPath)
	})
}

func copyFile(srcPath, destPath string) error {
	srcFile, err := os.Open(srcPath)
	if err != nil {
		return fmt.Errorf("failed to open source file %s: %w", srcPath, err)
	}
	defer srcFile.Close()
	destFile, err := os.Create(destPath)
	if err != nil {
		return fmt.Errorf("failed to create destination file %s: %w", destPath, err)
	}
	defer destFile.Close()
	_, err = io.Copy(destFile, srcFile)
	return err
}
