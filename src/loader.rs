//! Lecture du fichier JSON de contacts et construction du trie.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;
use crate::trie::Trie;

/// Contact du fichier JSON : `{ "nb": ..., "name": ... }`. L'ancien nom de
/// champ `"phone"` est aussi accepté.
#[derive(Debug, Deserialize)]
pub struct Contact {
    /// Nom du contact.
    pub name: String,
    /// Numéro, pas encore validé (champ `"nb"` ou `"phone"`).
    #[serde(alias = "nb")]
    pub phone: String,
}

/// Lit les contacts d'un fichier JSON. Renvoie `AppError::Io` si le fichier
/// est illisible et `AppError::Json` si son contenu est invalide.
pub fn load_contacts(path: &Path) -> Result<Vec<Contact>, AppError> {
    let content = fs::read_to_string(path)?;
    let contacts = serde_json::from_str(&content)?;
    Ok(contacts)
}

/// Contact ignoré par [`build_trie`] car son numéro est invalide.
#[derive(Debug)]
pub struct Rejected {
    /// Nom du contact.
    pub name: String,
    /// Raison du rejet.
    pub error: AppError,
}

/// Construit le trie (numéro vers nom). Les contacts au numéro invalide ne
/// sont pas insérés et sont renvoyés à part, dans l'ordre du fichier.
pub fn build_trie(contacts: Vec<Contact>) -> (Trie<String>, Vec<Rejected>) {
    let mut trie = Trie::new();
    let mut rejected = Vec::new();
    for contact in contacts {
        // On clone le nom pour pouvoir l'afficher si le numéro est rejeté.
        if let Err(error) = trie.insert(&contact.phone, contact.name.clone()) {
            rejected.push(Rejected {
                name: contact.name,
                error,
            });
        }
    }
    (trie, rejected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Chemin d'un fichier de `data/`.
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
        let contacts = load_contacts(&data_path("03_invalid_number.json")).unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].phone, "06-12-34-56-78");
    }

    #[test]
    fn malformed_file_returns_json_error() {
        let result = load_contacts(&data_path("07_malformed.json"));
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

    #[test]
    fn nb_field_is_read_as_phone() {
        let json = r#"[{ "nb": "0412578440", "name": "Alice" }]"#;
        let contacts: Vec<Contact> = serde_json::from_str(json).unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Alice");
        assert_eq!(contacts[0].phone, "0412578440");
    }

    #[test]
    fn phone_field_is_still_accepted() {
        let json = r#"[{ "name": "Bob", "phone": "0478123456" }]"#;
        let contacts: Vec<Contact> = serde_json::from_str(json).unwrap();
        assert_eq!(contacts[0].name, "Bob");
        assert_eq!(contacts[0].phone, "0478123456");
    }

    fn contact(name: &str, phone: &str) -> Contact {
        Contact {
            name: name.to_string(),
            phone: phone.to_string(),
        }
    }

    #[test]
    fn subject_file_loads_five_contacts_with_short_numbers() {
        let contacts = load_contacts(&data_path("04_common_parts.json")).unwrap();
        assert_eq!(contacts.len(), 5);
        assert_eq!(contacts[0].name, "Alice");
        assert_eq!(contacts[0].phone, "0412578440");

        let (trie, rejected) = build_trie(contacts);

        assert!(rejected.is_empty());
        assert_eq!(trie.len(), 5);
        assert!(trie.contains("15"));
        assert!(trie.contains("112"));
    }

    #[test]
    fn common_parts_builds_four_numbers() {
        let contacts = load_contacts(&data_path("02_common_parts.json")).unwrap();
        let phones: Vec<String> = contacts.iter().map(|c| c.phone.clone()).collect();

        let (trie, rejected) = build_trie(contacts);

        assert!(rejected.is_empty());
        assert_eq!(phones.len(), 4);
        for phone in &phones {
            assert!(trie.contains(phone), "{phone} absent du trie");
        }
    }

    #[test]
    fn invalid_number_file_gives_one_rejection() {
        let contacts = load_contacts(&data_path("03_invalid_number.json")).unwrap();

        let (trie, rejected) = build_trie(contacts);

        assert_eq!(trie.node_count(), 0);
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].name, "Eve");
        assert!(matches!(
            &rejected[0].error,
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

        let (trie, rejected) = build_trie(contacts);

        assert!(trie.contains("0612"));
        assert!(trie.contains("0634"));
        assert!(!trie.contains("06 12"));
        assert_eq!(rejected.len(), 2);
        assert_eq!(rejected[0].name, "Eve");
        assert!(matches!(rejected[0].error, AppError::InvalidDigit(' ', _)));
        assert_eq!(rejected[1].name, "Vide");
        assert!(matches!(rejected[1].error, AppError::EmptyNumber));
    }
}
