//! Lecture des contacts depuis un fichier JSON.
//!
//! Format attendu : un tableau d'objets `{ "name": string, "phone": string }`.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;
use crate::trie::Trie;

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

/// Construit un trie qui associe chaque numéro au nom du contact.
///
/// Un numéro invalide (vide ou contenant autre chose que des chiffres)
/// n'arrête pas la construction : le contact est ignoré et l'erreur est
/// ajoutée à la liste renvoyée, dans l'ordre des contacts. Rien n'est affiché
/// ici, c'est à l'appelant de signaler les numéros rejetés.
///
/// Il n'y a pas de `Result` : les seules erreurs bloquantes (fichier
/// introuvable, JSON invalide) sont déjà renvoyées par [`load_contacts`].
pub fn build_trie(contacts: Vec<Contact>) -> (Trie<String>, Vec<AppError>) {
    let mut trie = Trie::new();
    let mut errors = Vec::new();
    for contact in contacts {
        if let Err(err) = trie.insert(&contact.phone, contact.name) {
            errors.push(err);
        }
    }
    (trie, errors)
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

    fn contact(name: &str, phone: &str) -> Contact {
        Contact {
            name: name.to_string(),
            phone: phone.to_string(),
        }
    }

    #[test]
    fn common_parts_file_builds_four_numbers_without_error() {
        let contacts = load_contacts(&data_path("02_common_parts.json")).unwrap();
        let phones: Vec<String> = contacts.iter().map(|c| c.phone.clone()).collect();

        let (trie, errors) = build_trie(contacts);

        assert!(errors.is_empty());
        assert_eq!(phones.len(), 4);
        for phone in &phones {
            assert!(trie.contains(phone), "{phone} absent du trie");
        }
    }

    #[test]
    fn invalid_number_file_gives_empty_trie_and_one_error() {
        let contacts = load_contacts(&data_path("03_invalid_number.json")).unwrap();

        let (trie, errors) = build_trie(contacts);

        assert_eq!(trie.node_count(), 0);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            AppError::InvalidDigit('-', n) if n == "06-12-34-56-78"
        ));
    }

    #[test]
    fn mixed_contacts_keep_only_valid_numbers() {
        let contacts = vec![
            contact("Alice", "0612"),
            contact("Eve", "06 12"),
            contact("Bob", "0634"),
            contact("Vide", ""),
        ];

        let (trie, errors) = build_trie(contacts);

        assert!(trie.contains("0612"));
        assert!(trie.contains("0634"));
        assert!(!trie.contains("06 12"));
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], AppError::InvalidDigit(' ', _)));
        assert!(matches!(errors[1], AppError::EmptyNumber));
    }
}
