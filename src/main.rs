//! Programme principal : lit le JSON passé en argument et écrit le MindMap
//! dans `graph/`.

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use phone_trie::error::AppError;
use phone_trie::loader::{build_trie, load_contacts};
use phone_trie::plantuml::{mindmap_path, to_mindmap, write_mindmap};

/// Dossier de sortie, relatif au répertoire courant.
const GRAPH_DIR: &str = "graph";

fn main() -> ExitCode {
    // args_os car un chemin n'est pas forcément de l'UTF-8 valide.
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

/// Charge les contacts, construit le trie et écrit le MindMap. Renvoie le
/// chemin du fichier écrit.
fn run(input: &Path) -> Result<PathBuf, AppError> {
    let contacts = load_contacts(input)?;
    let (trie, rejected) = build_trie(contacts);
    for contact in &rejected {
        eprintln!(
            "avertissement : contact « {} » ignoré, {}",
            contact.name, contact.error
        );
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
