# Projet Rust — Répartition des tâches

## Contexte et règles communes

Le programme lit un fichier JSON de numéros, construit un trie en mémoire et génère un fichier PlantUML au format MindMap. Le travail est découpé en deux parties indépendantes : **A, le trie**, et **B, les entrées/sorties**.

Contraintes du sujet, valables pour tous les deux :

- Le trie est codé from scratch, sans aucun crate pour cette partie.
- `#![forbid(unsafe_code)]` figure dans `lib.rs`. Aucun code unsafe.
- Aucun `unwrap()` ni `expect()` hors des tests : on renvoie des `Result`.
- Les numéros n'ont pas de préfixe international. Seules l'insertion et le parcours sont demandés, pas de suppression ni d'update.
- Un dossier `data/` contient les JSON de test, un dossier `graph/` contient les `.puml` générés.
- Le README est obligatoire, avec au minimum la commande de lancement.
- Les commits suivent les conventional commits, car l'historique git est noté.
- Le code passe `cargo clippy` sans warning et `cargo test` au vert avant chaque merge.

## Structure du repo et API commune

On valide cette API ensemble avant de coder. Ensuite, chacun avance sans attendre l'autre.

```
phone_trie/
├── Cargo.toml / Cargo.lock
├── README.md
├── data/            # JSON de test
├── graph/           # .puml générés
└── src/
    ├── main.rs      # minimal : appelle la lib (B)
    ├── lib.rs       # forbid(unsafe_code) + module tree
    ├── error.rs     # erreurs communes (B, défini ensemble)
    ├── trie.rs      # Partie A
    ├── loader.rs    # Partie B
    └── plantuml.rs  # Partie B
```

API publique du trie (`src/trie.rs`) :

```rust
pub struct Trie<T> { root: Node<T> }

pub struct Node<T> {
    children: Vec<(char, Node<T>)>, // au plus 10 enfants (chiffres 0-9)
    value: Option<T>,               // Some si un numéro se termine ici
}

impl<T> Trie<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, number: &str, value: T) -> Result<(), AppError>;
    pub fn contains(&self, number: &str) -> bool;
    pub fn root(&self) -> &Node<T>;
}

impl<T> Node<T> {
    pub fn children(&self) -> impl Iterator<Item = (&char, &Node<T>)>;
    pub fn value(&self) -> Option<&T>;
}
```

Erreurs communes (`src/error.rs`, avec `thiserror`) :

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("numéro vide")]
    EmptyNumber,
    #[error("caractère invalide '{0}' dans le numéro {1}")]
    InvalidDigit(char, String),
    #[error("erreur de lecture : {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON invalide : {0}")]
    Json(#[from] serde_json::Error),
}
```

`T` est la donnée stockée en fin de numéro, par exemple le nom du contact.

Choix du `Vec<(char, Node<T>)>` plutôt que d'une `HashMap` : un nœud a au plus 10 enfants (les chiffres 0-9). Une recherche linéaire sur 10 éléments est aussi rapide qu'un hash, un `Vec` est plus compact en mémoire (pas de table de hachage par nœud), et l'ordre des enfants est déterministe (ordre d'insertion). L'export PlantUML trie quand même les clés : la sortie ne dépend pas de l'ordre dans lequel les numéros ont été insérés.

Format des JSON (question tranchée) : un tableau d'objets `{ "name": string, "phone": string }`. `T` vaut donc `String`, le nom du contact.

## Partie A : le cœur du trie

Objectif : un trie générique, codé from scratch, qui insère des numéros et expose un parcours en lecture seule. Fichier : `src/trie.rs`. Branche : `feat/trie`.

Tâches, dans l'ordre TDD (un test qui échoue, puis le code minimal) :

- [x] Créer `Node<T>` et `Trie<T>`, puis implémenter `Default` pour `Node` et `Trie::new()`.
- [x] `insert` : valider le numéro (non vide, uniquement des chiffres `0-9`), sinon renvoyer `AppError::EmptyNumber` ou `AppError::InvalidDigit`.
- [x] `insert` : descendre caractère par caractère avec `node = node.child_or_insert(c);` (cherche l'enfant dans le `Vec`, le crée s'il manque, renvoie `&mut Node<T>`), puis stocker `value` sur le dernier nœud.
- [x] Décider et documenter le comportement en cas de doublon : la valeur est écrasée (documenté sur `insert`, testé).
- [x] `contains(&self, number)` : parcours en lecture, `false` si un caractère manque ou si le dernier nœud n'a pas de valeur.
- [x] `root()`, `Node::children()` et `Node::value()` pour que la partie B parcoure l'arbre sans accéder aux champs privés.
- [ ] Option bonus : `len()` (nombre de numéros) et `node_count()` (nombre de nœuds), utiles pour montrer le gain mémoire en soutenance. `node_count()` est fait (racine exclue) ; `len()` reste à faire.
- [x] Doc comments `///` sur chaque élément public.

Tests unitaires attendus (`#[cfg(test)] mod tests` dans `trie.rs`) :

- Insérer un numéro puis `contains` renvoie `true`.
- Deux numéros avec un préfixe commun partagent leurs nœuds (vérifié par `node_count`).
- Un préfixe seul n'est pas considéré comme un numéro inséré.
- Numéro vide et caractère non numérique renvoient la bonne erreur.
- Comportement du doublon conforme à ce qui a été décidé.

C'est terminé quand tous les tests passent, que `cargo clippy` est propre et que la PR est relue par B.

À savoir expliquer en soutenance : pourquoi `node = node.child_or_insert(c)` compile alors qu'on réassigne une référence `&mut` à chaque tour (l'ancienne référence est « consommée » par l'appel et remplacée par celle, plus courte, vers l'enfant), le choix du `Vec<(char, Node<T>)>` pour les enfants (10 enfants au plus : recherche linéaire aussi rapide qu'un hash, moins de mémoire, ordre déterministe), la complexité de l'insertion (proportionnelle à la longueur du numéro) et le gain mémoire apporté par les préfixes partagés.

## Partie B : entrées/sorties

Objectif : lire le JSON, gérer les erreurs, générer le MindMap PlantUML et brancher le tout dans `main.rs`. Fichiers : `error.rs`, `loader.rs`, `plantuml.rs`, `main.rs`. Branches : `feat/errors`, `feat/loader`, `feat/plantuml`.

Tâches :

- [x] `error.rs` : poser l'enum `AppError` validée ensemble. C'est la première PR, car A en a besoin.
- [x] `Cargo.toml` : `serde` (feature `derive`), `serde_json`, `thiserror`.
- [x] `loader.rs` : écrire les structs `#[derive(Deserialize)]` qui collent au format des JSON d'exemple.
- [x] `loader.rs` : `pub fn load_contacts(path: &Path) -> Result<Vec<Contact>, AppError>` avec `fs::read_to_string(path)?` puis `serde_json::from_str(&content)?`.
- [ ] `loader.rs` : `pub fn build_trie(contacts: Vec<Contact>) -> (Trie<String>, Vec<AppError>)` qui appelle `insert` pour chaque contact. Un numéro invalide (`EmptyNumber`, `InvalidDigit`) n'arrête pas la construction : le contact est ignoré et l'erreur est ajoutée à la liste renvoyée. La lib n'affiche rien. Pas de `Result` ici : les seules erreurs bloquantes (fichier introuvable, JSON mal formé) sont déjà renvoyées par `load_contacts`.
- [ ] `plantuml.rs` : `pub fn to_mindmap<T: Display>(trie: &Trie<T>) -> String`. On écrit `@startmindmap`, puis une ligne par nœud avec autant de `*` que la profondeur + 1, enfants triés par clé, et `@endmindmap`. Une fonction récursive `write_node(node, depth, &mut String)` suffit.
- [ ] `plantuml.rs` : afficher la valeur sur les nœuds où un numéro se termine, par exemple `**** 7 : Alice`.
- [x] `plantuml.rs` : `pub fn write_mindmap(content: &str, path: &Path) -> Result<(), AppError>`.
- [ ] `main.rs` minimal : prendre le chemin du JSON en argument (`std::env::args`), charger, construire, écrire dans `graph/<nom>.puml`. Afficher un avertissement par numéro rejeté renvoyé par `build_trie` (le programme continue), et afficher l'erreur proprement puis s'arrêter si la lecture du fichier ou du JSON échoue.
- [ ] Vérifier chaque `.puml` généré à la main dans le serveur PlantUML (Docker, `localhost:8080`).

Exemple de sortie attendue pour `0612` (Alice) et `0634` (Bob) :

```
@startmindmap
* Racine
** 0
*** 6
**** 1
***** 2 : Alice
**** 3
***** 4 : Bob
@endmindmap
```

Tests unitaires attendus :

- `load_contacts` sur un JSON valide renvoie le bon nombre d'éléments.
- JSON mal formé et fichier inexistant renvoient `AppError::Json` et `AppError::Io`.
- `build_trie` avec un numéro valide et un numéro invalide : le trie contient le valide, la liste d'erreurs contient exactement un `AppError::InvalidDigit`.
- `to_mindmap` sur un trie de deux numéros produit exactement la sortie ci-dessus. Tant que A n'a pas fini, on peut tester sur un trie construit à la main.
- Le trie vide produit un mindmap valide avec la racine seule.

C'est terminé quand `cargo run --release -- data/xxx.json` génère un `.puml` qui s'affiche correctement dans PlantUML, que les tests passent et que la PR est relue par A.

À savoir expliquer en soutenance : ce que génère `#[derive(Deserialize)]` (une macro procédurale qui implémente le trait à la compilation), le fonctionnement de `?` avec `#[from]`, et la récursion de l'export.

## Tâches communes et workflow git

Tâches à faire à deux :

- [x] Valider l'API et `AppError` avant de commencer à coder.
- [x] Jeux de données dans `data/` : un cas simple, des préfixes communs, un numéro invalide et un fichier volumineux (`01` à `06`, dont un JSON mal formé et un tableau vide).
- [x] README : description, prérequis, lancement (`cargo run --release -- data/xxx.json`), tests (`cargo test`) et visualisation PlantUML.
- [ ] Relire le code de l'autre : le prof peut interroger n'importe qui sur n'importe quelle partie.
- [ ] Préparer les 5 minutes de présentation de la soutenance.

Workflow :

1. Une branche par tâche à partir de `main` à jour, par exemple `feat/trie` ou `feat/plantuml`.
2. Des petits commits au format conventional commits : `feat: add trie insertion`, `test: cover shared prefixes`, `fix: reject non-digit chars`, `docs: add usage to README`, `refactor: extract write_node`.
3. Avant de pousser : `cargo fmt`, `cargo clippy` et `cargo test`.
4. Une PR vers `main`, relue et approuvée par l'autre avant le merge. Pas de push direct sur `main`.
5. Après chaque merge, chacun fait un `git pull` sur `main` puis un rebase de sa branche.
