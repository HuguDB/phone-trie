# phone-trie

Programme Rust qui lit un fichier JSON de contacts, range les numéros dans un
trie (arbre de préfixes) et exporte l'arbre en MindMap PlantUML. Les numéros
qui commencent pareil partagent les mêmes nœuds.

## Prérequis

- Rust stable, édition 2024 (installé avec [rustup](https://rustup.rs)).
- Docker pour visualiser les MindMaps et lancer les tests de rendu.

## Lancement

Depuis la racine du projet :

```
cargo run --release -- data/02_common_parts.json
```

Le MindMap est écrit dans `graph/<nom du fichier>.puml` (le dossier `graph/`
est relatif au répertoire courant et créé s'il manque) :

```
4 numéros stockés dans 28 nœuds (40 chiffres sans trie)
MindMap généré : graph/02_common_parts.puml
```

Un numéro invalide (vide, ou avec autre chose que des chiffres) est ignoré avec
un avertissement, et le reste du fichier est traité :

```
avertissement : contact « Eve » ignoré, caractère invalide '-' dans le numéro 06-12-34-56-78
```

Un argument manquant arrête le programme avec le code 2. Un fichier
introuvable, un JSON invalide ou une écriture impossible l'arrêtent avec le
code 1. Si un numéro apparaît deux fois, le dernier contact est gardé.

## Format JSON

Un tableau d'objets avec les champs `name` et `phone`, tous deux obligatoires.
Les numéros n'ont pas de préfixe international.

```json
[
  { "name": "Alice", "phone": "0612345678" },
  { "name": "Bob", "phone": "0478123456" }
]
```

Fichiers de test dans `data/` :

| Fichier | Contenu |
|---|---|
| `01_simple.json` | 2 contacts sans préfixe commun |
| `02_common_parts.json` | 4 contacts avec des préfixes communs |
| `03_invalid_number.json` | 1 numéro avec des tirets (ignoré) |
| `04_malformed.json` | JSON cassé (erreur) |
| `05_empty.json` | tableau vide |
| `06_large.json` | 50 contacts sur une dizaine de préfixes |

## Tests

```
cargo test
```

Lance les tests unitaires (dans chaque fichier de `src/`) et les tests du
binaire (`tests/cli.rs`).

```
cargo test -- --ignored
```

Vérifie avec Docker (image `plantuml/plantuml`) que chaque `.puml` généré est
valide et que son rendu PNG tient dans la limite de 4096 px de PlantUML. Ces
tests sont ignorés par défaut car ils ont besoin de Docker.

Le code passe aussi `cargo fmt --check` et `cargo clippy --all-targets` sans
warning.

## Visualisation PlantUML

Les `.puml` des fichiers de test sont déjà dans `graph/`. Exemple pour `0612`
(Alice) et `0634` (Bob) :

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

Lancer le serveur PlantUML, puis ouvrir <http://localhost:8080> et coller le
contenu d'un fichier `graph/<nom>.puml` :

```
docker run -d -p 8080:8080 plantuml/plantuml-server:jetty
```

Les noms sont échappés pour PlantUML : `Jean-Pierre` s'écrit `Jean~-Pierre`
dans le `.puml` mais s'affiche normalement.

PlantUML coupe les images PNG au-delà de 4096 px. Pour un gros fichier, utiliser
la sortie SVG ou lancer le serveur avec `-e PLANTUML_LIMIT_SIZE=16384`.

## Structure du projet

```
src/
├── main.rs       # lecture de l'argument et enchaînement des étapes
├── lib.rs        # #![forbid(unsafe_code)] et modules
├── error.rs      # AppError (thiserror)
├── trie.rs       # Trie<T> et Node<T>, sans crate
├── loader.rs     # lecture du JSON (serde) et construction du trie
└── plantuml.rs   # export MindMap et écriture du .puml
tests/
├── cli.rs              # tests du binaire
└── plantuml_render.rs  # tests de rendu PlantUML (Docker)
```

Pas de code `unsafe`, pas de `unwrap()` ni `expect()` hors des tests : les
erreurs remontent avec `Result` et `?`. Dépendances : `serde`, `serde_json` et
`thiserror`.
