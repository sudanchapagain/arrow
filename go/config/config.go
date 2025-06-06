package config

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"gopkg.in/yaml.v2"
)

type Workspace struct {
	Path string `yaml:"path"`
}

type ServerConfig struct {
	Port int `yaml:"port"`
}

type Config struct {
	Workspaces map[string]Workspace `yaml:"workspaces"`
	Server     ServerConfig         `yaml:"server"`
}

func LoadConfig() (*Config, error) {
	configPath := getConfigPath()

	cfg := &Config{
		Workspaces: make(map[string]Workspace),
		Server:     ServerConfig{Port: 8000},
	}

	if _, err := os.Stat(configPath); err != nil {
		if os.IsNotExist(err) {
			return nil, errors.New("Config file not found")
		}
		return nil, fmt.Errorf("failed to stat config file: %w", err)
	}

	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	if err := yaml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config file: %w", err)
	}

	if cfg.Server.Port == 0 {
		cfg.Server.Port = 8000
	}

	return cfg, nil
}

func getConfigPath() string {
	if runtime.GOOS == "windows" {
		return filepath.Join(os.Getenv("APPDATA"), "arrow", "arrow.conf")
	}

	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "/etc/arrow/arrow.conf"
	}
	return filepath.Join(homeDir, ".config", "arrow", "arrow.conf")
}

func (cfg *Config) GetPath(workspaceKey string) (string, error) {
	if workspaceKey == "" {
		for key := range cfg.Workspaces {
			workspaceKey = key
			break
		}
	}

	workspace, exists := cfg.Workspaces[workspaceKey]
	if !exists {
		return "", fmt.Errorf("workspace %s not found in config", workspaceKey)
	}

	finalPath := workspace.Path

	if _, err := os.Stat(finalPath); os.IsNotExist(err) {
		return "", fmt.Errorf("workspace %s does not exist: %s", workspaceKey, finalPath)
	}

	return finalPath, nil
}
