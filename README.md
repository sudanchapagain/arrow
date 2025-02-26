<h1 align="center">Arrow</h1>

Arrow is a small personal site generator built to turn `.md` files into `.html`
files with minimal overhead. My site content is written in Markdown with
frontmatter for metadata. I initially considered handwriting all my HTML instead
of using other SSG tools as I found them to be annoying in one or more ways. But
writing in HTML disrupted my writing flow too much. It is more of a problem for
my notes than the site's content as I write my private notes in similar
structure to my site, I just do not deploy it online. As such, markdown was the
better option. I’d prefer to use Djot, but for now, Arrow works with markdown,
and I don’t feel like rewriting it, even though it’s only about 200-300 lines of
Go.

Currently, Arrow takes in Markdown files with following frontmatter fields
`title`, `description`, `date`, `CSS`, `JS`, and `status`. It is injected into
a single base template for all pages. Some pages might need custom CSS or JS, so
the frontmatter includes CSS and JS fields to handle that. The status field
determines whether a file gets built into HTML or not. I am using it to keep
drafts out of the live site.

Arrow follows a fixed repository structure. The `src/` folder is the only
directory used for content. `src/layout.html` is the base template for all
pages. Assets are stored in `src/assets/`, and Markdown files referencing
`image.png` should use `/assets/image.png`, not `/image.png`.

The output is placed in `dist/`, which is fully replaced on each build, so no
important files should be kept there. Markdown files from `src/` are converted
to HTML and placed in `dist/` with the same structure, while `src/assets/` is
copied to `dist/assets/`. The `dist/` folder is meant to be the root for
deployment or live hosting, with all resource paths starting there.

Arrow does exactly what I need. It is not a general-purpose static site
generator, nor does it aim to be. If your needs align with mine, it might be
useful; otherwise, there are plenty of other tools out there.

## Features

- Build your site with `arrow build .`
- Serve your pages locally with `arrow serve --entry <workspace name>`
- Create a new entry with `arrow serve -e <workspace name>`
- Check status of your markdown files with `arrow status -e <workspace name`. It
  is possible with status attribute in frontmatter.

## Building

Running just `go build .` will build it for all environments. However to use it
in your deployment server, you might need to produce a linux binary in non
linux environments for that the standard method of setting environment
variables & running `go build` will be enough.

On Windows you can run the following batch snippet:

```bat
@echo off

set GOOS=linux
set GOARCH=amd64
set CGO_ENABLED=0

set GOFILE=.\main.go

set OUTPUT_BINARY=arrow

go build -o %OUTPUT_BINARY% %GOFILE%

set GOOS=
set GOARCH=
set CGO_ENABLED=

echo Build complete!
pause
```

On WSL or other environments you can do the following:

for sh compatible shells:

```sh
#!/bin/sh

CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o arrow ./main.go
```

on fish:

```fish
#!/bin/fish

set CGO_ENABLED 0
set GOOS linux
set GOARCH amd64

go build -o arrow ./main.go
```

## Usage

To start you should at minimum have the following directory structure

```
./
./src/
./src/assets/{keep your css, js, images here}
./src/layout.html # this is the main & only layout, you should keep it as
                  # minimal as possible. You can extend your specific pages
                  # with css & js files specific to that page
./src/{your markdown files with your choice of structure.}
```

every markdown file with frontmatter attribute of `status=true` will be turned
to HTML with layout.html.

example `layout.html`

```xml
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <!-- title from frontmatter will be here -->
    <title>{{ .Title }}</title>
    <!-- same as above but description -->
    <meta name="description" content="{{ .Desc }}" />
    <meta property="og:locale" content="en" />
    <!-- link to your css file -->
    <link rel="stylesheet" type="text/css" href="{{ .AssetsPath }}/css/global.css"/> 
    <!-- more example of linking assets -->
    <link rel="icon" type="image/x-icon" href="{{ .AssetsPath }}/img/favicon.ico" />
  </head>

  <body>

    <main>
        <!-- the body of markdown will be converted to html & inserted here -->
      {{ .Content }}
    </main>

    <style>
        <!-- if you have defined a custom css for your markdown page it will be inserted here -->
        {{ .InlineCSS }}
    </style>
    
    <script>
        <!-- same as css but for js -->
        {{ .InlineJS }}
    </script>
  </body>
</html>
```

The markdown source files supports the following attributes as such, you should
have the following frontmatter at the top of your markdown file.

```md
---
title:
desc:
date:
status:
---

<!-- keep your content here -->

```

The Arrow CLI can help you with creation of new markdown source files but it
requires you to define configuration in your system. Check configuration
section to learn more.

With above structure in place, you can just enter `arrow build .` from the root
of your page to build your site.

## Configuration

To create multiple sites & help arrow recognize them & also use some of the
functionalities built into arrow, you need to declare a configuration file. The
path for configuration files is as follows:

on Windows: `~/AppData/Roaming/arrow/arrow.conf`

on Linux: `~/.config/arrow/arrow.conf` or `/etc/arrow/arrow.conf`

The structure of configuration file is in YAML format and should be as
following.

```yaml
workspaces:
  # an example name for your workspace. the first entry is considered default
  site:
    # path to your workspace
    path: "/home/me/site"
  # another workspace called note
  note:
    path: "/home/me/notes"

# the default port on which localhost will serve your pages.
server:
  port: 4321
```

with the above config, running `arrow serve -e note` will build the markdown
files at `/home/me/notes` & serve it on `localhost:4321`.

> on windows you have to escape the path with `//`

## todo

- better table printing for status command.
- new command improvements.