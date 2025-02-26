package main

import (
	"log"
	"os"

	"arrow/cmd"

	"github.com/urfave/cli/v2"
)

func main() {
	const appVersion = "4.0"

	app := &cli.App{
		Name:    "arrow",
		Usage:   "a simple static site generator",
		Version: appVersion,
		Commands: []*cli.Command{
			{
				/**
					Build default site with localhost of the built html structure
					such that upon file changes in the source structure rebuilds the
					site & updates automatically.

					Additional flags can be passed to use alternative layout.html
					and style sheet.

					usage:
						arrow serve -- serves default site
						arrow serve note -- serves notes folder.

					the entries are fetched from `.config/arrow/arrow.conf` in linux
					& macos but `AppData/roaming/arrow/arrow.conf` in windows.
				**/
				Name:    "serve",
				Aliases: []string{"s"},
				Usage:   "Start a local server and watch for changes",
				Flags: []cli.Flag{
					&cli.IntFlag{
						Name:  "port",
						Value: 0,
						Usage: "Specify the port to serve on",
					},
					&cli.StringFlag{
						Name:    "entry",
						Usage:   "Specify the workspace key (e.g., site, notes)",
						Aliases: []string{"e"},
					},
				},
				Action: func(c *cli.Context) error {
					port := c.Int("port")
					entry := c.String("entry")
					cmd.Serve(port, entry)
					return nil
				},
			},
			{
				/**
					This starts a TUI flow where user is prompted to enter file
					name, description, file path too (as notes might not be just
					src/writings or src/blogs rather src/computer-science/1.md or
					computer/operating-system.md)

					The input captured from the flow is used to create a new file
					in the workspace (notes or site). With that, user is finally
					prompted if they want to open the file in editor (declared
					through config)

					usage:
						arrow new -- default site
						arrow new note -- a entry in declared notes
				**/
				Name:    "new",
				Aliases: []string{"n"},
				Usage:   "Create a new entry",
				Flags: []cli.Flag{
					&cli.StringFlag{
						Name:    "entry",
						Usage:   "Specify the workspace key",
						Aliases: []string{"e"},
					},
				},
				Action: func(c *cli.Context) error {
					entry := c.String("entry")
					cmd.New(entry)
					return nil
				},
			},
			{
				/**
					outputs status of each source file

					usage:
						arrow status -- default site
						arrow status note -- a entry in declared notes
				**/
				Name:    "status",
				Aliases: []string{"st"},
				Usage:   "List status of all source markdown files",
				Flags: []cli.Flag{
					&cli.StringFlag{
						Name:    "entry",
						Usage:   "Specify the workspace key",
						Aliases: []string{"e"},
					},
				},
				Action: func(c *cli.Context) error {
					entry := c.String("entry")
					cmd.Status(entry)
					return nil
				},
			},
			{
				/**
					Just builds the site.

					usage:
						arrow build -- default site build
						arrow build note -- build notes
				**/
				Name:    "build",
				Aliases: []string{"b"},
				Usage:   "Build the static source files",
				Flags: []cli.Flag{
					&cli.StringFlag{
						Name:    "entry",
						Usage:   "Specify the workspace key",
						Aliases: []string{"e"},
					},
				},
				Action: func(c *cli.Context) error {
					entry := c.String("entry")
					cmd.Build(entry)
					return nil
				},
			},
		},
	}

	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}
