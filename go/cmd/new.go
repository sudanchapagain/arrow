package cmd

import (
	"arrow/config"
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/AlecAivazis/survey/v2"
)

func New(entry string) {
	cfg, err := config.LoadConfig()
	if err != nil {
		fmt.Println("Error loading config:", err)
		return
	}

	workspacePath, err := cfg.GetPath(entry)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}

	workspacePath = filepath.Join(workspacePath, "src")

	var fileName, description string
	prompt := []*survey.Question{
		{
			Name:   "fileName",
			Prompt: &survey.Input{Message: "Enter file name (without extension):"},
			Validate: func(val interface{}) error {
				if str, ok := val.(string); ok && str != "" {
					return nil
				}
				return fmt.Errorf("file name cannot be empty")
			},
		},
		{
			Name:   "description",
			Prompt: &survey.Input{Message: "Enter description (optional):"},
		},
	}
	answers := struct {
		FileName    string
		Description string
	}{}
	err = survey.Ask(prompt, &answers)
	if err != nil {
		fmt.Println("Prompt canceled.")
		return
	}
	fileName = answers.FileName
	description = answers.Description

	fileName = strings.TrimSpace(fileName)
	if fileName == "" {
		fmt.Println("File name cannot be empty.")
		return
	}

	filePath := filepath.Join(workspacePath, fileName+".md")

	if _, err := os.Stat(filePath); err == nil {
		fmt.Println("Error: A file with this name already exists.")
		return
	}

	file, err := os.Create(filePath)
	if err != nil {
		fmt.Println("Error creating file:", err)
		return
	}
	defer file.Close()

	date := time.Now().Format("2006-01-02")
	frontMatter := fmt.Sprintf(`---
title: %s
desc: %s
date: %s
status: false
---
`, fileName, description, date)

	writer := bufio.NewWriter(file)
	_, _ = writer.WriteString(frontMatter)
	writer.Flush()

	fmt.Println("New entry created at:", filePath)

	var openInEditor bool
	survey.AskOne(&survey.Confirm{Message: "Open file in editor?", Default: true}, &openInEditor)

	if openInEditor {
		editor := os.Getenv("EDITOR")
		if editor == "" {
			editor = "nano"
		}
		cmd := exec.Command(editor, filePath)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		err = cmd.Run()
		if err != nil {
			fmt.Println("Error opening editor:", err)
		}
	}
}
