<h1 align="center">Arrow</h1>

Arrow is a small personal site generator built to turn `.md` files into `.html`
files with minimal overhead. My site content is written in Markdown with
frontmatter for metadata. I initially considered handwriting all my HTML instead
of using other SSG tools as i found them to be annoying in one or more ways. But
writing in HTML disrupted my writing flow too much. It is more of a problem for
my notes than the site's content as i write my private notes in similar
structure to my site, i just do not deploy it online. As such, markdown was the
better option. I’d prefer to use Djot, but for now, Arrow works with markdown,
and I don’t feel like rewriting it, even though it’s only about 200-300 lines of
Go.

Currently, Arrow takes in Markdown files with following frontmatter fields
`title`, `description`, `date`, `CSS`, `JS`, and `status`. It is injected into
a single base template for all pages. Some pages might need custom CSS or JS, so
the frontmatter includes css and js fields to handle that. The status field
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
