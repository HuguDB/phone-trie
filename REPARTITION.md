# Répartition des tâches

Le programme lit un fichier JSON de contacts, construit un trie en mémoire et
génère un MindMap PlantUML.

- **Partie A, le trie** (Célian) : `src/trie.rs`.
- **Partie B, les entrées/sorties** (Hugo) : `src/error.rs`, `src/loader.rs`,
  `src/plantuml.rs`, `src/main.rs`.

## Contraintes du sujet

- Trie codé sans crate.
- `#![forbid(unsafe_code)]` dans `lib.rs`, aucun code `unsafe`.
- Pas de `unwrap()` ni `expect()` hors des tests.
- Numéros sans préfixe international. Insertion et parcours seulement.
- JSON de test dans `data/`, `.puml` générés dans `graph/`.
- README avec la commande de lancement.

## API commune

Trie (`src/trie.rs`) :

```rust
pub struct Trie<T> { root: Node<T> }

pub struct Node<T> {
    children: Vec<(char, Node<T>)>, // 10 enfants au plus
    value: Option<T>,               // Some si un numéro se termine ici
}

impl<T> Trie<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, number: &str, value: T) -> Result<(), AppError>;
    pub fn contains(&self, number: &str) -> bool;
    pub fn root(&self) -> &Node<T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn node_count(&self) -> usize;
    pub fn total_digits(&self) -> usize;
}

impl<T> Node<T> {
    pub fn children(&self) -> impl Iterator<Item = (&char, &Node<T>)>;
    pub fn value(&self) -> Option<&T>;
}
```

Un numéro déjà présent voit sa valeur remplacée.

Erreurs (`src/error.rs`) :

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("numéro vide")]
    EmptyNumber,
    #[error("caractère invalide '{0}' dans le numéro {1}")]
    InvalidDigit(char, String),
    #[error("erreur d'entrée/sortie : {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON invalide : {0}")]
    Json(#[from] serde_json::Error),
}
```

Entrées/sorties :

```rust
// loader.rs
pub fn load_contacts(path: &Path) -> Result<Vec<Contact>, AppError>;
pub fn build_trie(contacts: Vec<Contact>) -> (Trie<String>, Vec<Rejected>);

// plantuml.rs
pub fn to_mindmap<T: Display>(trie: &Trie<T>) -> String;
pub fn write_mindmap(content: &str, path: &Path) -> Result<(), AppError>;
pub fn mindmap_path(input: &Path, graph_dir: &Path) -> PathBuf;
```

Format JSON : un tableau d'objets `{ "name": string, "phone": string }`.

`build_trie` n'arrête pas sur un numéro invalide : le contact est renvoyé dans
la liste des rejets et `main` affiche un avertissement. Un fichier introuvable
ou un JSON invalide reste une erreur bloquante.

## Tâches

Partie A :

- [x] `Node<T>`, `Trie<T>`, `new()` et `Default`.
- [x] `insert` avec validation (`EmptyNumber`, `InvalidDigit`).
- [x] `contains`, `root()`, `Node::children()`, `Node::value()`.
- [x] Doublons : la valeur est remplacée.
- [x] Bonus : `len()`, `is_empty()`, `node_count()`, `total_digits()`.
- [x] Tests unitaires.

Partie B :

- [x] `AppError` et dépendances (`serde`, `serde_json`, `thiserror`).
- [x] `load_contacts` et `build_trie`.
- [x] `to_mindmap` (enfants triés, valeurs échappées) et `write_mindmap`.
- [x] `main.rs` : argument, avertissements, statistiques, codes de sortie.
- [x] Tests unitaires et tests du binaire (`tests/cli.rs`).
- [x] Vérification des `.puml` dans PlantUML (`cargo test -- --ignored`).

À deux :

- [x] Fichiers de test dans `data/`.
- [x] README.
- [ ] Relire le code de l'autre.
- [ ] Préparer la soutenance (5 minutes).

## Conventions

- On travaille sur `main`, en petits commits au format conventional commits
  (`feat:`, `fix:`, `test:`, `docs:`, `refactor:`, `chore:`).
- Avant chaque push : `cargo fmt --check`, `cargo clippy --all-targets` sans
  warning et `cargo test`, puis `git pull --rebase`.
- Historique linéaire, sans commit de merge.
