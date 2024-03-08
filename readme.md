# mdbook Typst Doc Preprocessor
A preprocess for mdbook to aid in writing Typst documentation. This is currently still a work in progress and only public because it needs to be accessed from Github Actions for another repository.

## Features
- [x] Display Typst types as colored pills, like in the Typst documentation.
- [ ] Typst code block:
  - [ ] Highlighting
  - [ ] Rendering
  - [ ] Examples (highlighting code and rendering)
- [ ] Parameter descriptions
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
- `class`: The text to append to `"type-"` to make up the CSS class of the HTML element. This is to aid in the styling of the element. This is required unless the key `typst-doc.default-type-class` is given, in which case the default class will be used.

Normally the preprocessor will panic if a type is used that is not in the config table. That is, unless the `default-type-class` key has been given, in which case an element with the given class and no link will be placed.