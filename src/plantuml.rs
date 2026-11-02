//! Export du trie au format PlantUML MindMap.
//!
//! La génération du texte (`to_mindmap`) dépend de `Trie` et sera ajoutée
//! quand la partie A sera disponible. L'écriture sur disque est déjà là.

use std::fs;
use std::path::Path;

use crate::error::AppError;

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
