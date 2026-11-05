//! Export du trie en MindMap PlantUML.

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::trie::{Node, Trie};

/// Génère le MindMap d'un trie : une ligne par nœud avec profondeur + 1
/// étoiles, et la valeur sur les nœuds terminaux (`**** 7 : Alice`).
/// Les enfants sont triés par chiffre.
pub fn to_mindmap<T: Display>(trie: &Trie<T>) -> String {
    let mut out = String::from("@startmindmap\n* Racine\n");
    for (digit, child) in sorted_children(trie.root()) {
        write_node(digit, child, 1, &mut out);
    }
    out.push_str("@endmindmap\n");
    out
}

/// Écrit la ligne d'un nœud puis celles de ses enfants.
fn write_node<T: Display>(digit: char, node: &Node<T>, depth: usize, out: &mut String) {
    out.push_str(&"*".repeat(depth + 1));
    out.push(' ');
    out.push(digit);
    if let Some(value) = node.value() {
        out.push_str(" : ");
        out.push_str(&escape_value(&value.to_string()));
    }
    out.push('\n');

    for (child_digit, child) in sorted_children(node) {
        write_node(child_digit, child, depth + 1, out);
    }
}

/// Échappe une valeur pour PlantUML : les retours à la ligne deviennent des
/// espaces et les caractères de mise en forme sont précédés de `~`. Le tilde
/// est écrit `<U+007E>` car PlantUML n'échappe pas `~~`.
fn escape_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                // `\r\n` ne donne qu'une seule espace.
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                escaped.push(' ');
            }
            c if c.is_control() => escaped.push(' '),
            '~' => escaped.push_str("<U+007E>"),
            '*' | '/' | '"' | '-' | '_' | '[' | ']' | '<' => {
                escaped.push('~');
                escaped.push(c);
            }
            c => escaped.push(c),
        }
    }
    escaped
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

/// Écrit le MindMap dans `path` en créant les dossiers manquants.
pub fn write_mindmap(content: &str, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Chemin du `.puml` pour `input` : `<graph_dir>/<nom sans extension>.puml`,
/// ou `mindmap.puml` si `input` n'a pas de nom de fichier.
pub fn mindmap_path(input: &Path, graph_dir: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap_or(OsStr::new("mindmap"));
    graph_dir.join(stem).with_extension("puml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Chemin temporaire propre au test et au processus.
    fn temp_path(test_name: &str, file: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "phone_trie_{test_name}_{}_{file}",
            std::process::id()
        ))
    }

    #[test]
    fn mindmap_path_uses_input_name_without_extension() {
        let path = mindmap_path(Path::new("data/02_common_parts.json"), Path::new("graph"));
        assert_eq!(path, Path::new("graph").join("02_common_parts.puml"));
    }

    #[test]
    fn mindmap_path_falls_back_to_default() {
        let path = mindmap_path(Path::new(".."), Path::new("graph"));
        assert_eq!(path, Path::new("graph").join("mindmap.puml"));
    }

    /// Trie construit à partir de couples (numéro, nom).
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
    fn prefix_node_shows_value_and_children() {
        // 15 est un préfixe de 1515.
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
    fn escape_keeps_plain_names_unchanged() {
        assert_eq!(escape_value("Zoé Vincent"), "Zoé Vincent");
        assert_eq!(escape_value("Alice"), "Alice");
    }

    #[test]
    fn escape_replaces_line_breaks() {
        assert_eq!(escape_value("Alice\nDupont"), "Alice Dupont");
        assert_eq!(escape_value("Alice\r\nDupont"), "Alice Dupont");
        assert_eq!(escape_value("Alice\tDupont\r"), "Alice Dupont ");
    }

    #[test]
    fn escape_neutralizes_markup() {
        assert_eq!(escape_value("*Bob"), "~*Bob");
        assert_eq!(escape_value("**gras**"), "~*~*gras~*~*");
        assert_eq!(escape_value("<b>x</b>"), "~<b>x~<~/b>");
        assert_eq!(escape_value("//italique//"), "~/~/italique~/~/");
        assert_eq!(escape_value("[[lien]]"), "~[~[lien~]~]");
        assert_eq!(escape_value("--barré--"), "~-~-barré~-~-");
        assert_eq!(escape_value("__souligné__"), "~_~_souligné~_~_");
        assert_eq!(escape_value("\"\"mono\"\""), "~\"~\"mono~\"~\"");
        assert_eq!(
            escape_value("~~vague~~"),
            "<U+007E><U+007E>vague<U+007E><U+007E>"
        );
        assert_eq!(escape_value("Jean-Pierre"), "Jean~-Pierre");
    }

    #[test]
    fn special_names_stay_on_one_line() {
        let trie = trie_of(&[("12", "*Bob\nDupont"), ("15", "SAMU")]);
        let expected = "\
@startmindmap
* Racine
** 1
*** 2 : ~*Bob Dupont
*** 5 : SAMU
@endmindmap
";
        assert_eq!(to_mindmap(&trie), expected);
    }

    #[test]
    fn mindmap_written_to_file_can_be_read_back() {
        let trie = trie_of(&[("0612", "Alice"), ("0634", "Bob")]);
        let content = to_mindmap(&trie);
        let path = temp_path("mindmap_written_to_file_can_be_read_back", "roundtrip.puml");

        write_mindmap(&content, &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_creates_file_with_content() {
        let path = temp_path("write_mindmap_creates_file_with_content", "ok.puml");
        let content = "@startmindmap\n* Racine\n@endmindmap\n";

        write_mindmap(content, &path).unwrap();

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, content);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_overwrites_existing_file() {
        let path = temp_path("write_mindmap_overwrites_existing_file", "overwrite.puml");
        fs::write(&path, "ancien contenu beaucoup plus long").unwrap();

        write_mindmap("nouveau", &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "nouveau");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_mindmap_creates_missing_directories() {
        let dir = temp_path(
            "write_mindmap_creates_missing_directories",
            "dossier_inexistant",
        );
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("sous_dossier").join("x.puml");

        write_mindmap("contenu", &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "contenu");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn write_mindmap_under_a_file_returns_io_error() {
        // Le parent est un fichier, pas un dossier.
        let file = temp_path(
            "write_mindmap_under_a_file_returns_io_error",
            "pas_un_dossier.txt",
        );
        fs::write(&file, "fichier").unwrap();

        let result = write_mindmap("contenu", &file.join("x.puml"));

        assert!(matches!(result, Err(AppError::Io(_))));
        fs::remove_file(&file).unwrap();
    }
}
