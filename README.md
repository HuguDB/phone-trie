# phone-trie

Gestionnaire de numéros de téléphone basé sur un trie (arbre de préfixes), écrit en Rust.

Le programme lit un fichier JSON de contacts, insère chaque numéro dans un trie
construit en mémoire, puis exporte l'arbre au format PlantUML MindMap pour le
visualiser. Les numéros qui partagent un préfixe (par exemple `0612…`) partagent
les mêmes nœuds, ce qui permet de voir le gain de place apporté par la structure.

## Prérequis

- Rust stable (édition 2024), installé avec [rustup](https://rustup.rs).
- Pour visualiser les graphes : Docker, ou Java avec `plantuml.jar`.

## Lancement

```
cargo run --release -- data/02_common_parts.json
```

Le programme écrit le MindMap dans `graph/<nom du fichier sans extension>.puml`,
ici `graph/02_common_parts.puml`, et affiche le chemin du fichier généré. Le
dossier `graph/` est relatif au répertoire courant : lancé depuis la racine du
projet, le programme écrit dans le `graph/` du dépôt. S'il n'existe pas, il est
créé.

Il affiche aussi une ligne de statistiques qui montre le gain apporté par le
partage des préfixes :

```
4 numéros stockés dans 28 nœuds (40 chiffres sans trie)
MindMap généré : graph/02_common_parts.puml
```

Sur `06_large.json`, 200 numéros de 10 chiffres tiennent dans 1084 nœuds au
lieu de 2000 chiffres.

Comportement en cas de problème :

- **Numéro invalide** (vide, ou contenant autre chose que des chiffres) : le
  contact est ignoré, un avertissement est affiché sur la sortie d'erreur, et
  le MindMap est généré avec les autres numéros.

  ```
  avertissement : numéro ignoré, caractère invalide '-' dans le numéro 06-12-34-56-78
  MindMap généré : graph/03_invalid_number.puml
  ```

- **Erreur bloquante** (argument manquant, fichier introuvable, JSON invalide,
  écriture du `.puml` impossible) : un message est affiché sur la sortie
  d'erreur et le programme s'arrête avec un code de sortie non nul (2 pour un
  argument manquant, 1 sinon).

  ```
  erreur : JSON invalide : EOF while parsing an object at line 3 column 0
  ```

Si un même numéro apparaît plusieurs fois, c'est le dernier contact qui est
gardé.

## Format d'entrée

Un tableau JSON d'objets avec deux champs texte, `name` (le nom du contact,
affiché sur le MindMap) et `phone` (le numéro). Les deux champs sont
obligatoires ; un champ manquant ou un JSON mal formé est une erreur bloquante.

```json
[
  { "name": "Alice", "phone": "0612345678" },
  { "name": "Bob", "phone": "0478123456" }
]
```

Les numéros sont écrits sans préfixe international et ne contiennent que des
chiffres. Le dossier `data/` fournit des jeux de test :

| Fichier | Contenu |
|---|---|
| `01_simple.json` | 2 contacts sans préfixe commun |
| `02_common_parts.json` | 4 contacts avec préfixes partagés |
| `03_invalid_number.json` | 1 numéro avec des tirets, rejeté à l'insertion |
| `04_malformed.json` | JSON volontairement cassé, rejeté à la lecture |
| `05_empty.json` | tableau vide |
| `06_large.json` | 200 contacts sur une dizaine de préfixes |

## Tests

```
cargo test
```

Les tests unitaires sont dans chaque fichier source (`#[cfg(test)] mod tests`).
Avant chaque commit, le projet doit aussi passer :

```
cargo fmt --check
cargo clippy --all-targets
```

## Visualisation PlantUML

Le dossier `graph/` contient déjà les `.puml` générés à partir des jeux de test.
Chaque chiffre est un nœud, et le nom du contact s'affiche sur le dernier
chiffre de son numéro :

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

Avec le serveur PlantUML en Docker :

```
docker run -d -p 8080:8080 plantuml/plantuml-server:jetty
```

Ouvrir ensuite <http://localhost:8080> et coller le contenu du fichier
`graph/<nom>.puml` dans la zone de texte.

Avec `plantuml.jar` en local :

```
java -jar plantuml.jar graph/02_common_parts.puml
```

Cela génère une image `graph/02_common_parts.png`.

## Organisation du code

```
src/
├── main.rs       # point d'entrée : lit les arguments et enchaîne les étapes
├── lib.rs        # #![forbid(unsafe_code)] et déclaration des modules
├── error.rs      # AppError, l'enum d'erreurs commune (thiserror)
├── trie.rs       # Trie<T> et Node<T>, codés from scratch
├── loader.rs     # lecture du JSON (serde) et construction du trie
└── plantuml.rs   # export MindMap et écriture du fichier .puml
```

Règles du projet : aucun code `unsafe`, aucun `unwrap()` ni `expect()` hors
des tests, erreurs propagées avec `Result` et l'opérateur `?`. Les seules
dépendances sont `serde`, `serde_json` et `thiserror` ; le trie n'utilise
aucun crate.
