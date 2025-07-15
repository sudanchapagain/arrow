Arrow
=====

Arrow is a small personal site generator built to turn `.djot` files into
`.html` files with minimal overhead. My site content is written in Djot with
frontmatter for metadata.

Arrow takes in Djot files with following frontmatter fields `title`,
`description`, `date`, `CSS`, `JS`, and `status`. It is injected into
a single base template for all pages. Some pages might need custom CSS or JS, so
the frontmatter includes CSS and JS fields to handle that. The status field
determines whether a file gets built into HTML or not. I am using it to keep
drafts out of the live site.

Arrow follows a fixed repository structure. The `src/` folder is the only
directory used for content. `templates/layout.html` is the base template for all
pages. Assets are stored in `src/assets/`, and djot files referencing
`image.png` should use `/assets/image.png`, not `/image.png`.

The output is placed in `dist/`, which is fully replaced on each build, so no
important files should be kept there. Djot files from `src/` are converted
to HTML and placed in `dist/` with the same structure, while `src/assets/` is
copied to `dist/assets/`. The `dist/` folder is meant to be the root for
deployment or live hosting, with all resource paths starting there.

Arrow does exactly what I need. It is not a general-purpose static site
generator, nor does it aim to be. If your needs align with mine, it might be
useful; otherwise, there are plenty of other tools out there.

usage
-----

To start you should at minimum have the following directory structure

```
./src/assets/{keep your css, js, images here}
./templates/layout.html
                  # this is the main & only layout, you should keep it
                  # as minimal as possible. You can extend your specific
                  # pages with css & js files specific to that page

./src/{your djot files with your choice of structure.}
```

example `layout.html`

```xml
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta property="og:locale" content="en" />
    <title>{{ page.title | default(value="default title goes here") }}</title>
    <meta name="description" content="{{ page.desc | default(value="deault description goes here") }}" />
    <link rel="stylesheet" type="text/css" href="{{ page.assets_path }}/css/global.css"/>
    <link rel="icon" type="image/x-icon" href="{{ page.assets_path }}/img/favicon.ico" />
  </head>
  <body>
    <main>
      {{ page.content | safe }}
    </main>

    {% if page.inline_css %}
      <style>
        {{ page.inline_css | safe }}
      </style>
    {% endif %}
    {% if page.inline_js %}
      <script>
        {{ page.inline_js | safe }}
      </script>
    {% endif %}
  </body>
</html>
```

configuration
-------------

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

with the above config, running `arrow serve -e note` will build the djot
files at `/home/me/notes` & serve it on `localhost:4321`.
