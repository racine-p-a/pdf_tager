use clap::Parser;
use lopdf::{Document, Object, StringFormat};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(subcommand_precedence_over_arg = true)]
struct Cli {
    /// The path to the PDF file (optional for add-calibre).
    path_to_pdf: PathBuf,

    #[command(subcommand)]
    action_to_do: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    List, // pdf_tager example.pdf list

    Add {
        // pdf_tager example.pdf add my_tag my_value
        tag_name: String,
        tag_value: String,
    },

    Purge, // pdf_tager example.pdf purge

    Rm {
        // pdf_tager example.pdf rm Author
        tag_name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.action_to_do {
        Action::List => {
            print_tags(&args.path_to_pdf)?;
        }

        Action::Add {
            tag_name,
            tag_value,
        } => {
            add_tag(&args.path_to_pdf, &tag_name, &tag_value)?;
        }

        Action::Purge => {
            purge_pdf(&args.path_to_pdf)?;
        }

        Action::Rm { tag_name } => {
            rm_tag(&args.path_to_pdf, &tag_name)?;
        }
    }

    Ok(())
}

/// Delete a tag given in argument
fn rm_tag(path_to_pdf: &Path, tag_name: &str) -> anyhow::Result<()> {
    let mut doc = Document::load(path_to_pdf)?;
    if doc.is_encrypted() {
        anyhow::bail!("PDF file is encrypted.");
    }

    if !doc.trailer.has(b"Info") {
        println!("No metadata Info found.");
        return Ok(());
    }

    let info = match doc.trailer.get(b"Info")?.clone() {
        Object::Reference(id) => doc.get_object_mut(id)?.as_dict_mut()?,
        Object::Dictionary(_) => {
            anyhow::bail!("Can't get the Dictionnary Info of this pdf.");
        }
        _ => anyhow::bail!("The Dictionnary info of the pdf is invalid"),
    };

    // OK, Removing now.
    if info.remove(tag_name.as_bytes()).is_some() {
        doc.save(path_to_pdf)?;
        println!("Deleted Metadata : {tag_name}");
    } else {
        println!("Metadata not found : {tag_name}");
    }

    Ok(())
}

/// Remove metadata frome the pdf dictionnary.
fn purge_pdf(path_to_pdf: &Path) -> anyhow::Result<()> {
    let mut doc = Document::load(path_to_pdf)?;
    if doc.is_encrypted() {
        anyhow::bail!("PDF file is encrypted.");
    }

    // Complete deletion.
    if doc.trailer.remove(b"Info").is_some() {
        doc.save(path_to_pdf)?;
        println!("Deleted Metadata : {}", path_to_pdf.display());
    } else {
        println!("No metadata to remove.");
    }

    Ok(())
}

/// Simply adds a tag and its value to the dictionary info of the PDF file.
fn add_tag(path_to_pdf: &Path, tag_name: &str, tag_value: &str) -> anyhow::Result<()> {
    let mut doc = Document::load(path_to_pdf)?;
    if doc.is_encrypted() {
        anyhow::bail!("PDF file is encrypted.");
    }

    let info_id = match doc.trailer.get(b"Info").cloned() {
        Ok(Object::Reference(id)) => id,
        Ok(Object::Dictionary(_)) => {
            anyhow::bail!("Failed to load dictionary info.");
        }
        _ => {
            let id = doc.add_object(lopdf::Dictionary::new());
            doc.trailer.set("Info", id);
            id
        }
    };

    let info = doc.get_object_mut(info_id)?.as_dict_mut()?;

    info.set(
        tag_name.as_bytes().to_vec(),
        Object::String(tag_value.as_bytes().to_vec(), StringFormat::Literal),
    );

    doc.save(path_to_pdf)?;
    println!("Added : {tag_name} = {tag_value}");

    Ok(())
}

/// Simply print on output (console) the dictionary info in the form key : value.
fn print_tags(path_to_pdf: &Path) -> anyhow::Result<()> {
    let doc = Document::load(path_to_pdf)?;
    if let Ok(info) = doc.trailer.get(b"Info") {
        let dictionary = match info {
            Object::Reference(id) => doc.get_object(*id)?.as_dict()?,
            Object::Dictionary(dict) => dict,
            _ => {
                println!("Invalid dictionary Info.");
                return Ok(());
            }
        };

        for (key, value) in dictionary.iter() {
            println!("{}: {:?}", String::from_utf8_lossy(key), value);
        }
    } else {
        println!("No metadata Info found.");
    }
    Ok(())
}
