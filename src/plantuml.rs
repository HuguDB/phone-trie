//! Export du trie au format PlantUML MindMap.
//!
//! `to_mindmap` génère le texte du MindMap, `write_mindmap` l'écrit sur disque.

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::trie::{Node, Trie};

/// Génère le MindMap PlantUML d'un trie.
///
/// La racine s'affiche `* Racine`, puis chaque nœud occupe une ligne avec
/// autant de `*` que sa profondeur + 1, suivis de son chiffre. Un nœud où un
/// numéro se termine affiche aussi sa valeur : `**** 7 : Alice`. Les enfants
/// sont triés par chiffre, la sortie ne dépend donc pas de l'ordre d'insertion.
pub fn to_mindmap<T: Display>(trie: &Trie<T>) -> String {
    let mut out = String::from("@startmindmap\n* Racine\n");
    for (digit, child) in sorted_children(trie.root()) {
        write_node(digit, child, 1, &mut out);
    }
    out.push_str("@endmindmap\n");
    out
}

/// Écrit la ligne du nœud `node` (profondeur `depth`, racine = 0) puis,
/// récursivement, celles de ses descendants.
fn write_node<T: Display>(digit: char, node: &Node<T>, depth: usize, out: &mut String) {
    out.push_str(&"*".repeat(depth + 1));
    out.push(' ');
    out.push(digit);
    if let Some(value) = node.value() {
        out.push_str(" : ");
        out.push_str(&value.to_string());
    }
    out.push('\n');

    for (child_digit, child) in sorted_children(node) {
        write_node(child_digit, child, depth + 1, out);
    }
}

/// Enfants d'un nœud, triés par chiffre.
fn sorted_children<T>(node: &Node<T>) -> Vec<(char, &Node<T>)> {
    let mut children: Vec<(char, &Node<T>)> = node
        .children()
        .map(|(digit, child)| (*digit, child))
        .collect();
    children.sort_unstable_by_key(|(digit, _)| *digit);
    children
}

/// Écrit `content` dans le fichier situé à `path`, en l'écrasant s'il existe.
///
/// # Erreurs
///
/// `AppError::Io` si le fichier ne peut pas être créé ou écrit, par exemple
/// si le dossier parent n'existe pas.
pub fn write_mindmap(content: &str, path: &Path) -> Result<(), AppError> {
    fs::write(path, content)?;
    Ok(())
}

/// Chemin du `.puml` à générer pour le fichier d'entrée `input` :
/// `<graph_dir>/<nom de input sans extension>.puml`.
///
/// Si `input` n'a pas de nom de fichier (par exemple `..`), le fichier
/// s'appelle `mindmap.puml`.
pub fn mindmap_path(input: &Path, graph_dir: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap_or(OsStr::new("mindmap"));
    graph_dir.join(stem).with_extension("puml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Chemin d'un fichier temporaire propre à chaque test.
    fn temp_path(file: &str) -> PathBuf {
        std::env::temp_dir().join(format!("phone_trie_{file}"))
    }

    #[test]
    fn mindmap_path_uses_input_name_without_extension() {
        let path = mindmap_path(Path::new("data/02_common_parts.json"), Path::new("graph"));
        assert_eq!(path, Path::new("graph").join("02_common_parts.puml"));
    }

    #[test]
    fn mindmap_path_without_file_name_falls_back_to_default() {
        let path = mindmap_path(Path::new(".."), Path::new("graph"));
        assert_eq!(path, Path::new("graph").join("mindmap.puml"));
    }

    /// Construit un trie de contacts à partir de couples (numéro, nom).
    fn trie_of(contacts: &[(&str, &str)]) -> Trie<String> {
        let mut trie = Trie::new();
        for (number, name) in contacts {
            trie.insert(number, name.to_string()).unwrap();
        }
        trie
    }

    #[test]
    fn mindmap_of_two_numbers_matches_expected_output() {
        let trie = trie_of(&[("0612", "Alice"), ("0634", "Bob")]);
        let expected = "\
@startmindmap
* Racine
** 0
*** 6
**** 1
***** 2 : Alice
**** 3
***** 4 : Bob
@endmindmap
";
        assert_eq!(to_mindmap(&trie), expected);
    }

    #[test]
    fn mindmap_of_empty_trie_has_root_only() {
        let trie: Trie<String> = Trie::new();
        assert_eq!(to_mindmap(&trie), "@startmindmap\n* Racine\n@endmindmap\n");
    }

    #[test]
    fn terminal_node_with_children_shows_value_and_children() {
        // "15" est un préfixe de "1515" : le nœud 5 porte une valeur ET a un enfant.
        let trie = trie_of(&[("1515", "Bob"), ("15", "SAMU")]);
        let expected = "\
@startmindmap
* Racine
** 1
*** 5 : SAMU
**** 1
***** 5 : Bob
@endmindmap
";
        assert_eq!(to_mindmap(&trie), expected);
    }

    #[test]
    fn output_does_not_depend_on_insertion_order() {
        let contacts = [("0634", "Bob"), ("0478", "Chloé"), ("0612", "Alice")];
        let mut reversed = contacts;
        reversed.reverse();

        let forward = to_mindmap(&trie_of(&contacts));
        let backward = to_mindmap(&trie_of(&reversed));

        assert_eq!(forward, backward);
        let expected = "\
@startmindmap
* Racine
** 0
*** 4
**** 7
***** 8 : Chloé
*** 6
**** 1
***** 2 : Alice
**** 3
***** 4 : Bob
@endmindmap
";
        assert_eq!(forward, expected);
    }

    #[test]
    fn mindmap_written_to_file_can_be_read_back() {
        let trie = trie_of(&[("0612", "Alice"), ("0634", "Bob")]);
        let content = to_mindmap(&trie);
        let path = temp_path("roundtrip.puml");

        write_mindmap(&content, &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_creates_file_with_content() {
        let path = temp_path("ok.puml");
        let content = "@startmindmap\n* Racine\n@endmindmap\n";

        write_mindmap(content, &path).unwrap();

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, content);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_overwrites_existing_file() {
        let path = temp_path("overwrite.puml");
        fs::write(&path, "ancien contenu beaucoup plus long").unwrap();

        write_mindmap("nouveau", &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "nouveau");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_in_missing_directory_returns_io_error() {
        let path = temp_path("dossier_inexistant").join("x.puml");
        let result = write_mindmap("contenu", &path);
        assert!(matches!(result, Err(AppError::Io(_))));
    }
}
