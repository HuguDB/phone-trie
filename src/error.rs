//! Erreurs du projet.

/// Erreur renvoyée par les fonctions du projet.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Numéro vide.
    #[error("numéro vide")]
    EmptyNumber,
    /// Caractère qui n'est pas un chiffre, avec le numéro complet.
    #[error("caractère invalide '{0}' dans le numéro {1}")]
    InvalidDigit(char, String),
    /// Lecture ou écriture de fichier impossible.
    #[error("erreur d'entrée/sortie : {0}")]
    Io(#[from] std::io::Error),
    /// JSON mal formé ou qui ne suit pas le format attendu.
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
        let json_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let err: AppError = json_err.into();
        assert!(matches!(err, AppError::Json(_)));
        assert!(err.to_string().starts_with("JSON invalide : "));
    }

    #[test]
    fn question_mark_operator_propagates_io_error() {
        fn read_missing() -> Result<String, AppError> {
            let content = std::fs::read_to_string("ce_fichier_n_existe_pas.json")?;
            Ok(content)
        }
        assert!(matches!(read_missing(), Err(AppError::Io(_))));
    }
}
