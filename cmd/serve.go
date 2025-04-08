package cmd

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"syscall"

	"arrow/config"
)

func Serve(port int, entry string) {
	signalChan := make(chan os.Signal, 1)
	signal.Notify(signalChan, syscall.SIGINT, syscall.SIGTERM)
	Build(entry)

	cfg, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	buildPath, err := cfg.GetPath(entry)
	if err != nil {
		log.Fatalf("Error getting path for entry '%s': %v", entry, err)
	}

	distPath := filepath.Join(buildPath, "dist")

	if port == 0 {
		port = cfg.Server.Port
	}

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Printf("Request: %s %s\n", r.Method, r.URL.Path)

		requestedPath := r.URL.Path
		if requestedPath == "/" {
			requestedPath = "/index.html"
		}

		filePath := filepath.Join(distPath, requestedPath)

		info, err := os.Stat(filePath)
		if err == nil && !info.IsDir() {
			http.ServeFile(w, r, filePath)
			return
		}

		if err == nil && info.IsDir() {
			indexPath := filepath.Join(filePath, "index.html")
			if _, err := os.Stat(indexPath); err == nil {
				http.ServeFile(w, r, indexPath)
				return
			}
		}

		if !hasExtension(requestedPath) {
			altPath := filepath.Join(distPath, requestedPath+".html")
			if _, err := os.Stat(altPath); err == nil {
				http.ServeFile(w, r, altPath)
				return
			}
		}

		http.NotFound(w, r)
	})

	server := &http.Server{Addr: fmt.Sprintf(":%d", port)}
	fmt.Printf("Serving at http://localhost:%d/\n", port)

	go func() {
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Failed to start server: %v", err)
		}
	}()

	<-signalChan
	fmt.Println("\nReceived shutdown signal, exiting.")
}

func hasExtension(path string) bool {
	lastSlash := strings.LastIndex(path, "/")
	lastDot := strings.LastIndex(path, ".")
	return lastDot > lastSlash
}
