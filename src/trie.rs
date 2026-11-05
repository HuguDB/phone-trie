//! Trie (arbre de préfixes) de numéros de téléphone, codé sans crate.
//!
//! Chaque nœud correspond à un chiffre ; les numéros qui partagent un préfixe
//! partagent les nœuds de ce préfixe. Une valeur (par exemple le nom du
//! contact) est stockée sur le nœud du dernier chiffre de chaque numéro.
//! Les enfants sont rangés dans un `Vec<(char, Node<T>)>` : au plus 10
//! enfants par nœud, donc une recherche linéaire suffit.

use crate::error::AppError;

/// Un nœud du trie : ses enfants et une valeur si un numéro se termine ici.
#[derive(Debug)]
pub struct Node<T> {
    children: Vec<(char, Node<T>)>,
    value: Option<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            value: None,
        }
    }
}

impl<T> Node<T> {
    /// Parcourt les enfants dans leur ordre d'insertion.
    pub fn children(&self) -> impl Iterator<Item = (&char, &Node<T>)> {
        self.children.iter().map(|(digit, node)| (digit, node))
    }

    /// Renvoie la valeur du nœud, si un numéro se termine ici.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    fn descendants(&self) -> usize {
        self.children
            .iter()
            .map(|(_, node)| 1 + node.descendants())
            .sum()
    }

    fn stored_values(&self) -> usize {
        let own = usize::from(self.value.is_some());
        own + self
            .children
            .iter()
            .map(|(_, node)| node.stored_values())
            .sum::<usize>()
    }

    fn child(&self, digit: char) -> Option<&Node<T>> {
        self.children
            .iter()
            .find(|(c, _)| *c == digit)
            .map(|(_, node)| node)
    }

    fn child_or_insert(&mut self, digit: char) -> &mut Node<T> {
        let index = match self.children.iter().position(|(c, _)| *c == digit) {
            Some(index) => index,
            None => {
                self.children.push((digit, Node::default()));
                self.children.len() - 1
            }
        };
        &mut self.children[index].1
    }
}

/// Arbre de préfixes qui associe une valeur à chaque numéro.
#[derive(Debug)]
pub struct Trie<T> {
    root: Node<T>,
}

impl<T> Trie<T> {
    /// Crée un trie vide.
    pub fn new() -> Self {
        Self {
            root: Node::default(),
        }
    }

    /// Insère un numéro. S'il existe déjà, sa valeur est remplacée.
    pub fn insert(&mut self, number: &str, value: T) -> Result<(), AppError> {
        validate(number)?;

        let mut node = &mut self.root;
        for digit in number.chars() {
            node = node.child_or_insert(digit);
        }
        node.value = Some(value);
        Ok(())
    }

    /// Indique si ce numéro a été inséré (un simple préfixe ne compte pas).
    pub fn contains(&self, number: &str) -> bool {
        self.find_node(number)
            .is_some_and(|node| node.value.is_some())
    }

    /// Renvoie la racine, pour parcourir le trie en lecture seule.
    pub fn root(&self) -> &Node<T> {
        &self.root
    }

    /// Renvoie le nombre de nœuds, racine exclue.
    pub fn node_count(&self) -> usize {
        self.root.descendants()
    }

    /// Renvoie le nombre de numéros stockés (un doublon compte une fois).
    ///
    /// Parcourt tout le trie : la complexité est proportionnelle au nombre
    /// de nœuds.
    pub fn len(&self) -> usize {
        self.root.stored_values()
    }

    /// Indique si le trie ne contient aucun numéro.
    ///
    /// En temps constant : sans suppression, et comme `insert` valide le
    /// numéro avant de créer le moindre nœud, la racine n'a des enfants que
    /// si au moins un numéro a été inséré.
    pub fn is_empty(&self) -> bool {
        self.root.children.is_empty()
    }

    fn find_node(&self, number: &str) -> Option<&Node<T>> {
        let mut node = &self.root;
        for digit in number.chars() {
            node = node.child(digit)?;
        }
        Some(node)
    }
}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn validate(number: &str) -> Result<(), AppError> {
    if number.is_empty() {
        return Err(AppError::EmptyNumber);
    }
    if let Some(c) = number.chars().find(|c| !c.is_ascii_digit()) {
        return Err(AppError::InvalidDigit(c, number.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digits<T>(node: &Node<T>) -> Vec<char> {
        node.children.iter().map(|(c, _)| *c).collect()
    }

    #[test]
    fn new_trie_has_empty_root() {
        let trie: Trie<String> = Trie::new();
        assert!(trie.root.children.is_empty());
        assert!(trie.root.value.is_none());
    }

    #[test]
    fn insert_rejects_empty_number() {
        let mut trie = Trie::new();
        let result = trie.insert("", "Alice".to_string());
        assert!(matches!(result, Err(AppError::EmptyNumber)));
    }

    #[test]
    fn insert_rejects_non_digit_character() {
        let mut trie = Trie::new();
        let result = trie.insert("06a2", "Alice".to_string());
        assert!(matches!(result, Err(AppError::InvalidDigit('a', n)) if n == "06a2"));
    }

    #[test]
    fn invalid_number_leaves_trie_unchanged() {
        let mut trie = Trie::new();
        let _ = trie.insert("06a2", "Alice".to_string());
        assert!(trie.root.children.is_empty());
    }

    #[test]
    fn insert_stores_value_on_last_digit_only() {
        let mut trie = Trie::new();
        trie.insert("112", "Urgences".to_string()).unwrap();

        assert!(trie.find_node("1").unwrap().value.is_none());
        assert!(trie.find_node("11").unwrap().value.is_none());
        assert_eq!(
            trie.find_node("112").unwrap().value.as_deref(),
            Some("Urgences")
        );
    }

    #[test]
    fn numbers_with_common_prefix_share_nodes() {
        let mut trie = Trie::new();
        trie.insert("0612", "Alice".to_string()).unwrap();
        trie.insert("0634", "Bob".to_string()).unwrap();

        assert_eq!(digits(&trie.root), vec!['0']);
        assert_eq!(digits(trie.find_node("0").unwrap()), vec!['6']);
        assert_eq!(digits(trie.find_node("06").unwrap()), vec!['1', '3']);
        assert_eq!(trie.node_count(), 6);
    }

    #[test]
    fn children_keep_insertion_order() {
        let mut trie = Trie::new();
        trie.insert("0412578440", "Alice".to_string()).unwrap();
        trie.insert("0412199803", "Bob".to_string()).unwrap();

        assert_eq!(digits(trie.find_node("0412").unwrap()), vec!['5', '1']);
    }

    #[test]
    fn duplicate_number_overwrites_value() {
        let mut trie = Trie::new();
        trie.insert("15", "SAMU".to_string()).unwrap();
        trie.insert("15", "Urgences".to_string()).unwrap();

        assert_eq!(
            trie.find_node("15").unwrap().value.as_deref(),
            Some("Urgences")
        );
        assert_eq!(digits(&trie.root), vec!['1']);
    }

    #[test]
    fn contains_inserted_number() {
        let mut trie = Trie::new();
        trie.insert("0612", "Alice".to_string()).unwrap();
        assert!(trie.contains("0612"));
    }

    #[test]
    fn contains_rejects_unknown_number() {
        let mut trie = Trie::new();
        trie.insert("0612", "Alice".to_string()).unwrap();
        assert!(!trie.contains("0634"));
        assert!(!trie.contains("06123"));
    }

    #[test]
    fn prefix_alone_is_not_contained() {
        let mut trie = Trie::new();
        trie.insert("0612", "Alice".to_string()).unwrap();
        assert!(!trie.contains("061"));
        assert!(!trie.contains(""));
    }

    #[test]
    fn prefix_number_inserted_first_keeps_both_values() {
        let mut trie = Trie::new();
        trie.insert("15", "SAMU".to_string()).unwrap();
        trie.insert("1515", "Bob".to_string()).unwrap();

        assert!(trie.contains("15"));
        assert!(trie.contains("1515"));
        assert_eq!(
            trie.find_node("15").unwrap().value(),
            Some(&"SAMU".to_string())
        );
        assert_eq!(
            trie.find_node("1515").unwrap().value(),
            Some(&"Bob".to_string())
        );
        assert!(!trie.contains("151"));
        assert_eq!(trie.node_count(), 4);
    }

    #[test]
    fn prefix_number_inserted_last_keeps_both_values() {
        let mut trie = Trie::new();
        trie.insert("1515", "Bob".to_string()).unwrap();
        trie.insert("15", "SAMU".to_string()).unwrap();

        assert!(trie.contains("15"));
        assert!(trie.contains("1515"));
        assert_eq!(
            trie.find_node("15").unwrap().value(),
            Some(&"SAMU".to_string())
        );
        assert_eq!(
            trie.find_node("1515").unwrap().value(),
            Some(&"Bob".to_string())
        );
        assert!(!trie.contains("151"));
        assert_eq!(trie.node_count(), 4);
    }

    #[test]
    fn empty_trie_has_len_zero() {
        let trie: Trie<String> = Trie::new();
        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());
    }

    #[test]
    fn len_counts_inserted_numbers() {
        let mut trie = Trie::new();
        trie.insert("0612", "Alice".to_string()).unwrap();
        trie.insert("0634", "Bob".to_string()).unwrap();
        assert_eq!(trie.len(), 2);
        assert!(!trie.is_empty());
    }

    #[test]
    fn len_counts_prefix_numbers_separately() {
        let mut trie = Trie::new();
        trie.insert("15", "SAMU".to_string()).unwrap();
        trie.insert("1515", "Bob".to_string()).unwrap();
        assert_eq!(trie.len(), 2);
        assert_eq!(trie.node_count(), 4);
    }

    #[test]
    fn len_ignores_duplicates_and_invalid_numbers() {
        let mut trie = Trie::new();
        trie.insert("15", "SAMU".to_string()).unwrap();
        trie.insert("15", "Urgences".to_string()).unwrap();
        let _ = trie.insert("1a", "Eve".to_string());
        let _ = trie.insert("", "Vide".to_string());
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn empty_trie_has_no_nodes() {
        let trie: Trie<String> = Trie::new();
        assert_eq!(trie.node_count(), 0);
    }

    #[test]
    fn empty_trie_contains_nothing() {
        let trie: Trie<String> = Trie::new();
        assert!(!trie.contains("0612"));
        assert!(!trie.contains(""));
    }

    #[test]
    fn international_prefix_is_rejected() {
        let mut trie = Trie::new();
        let result = trie.insert("+33612345678", "Alice".to_string());
        assert!(matches!(result, Err(AppError::InvalidDigit('+', n)) if n == "+33612345678"));
    }

    #[test]
    fn spaces_are_rejected() {
        let mut trie = Trie::new();
        let result = trie.insert("06 12 34", "Alice".to_string());
        assert!(matches!(result, Err(AppError::InvalidDigit(' ', n)) if n == "06 12 34"));
    }

    #[test]
    fn non_ascii_characters_are_rejected() {
        let mut trie = Trie::new();
        // Lettre accentuée.
        let accent = trie.insert("06é2", "Alice".to_string());
        assert!(matches!(accent, Err(AppError::InvalidDigit('é', _))));
        // Chiffres arabo-indiens : numériques pour Unicode, mais pas ASCII.
        let arabic = trie.insert("٠٦١٢", "Bob".to_string());
        assert!(matches!(arabic, Err(AppError::InvalidDigit('٠', _))));
        // Chiffres pleine chasse.
        let fullwidth = trie.insert("０６", "Chloé".to_string());
        assert!(matches!(fullwidth, Err(AppError::InvalidDigit('０', _))));

        assert_eq!(trie.node_count(), 0);
    }

    #[test]
    fn accessors_expose_structure_in_read_only() {
        let mut trie = Trie::new();
        trie.insert("12", "Alice".to_string()).unwrap();
        trie.insert("15", "Bob".to_string()).unwrap();

        let (digit, one) = trie.root().children().next().unwrap();
        assert_eq!(*digit, '1');
        assert_eq!(one.value(), None);

        let leaves: Vec<(char, Option<&String>)> = one
            .children()
            .map(|(digit, node)| (*digit, node.value()))
            .collect();
        let alice = "Alice".to_string();
        let bob = "Bob".to_string();
        assert_eq!(leaves, vec![('2', Some(&alice)), ('5', Some(&bob))]);
    }
}
