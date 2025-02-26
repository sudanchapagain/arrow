package cmd

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"sync"
	"syscall"
	"time"

	"arrow/config"

	"github.com/fsnotify/fsnotify"
)

const debounceDelay = 500 * time.Millisecond

var (
	serveMutex sync.Mutex
	quitChan   = make(chan os.Signal, 1)
	stopChan   = make(chan struct{})
)

func Serve(port int, entry string) {
	signal.Notify(quitChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-quitChan
		fmt.Println("Received shutdown signal, shutting down...")

		shutdown()
	}()

	cfg, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	buildPath, err := cfg.GetPath(entry)
	if err != nil {
		log.Fatalf("Error getting path for entry '%s': %v", entry, err)
	}

	distPath := filepath.Join(buildPath, "dist")

	if _, err := os.Stat(distPath); os.IsNotExist(err) {
		fmt.Println("dist directory not found. Running Build...")
		Build(entry)
	}

	if port == 0 {
		port = cfg.Port
	}

	go func() {
		serveMutex.Lock()
		defer serveMutex.Unlock()

		fmt.Printf("Serving at http://localhost:%d/\n", port)

		http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
			fmt.Printf("Request: %s %s\n", r.Method, r.URL.Path)

			requestedPath := r.URL.Path

			if requestedPath == "/" {
				requestedPath = "/index.html"
			}

			filePath := distPath + requestedPath

			if info, err := os.Stat(filePath); err == nil {
				if info.IsDir() {
					if !strings.HasSuffix(filePath, "/") {
						filePath += "/"
					}
					indexPath := filePath + "index.html"
					if _, err := os.Stat(indexPath); err == nil {
						filePath = indexPath
					} else {
						http.NotFound(w, r)
						return
					}
				}
				http.ServeFile(w, r, filePath)
				return
			}

			if !hasExtension(requestedPath) {
				altPath := distPath + requestedPath + ".html"
				if _, err := os.Stat(altPath); err == nil {
					http.ServeFile(w, r, altPath)
					return
				}
			}

			http.NotFound(w, r)
		})

		err := http.ListenAndServe(fmt.Sprintf(":%d", port), nil)
		if err != nil {
			log.Fatalf("Failed to start server: %v", err)
		}
	}()

	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		log.Fatalf("Failed to create file watcher: %v", err)
	}
	defer watcher.Close()

	err = watchDirectories(watcher, buildPath)
	if err != nil {
		log.Fatalf("Failed to watch source directories: %v", err)
	}

	var timer *time.Timer

	for {
		select {
		case event, ok := <-watcher.Events:
			if !ok {
				return
			}

			if event.Op&(fsnotify.Write|fsnotify.Create|fsnotify.Remove) != 0 {
				if timer != nil {
					timer.Stop()
				}

				timer = time.AfterFunc(debounceDelay, func() {
					fmt.Println("Change detected, rebuilding...")
					Build(entry)
				})
			}

		case err, ok := <-watcher.Errors:
			if ok {
				log.Printf("Watcher error: %v", err)
			}

		case <-stopChan:
			return
		}
	}
}

func hasExtension(path string) bool {
	lastSlash := strings.LastIndex(path, "/")
	lastDot := strings.LastIndex(path, ".")
	return lastDot > lastSlash
}

func watchDirectories(watcher *fsnotify.Watcher, basePath string) error {
	return filepath.Walk(basePath, func(path string, info os.FileInfo, err error) error {
		if err == nil && info.IsDir() {
			return watcher.Add(path)
		}
		return err
	})
}

func shutdown() {
	close(stopChan)
	serveMutex.Unlock()
}
