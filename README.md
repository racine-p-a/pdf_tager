# pdf_tager

`pdf_tager` is a small command-line tool for managing metadata fields in the `/Info` dictionary of PDF files.

It can list, add, remove, and clear metadata fields in unencrypted PDF files.

**Note:** `pdf_tager` modifies the PDF `/Info` dictionary. It does not manage XMP metadata.

## Prerequisites

Install the `Rust toolchain` on your computer ([instructions here](https://rust-lang.org/learn/get-started/)).

## Installation

```shell
# Download the code in any directory or clone it with git.
git clone https://github.com/racine-p-a/pdf_tager
cd pdf_tager
cargo build --release
```

That's it ! You can optionally install the executable system-wide.

```shell
# Update the locations if necessary.
sudo install -Dm755 target/release/pdf_tager /usr/local/bin/pdf_tager
```

## Usage

The following examples assume that you added the command to your system. If not, just change `pdf_tager` with
`path_to_your_executable_pdf_tager`.  

### List metadata

```shell
pdf_tager path/to/your/file.pdf list
```

### Add metadata

```shell
# Add the tag
pdf_tager path/to/your/file.pdf add Title "My Wonderful Title"
# Check if the tag is properly added
pdf_tager path/to/your/file.pdf list
```

### Remove metadata

```shell
# Will remove the tag "Title"
pdf_tager path/to/your/file.pdf rm Title

# The following command removes all entries from the PDF's Info dictionary.
pdf_tager path/to/your/file.pdf purge
```
**Warning:** This does not necessarily remove XMP metadata embedded in the PDF.

## Limitations

- Encrypted PDF files cannot be modified.
- Only the PDF `/Info` dictionary is managed; XMP metadata is not supported.
- Some PDF readers may interpret metadata values differently, especially when they contain non-ASCII characters.