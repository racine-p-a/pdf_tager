use clap::{Parser, Subcommand};
use std::path::PathBuf;
use lopdf::{Document, Object, StringFormat};
use roxmltree::Document as XmlDocument;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    /// The path to the PDF file.
    path_to_pdf: PathBuf,

    #[command(subcommand)]
    action_to_do: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    List,

    Add {
        tag_name: String,
        tag_value: String,
    },

    AddCalibre {
        /// Optional path to the Calibre OPF file.
        path_to_metadata: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.action_to_do {
        Action::List => print_tags(&args.path_to_pdf)?,

        Action::Add {
            tag_name,
            tag_value,
        } => add_tag(&args.path_to_pdf, &tag_name, &tag_value)?,

        Action::AddCalibre { path_to_metadata } => {
            let metadata_path = path_to_metadata.unwrap_or_else(|| {
                args.path_to_pdf
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("metadata.opf")
            });

            add_calibre(&args.path_to_pdf, &metadata_path)?;
        }
    }

    Ok(())
}

fn add_calibre(path_to_pdf: &Path, path_to_metadata: &Path) -> anyhow::Result<()> {
    // Lire et analyser le fichier OPF.
    let xml_content = fs::read_to_string(path_to_metadata)?;
    let xml = XmlDocument::parse(&xml_content)?;

    // Rechercher l'élément metadata, quel que soit son préfixe XML.
    let metadata = xml
        .descendants()
        .find(|node| node.is_element() && node.tag_name().name() == "metadata")
        .ok_or_else(|| anyhow::anyhow!("Élément metadata introuvable dans l'OPF"))?;

    // Regrouper les valeurs par clé.
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for node in metadata.children().filter(|node| node.is_element()) {
        let name = node.tag_name().name();

        if let Some(value) = node.text().map(str::trim).filter(|v| !v.is_empty()) {
            let key = match name {
                "title" => "Title",
                "creator" => "Author",
                "description" => "Subject",
                "subject" => "Keywords",
                "publisher" => "Publisher",
                "date" => "PublicationDate",
                "language" => "Language",
                "rights" => "Rights",
                "identifier" => "Identifier",
                _ => continue,
            };

            fields
                .entry(key.to_string())
                .or_default()
                .push(value.to_string());
        }

        // Métadonnées Calibre de la forme :
        // <meta name="calibre:series" content="Ma série"/>
        if name == "meta" {
            if let (Some(meta_name), Some(content)) = (
                node.attribute("name"),
                node.attribute("content"),
            ) {
                if !content.trim().is_empty() {
                    let key = match meta_name {
                        "calibre:series" => "CalibreSeries",
                        "calibre:series_index" => "CalibreSeriesIndex",
                        "calibre:rating" => "CalibreRating",
                        _ => continue,
                    };

                    fields
                        .entry(key.to_string())
                        .or_default()
                        .push(content.trim().to_string());
                }
            }
        }
    }

    if fields.is_empty() {
        anyhow::bail!("Aucune métadonnée reconnue dans le fichier OPF");
    }

    // Charger le PDF.
    let mut pdf = Document::load(path_to_pdf)?;

    if pdf.is_encrypted() {
        anyhow::bail!("Le PDF est chiffré et ne peut pas être modifié.");
    }

    // Récupérer ou créer le dictionnaire Info.
    let info_id = match pdf.trailer.get(b"Info").cloned() {
        Ok(Object::Reference(id)) => id,
        Ok(Object::Dictionary(dictionary)) => {
            let id = pdf.add_object(dictionary);
            pdf.trailer.set("Info", id);
            id
        }
        Ok(_) => anyhow::bail!("Dictionnaire Info PDF invalide"),
        Err(_) => {
            let id = pdf.add_object(lopdf::Dictionary::new());
            pdf.trailer.set("Info", id);
            id
        }
    };

    // Écrire les valeurs dans le dictionnaire PDF.
    let info = pdf.get_object_mut(info_id)?.as_dict_mut()?;

    for (key, values) in &fields {
        let value = values.join(", ");

        info.set(
            key.as_bytes().to_vec(),
            Object::String(value.into_bytes(), StringFormat::Literal),
        );
    }

    // Sauvegarder le PDF.
    pdf.save(path_to_pdf)?;

    println!(
        "{} champ(s) importé(s) depuis {}",
        fields.len(),
        path_to_metadata.display()
    );

    Ok(())
}


fn add_tag(
    path_to_pdf: &PathBuf,
    tag_name: &str,
    tag_value: &str,
) -> anyhow::Result<()> {
    let mut doc = Document::load(path_to_pdf)?;

    let info_id = match doc.trailer.get(b"Info").cloned() {
        Ok(Object::Reference(id)) => id,
        Ok(Object::Dictionary(_)) => {
            anyhow::bail!("Le dictionnaire Info direct n'est pas pris en charge ici.");
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

    println!("Métadonnée ajoutée : {tag_name} = {tag_value}");

    Ok(())
}

fn print_tags(path_to_pdf: &PathBuf) -> anyhow::Result<()> {
    let doc = Document::load(path_to_pdf)?;
    if let Ok(info) = doc.trailer.get(b"Info") {
        let dictionary = match info {
            Object::Reference(id) => doc.get_object(*id)?.as_dict()?,
            Object::Dictionary(dict) => dict,
            _ => {
                println!("Dictionnaire Info invalide.");
                return Ok(());
            }
        };

        for (key, value) in dictionary.iter() {
            println!(
                "{}: {:?}",
                String::from_utf8_lossy(key),
                value
            );
        }
    } else {
        println!("Aucune métadonnée Info trouvée.");
    }
    Ok(())
}