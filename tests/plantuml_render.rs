//! Vérification du rendu PlantUML des jeux de test, avec l'image Docker
//! `plantuml/plantuml`.
//!
//! Ces tests sont marqués `#[ignore]` : ils dépendent de Docker et du réseau
//! (téléchargement de l'image), ce qui contredit le principe « Repeatable »
//! de F.I.R.S.T. Ils ne tournent donc pas avec `cargo test` ; on les lance
//! explicitement avec `cargo test -- --ignored`.
//!
//! Pour chaque fichier valide de `data/`, le test :
//! 1. génère le `.puml` avec le binaire, dans un dossier temporaire ;
//! 2. valide sa syntaxe (`-checkonly`) ;
//! 3. rend le PNG avec les réglages par défaut et vérifie qu'il n'atteint pas
//!    la limite de 4096 px, au-delà de laquelle PlantUML coupe l'image.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Taille maximale d'un PNG rendu par PlantUML sans `PLANTUML_LIMIT_SIZE`.
const PLANTUML_LIMIT: u32 = 4096;

/// Dossier temporaire vide, propre à un test et à ce processus de test.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test_name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "phone_trie_render_{test_name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Échoue avec un message clair si Docker n'est pas utilisable.
fn require_docker() {
    let running = Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(running, "Docker requis : lancez Docker Desktop");
}

/// Lance `plantuml/plantuml` sur `dir` (monté dans `/data`) avec `args`.
fn plantuml(dir: &Path, args: &[&str]) -> Output {
    let volume = format!("{}:/data", dir.display());
    Command::new("docker")
        .args(["run", "--rm", "-v", &volume, "plantuml/plantuml"])
        .args(args)
        .output()
        .unwrap()
}

/// Largeur et hauteur lues dans l'en-tête IHDR d'un PNG : signature sur
/// 8 octets, longueur du bloc sur 4, type `IHDR` sur 4, puis largeur
/// (octets 16 à 19) et hauteur (20 à 23) en big-endian.
fn png_size(png: &[u8]) -> (u32, u32) {
    assert!(png.len() >= 24, "PNG trop court");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "signature PNG invalide");
    assert_eq!(&png[12..16], b"IHDR", "bloc IHDR absent");
    let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
    let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
    (width, height)
}

/// Génère le `.puml` de `data/<stem>.json`, valide sa syntaxe et vérifie que
/// son rendu PNG tient dans la limite par défaut de PlantUML.
fn check_render(stem: &str) {
    require_docker();
    let dir = TempDir::new(stem);
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(format!("{stem}.json"));

    // 1. Génération du .puml avec le binaire, dans le dossier temporaire.
    let generated = Command::new(env!("CARGO_BIN_EXE_phone-trie"))
        .arg(&input)
        .current_dir(&dir.0)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "génération échouée : {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let graph = dir.0.join("graph");
    let puml = format!("/data/{stem}.puml");

    // 2. Validation de la syntaxe.
    let check = plantuml(&graph, &["-checkonly", "-failfast2", &puml]);
    assert!(
        check.status.success(),
        "syntaxe PlantUML invalide pour {stem}.puml :\n{}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    // 3. Rendu PNG avec les réglages par défaut.
    let render = plantuml(&graph, &["-tpng", "-failfast2", &puml]);
    assert!(
        render.status.success(),
        "rendu PNG échoué pour {stem}.puml :\n{}",
        String::from_utf8_lossy(&render.stderr)
    );
    let png = fs::read(graph.join(format!("{stem}.png"))).unwrap();
    let (width, height) = png_size(&png);
    assert!(
        height < PLANTUML_LIMIT && width < PLANTUML_LIMIT,
        "{stem}.png fait {width}×{height} px : PlantUML a coupé l'image \
         (limite de {PLANTUML_LIMIT} px)"
    );
}

#[test]
fn png_size_reads_ihdr_header() {
    // En-tête d'un PNG de 300×2834 px (seuls les 24 premiers octets comptent).
    let mut header = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    header.extend_from_slice(&300u32.to_be_bytes());
    header.extend_from_slice(&2834u32.to_be_bytes());
    assert_eq!(png_size(&header), (300, 2834));
}

#[test]
#[ignore = "nécessite Docker (plantuml/plantuml)"]
fn render_01_simple() {
    check_render("01_simple");
}

#[test]
#[ignore = "nécessite Docker (plantuml/plantuml)"]
fn render_02_common_parts() {
    check_render("02_common_parts");
}

#[test]
#[ignore = "nécessite Docker (plantuml/plantuml)"]
fn render_03_invalid_number() {
    check_render("03_invalid_number");
}

#[test]
#[ignore = "nécessite Docker (plantuml/plantuml)"]
fn render_05_empty() {
    check_render("05_empty");
}

#[test]
#[ignore = "nécessite Docker (plantuml/plantuml)"]
fn render_06_large() {
    check_render("06_large");
}
