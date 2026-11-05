//! Point d'entrée : lit le chemin du JSON en argument, construit le trie et
//! écrit le MindMap PlantUML dans `graph/`. Toute la logique est dans la lib.

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use phone_trie::error::AppError;
use phone_trie::loader::{build_trie, load_contacts};
use phone_trie::plantuml::{mindmap_path, to_mindmap, write_mindmap};

/// Dossier où sont écrits les `.puml`.
const GRAPH_DIR: &str = "graph";

fn main() -> ExitCode {
    // `args_os` plutôt que `args` : `args` panique si un argument n'est pas
    // de l'UTF-8 valide, alors qu'un chemin de fichier peut ne pas l'être.
    let Some(input) = env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage : phone-trie <fichier.json>");
        eprintln!("exemple : cargo run --release -- data/02_common_parts.json");
        return ExitCode::from(2);
    };

    match run(&input) {
        Ok(output) => {
            println!("MindMap généré : {}", output.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("erreur : {err}");
            ExitCode::FAILURE
        }
    }
}

/// Charge les contacts, construit le trie et écrit le MindMap.
/// Renvoie le chemin du fichier généré.
fn run(input: &Path) -> Result<PathBuf, AppError> {
    let contacts = load_contacts(input)?;
    let (trie, rejected) = build_trie(contacts);
    for err in &rejected {
        eprintln!("avertissement : numéro ignoré, {err}");
    }

    let output = mindmap_path(input, Path::new(GRAPH_DIR));
    write_mindmap(&to_mindmap(&trie), &output)?;
    println!(
        "{} numéros stockés dans {} nœuds ({} chiffres sans trie)",
        trie.len(),
        trie.node_count(),
        trie.total_digits()
    );
    Ok(output)
}
