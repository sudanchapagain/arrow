<div align="center"><h1>Arrow</h1></div>

## what?

a small personal site generator. this is that glue code that turns my markdown
to HTML and does few more things that i want in my site.

## why?

[read this](https://sudanchapagain.com.np/writings/writing-html-is-hard).

everything came with their own quirks so, i decided build my own solution.
there’s no complex setup, no features, it's almost just writing html by myself.

## how?

markdown files following frontmatter (title, desc, date, css, js, status).
state decides if that file should be public or not. css & js are specific to
that page. all asset's are assumed to be inside `/src/assets/`. CSS and JS
file's path are assumed to be `/src/assets/css` and `/src/assets/js`
respectively.

everything should be inside `/src/` and output is generated inside of `/dist/`.

directory structure:

```
├───dist/
│   ├───css/
│   ├───img/
│   ├───js/
│   ├───index.html
│   └───writings/
└───src/
    ├───assets
    │   ├───css/
    │   ├───img/
    │   └───js/
    ├───index.md
    └───writings/
```
