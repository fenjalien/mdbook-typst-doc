# mdbook Typst Doc Preprocessor
A preprocess for mdbook's html renderer to aid in writing Typst documentation. This is currently still a work in progress.

## Features
- [x] Display Typst types as colored pills, like in the Typst documentation.
- [x] Typst code block:
  - [x] Highlighting
  - [x] Rendering
  - [x] Examples (highlighting code and rendering)
- [x] Parameter descriptions
- [ ] Function definitions

## Setup
Install the preprocessor through `cargo`'s `--git` flag:
```
cargo install --git https://github.com/fenjalien/mdbook-typst-doc.git
```

It is recommended to copy and include `example/typst-doc.css` in your book as it contains the recommended styling, feel free to modify it how you see fit. See `example/book.toml` for more recommeded setup.

## Usage
### Typst Types
Converts `{{#type ...}}` into a link to the type's web page with the same styling as the official Typst documentation. You can add new types by adding them to the `preprocessor.typst-doc.types` key in the `book.toml`:
```toml
[preprocessor.typst-doc.types]
int = {class = "num", link = "https://typst.app/docs/reference/foundations/int/"}
```
- `link`: The link to the type's definition. You can use a realtive url if you define your own types within the book (`"/type_definition.html"`). If no link is given the output HTML element will not be a link.
- `class`: The text to append to `"type-"` to make up the CSS class of the HTML element. This is to aid in the styling of the element. This is required unless the key `default-type-class` is given, in which case the default class will be used.

Normally the preprocessor will panic if a type is used that is not in the config table. That is, unless the `default-type-class` key has been given, in which case an element with the given class and no link will be placed.

For some reason having a link in the default template breaks when used within a heading. You can place an exclamation mark `!` between the hash and the type (`{{#!type ...}}`) to not use a link. If you know/find a way to fix this without having to turn off the link please let me know :)

### Typst Code Blocks
Code blocks that have a language `typ` or `typc` can be processed. A `typ` block is in markup mode while a `typc` block is in code mode.

By default syntax highlighting will be applied to the code and nothing else. To render the code instead you can include the `render` option on the code block like this:
```typ,render
Hello, world!
```
To show the code block and the rendered output include the `example` option instead of `render`.

Rendering Typst code blocks requires the Typst CLI 0.11.0 to be installed on the system. By default the `typst` command will be used but this can be changed by setting the `typst-command` option. The root folder will be the book's folder (`example/`) but the `root-arg` option will be given as the `--root` argument if present.

Rendered images are named after the first 5 characters of the md5 sum of their source code. They will first be rendered to `mdbook-typst-doc/` then moved all at once into `src/mdbook-typst-doc/`. This will trigger a re-render if the `serve` or `watch` command is being used. However if the file already exists in `src/mdbook-typst-src/` the file it will not be rendered and moved again.

### Parameter Defintions
Stylised and formatted parameter descriptions! It should be written as an html block like so:
```html
<parameter-description name="name" types="int,float" default="default">
Some description
</parameter-description>
```

The `name` attribute and text inside the tags will be left alone. The `types` attribute will be split on commas and formatted to use the `{{#type ...}}` preprocessor. The `default` attribute will be highlighted.


## Custom Templates
You can override the default look of the above features by providing handlebar templates in `themes/typst-doc` with the following names. The keys and values of the data passed to the template is also described:
- `type.hbs`: Template for the Typst types.
  - `link` The url to the type's definition
  - `class` The css class to apply to the type
  - `name` The name of the type.
  - `use_link` A boolean that is false when `!` is added after the hash. It should not use the link if false.
- `code.hbs`: Template for a Typst code block with no options.
  - `source` The highlighted code block.
- `render.hbs`: Template for a rendered Typst code block.
  - `image` The markdown link to the generated image.
- `example.hbs`: Template for a Typst code block with a rendered image.
  - `source` The highlighted code block.
  - `image` The markdown link to the generated image.
- `parameter.hbs`: Template for the parameter description.
  - `name` The string given by the `name` attribute.
  - `types` An array of strings formatted to use the type preprocessor feature given by splitting the `types` attribute on commas.
  - `default` The default value given by the `default` attribute.
  - `description` The text between the tags.

 The default templates are kept in `/src/themes/`.

You can also specify a code template to use before a Typst code block is rendered. They should be stored in a table with the key `code-templates`. You can specify a unique template for `typ` code and `typc` code. The `{{input}}` will be replaced by the source code to be rendered. See the example book toml for more details.