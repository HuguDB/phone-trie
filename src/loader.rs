//! Lecture des contacts depuis un fichier JSON.
//!
//! Format attendu : un tableau d'objets `{ "name": string, "phone": string }`.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;

/// Un contact tel qu'il apparaît dans le fichier JSON.
///
/// Les noms des champs doivent correspondre exactement aux clés du JSON :
/// c'est ce que `#[derive(Deserialize)]` utilise pour remplir la struct.
/// Pour changer de format, il suffit de modifier cette struct.
#[derive(Debug, Deserialize)]
pub struct Contact {
    /// Nom du contact, stocké en fin de numéro dans le trie.
    pub name: String,
    /// Numéro de téléphone brut, sans validation à ce stade.
    pub phone: String,
}

/// Lit le fichier JSON situé à `path` et renvoie la liste des contacts.
///
/// # Erreurs
///
/// - `AppError::Io` si le fichier ne peut pas être lu (absent, droits...).
/// - `AppError::Json` si le contenu n'est pas un tableau de contacts valide.
pub fn load_contacts(path: &Path) -> Result<Vec<Contact>, AppError> {
    let content = fs::read_to_string(path)?;
    let contacts = serde_json::from_str(&content)?;
    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Chemin absolu d'un fichier du dossier `data/` du projet.
    fn data_path(file: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join(file)
    }

    #[test]
    fn simple_file_returns_two_contacts() {
        let contacts = load_contacts(&data_path("01_simple.json")).unwrap();
        assert_eq!(contacts.len(), 2);
        assert_eq!(contacts[0].name, "Alice");
        assert_eq!(contacts[0].phone, "0612345678");
        assert_eq!(contacts[1].name, "Bob");
        assert_eq!(contacts[1].phone, "0478123456");
    }

    #[test]
    fn invalid_number_is_still_valid_json() {
        // Le format du numéro n'est pas vérifié ici : c'est le rôle du trie.
        let contacts = load_contacts(&data_path("03_invalid_number.json")).unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].phone, "06-12-34-56-78");
    }

    #[test]
    fn malformed_file_returns_json_error() {
        let result = load_contacts(&data_path("04_malformed.json"));
        assert!(matches!(result, Err(AppError::Json(_))));
    }

    #[test]
    fn missing_file_returns_io_error() {
        let result = load_contacts(&data_path("inexistant.json"));
        assert!(matches!(result, Err(AppError::Io(_))));
    }

    #[test]
    fn empty_array_returns_empty_vec() {
        let contacts = load_contacts(&data_path("05_empty.json")).unwrap();
        assert!(contacts.is_empty());
    }
}
