//! Erreurs communes au projet.
//!
//! Toutes les fonctions faillibles du projet renvoient `Result<_, AppError>`.

/// Erreur commune à toutes les parties du projet.
///
/// `thiserror` génère l'implémentation de `std::error::Error` et de `Display`
/// à partir des attributs `#[error(...)]`. Les variantes marquées `#[from]`
/// se construisent automatiquement depuis l'erreur d'origine, ce qui permet
/// d'utiliser l'opérateur `?` sans conversion manuelle.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Le numéro fourni est une chaîne vide.
    #[error("numéro vide")]
    EmptyNumber,
    /// Le numéro contient un caractère qui n'est pas un chiffre.
    /// Le premier champ est le caractère fautif, le second le numéro complet.
    #[error("caractère invalide '{0}' dans le numéro {1}")]
    InvalidDigit(char, String),
    /// Erreur d'entrée/sortie (fichier introuvable, droits insuffisants...).
    #[error("erreur d'entrée/sortie : {0}")]
    Io(#[from] std::io::Error),
    /// Le contenu du fichier n'est pas du JSON valide pour notre format.
    #[error("JSON invalide : {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_number_message() {
        let err = AppError::EmptyNumber;
        assert_eq!(err.to_string(), "numéro vide");
    }

    #[test]
    fn invalid_digit_message_contains_char_and_number() {
        let err = AppError::InvalidDigit('a', "06a1".to_string());
        assert_eq!(
            err.to_string(),
            "caractère invalide 'a' dans le numéro 06a1"
        );
    }

    #[test]
    fn io_error_is_converted_with_from() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "fichier absent");
        let err: AppError = io.into();
        assert!(matches!(err, AppError::Io(_)));
        assert_eq!(err.to_string(), "erreur d'entrée/sortie : fichier absent");
    }

    #[test]
    fn json_error_is_converted_with_from() {
        // `{` seul est un JSON incomplet : serde_json renvoie une erreur.
        let json_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let err: AppError = json_err.into();
        assert!(matches!(err, AppError::Json(_)));
        assert!(err.to_string().starts_with("JSON invalide : "));
    }

    #[test]
    fn question_mark_operator_propagates_io_error() {
        // Simule une fonction de la partie B qui lit un fichier inexistant.
        fn read_missing() -> Result<String, AppError> {
            let content = std::fs::read_to_string("ce_fichier_n_existe_pas.json")?;
            Ok(content)
        }
        assert!(matches!(read_missing(), Err(AppError::Io(_))));
    }
}
