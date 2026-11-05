//! Tests du binaire : codes de sortie, messages et fichier généré. Chaque test
//! tourne dans son propre dossier temporaire.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Dossier temporaire supprimé à la fin du test.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test_name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("phone_trie_cli_{test_name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Chemin d'un fichier de `data/`.
fn data_path(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(file)
}

/// Lance le binaire depuis `dir`.
fn run_in(dir: &Path, args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_phone-trie"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn no_argument_exits_with_2() {
    let dir = TempDir::new("no_arg");

    let output = run_in(dir.path(), &[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("usage"));
}

#[test]
fn common_parts_file_generates_mindmap() {
    let dir = TempDir::new("common_parts");

    let output = run_in(dir.path(), &[&data_path("02_common_parts.json")]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr : {}",
        stderr(&output)
    );
    let puml = dir.path().join("graph").join("02_common_parts.puml");
    let content = fs::read_to_string(&puml).unwrap();
    assert!(content.starts_with("@startmindmap\n* Racine\n"));
    assert!(content.contains(" : Alice\n"));
    assert!(content.ends_with("@endmindmap\n"));
}

#[test]
fn success_prints_statistics() {
    let dir = TempDir::new("statistics");

    let output = run_in(dir.path(), &[&data_path("02_common_parts.json")]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("4 numéros stockés dans 28 nœuds (40 chiffres sans trie)"),
        "stdout : {stdout}"
    );
}

#[test]
fn invalid_number_prints_warning_and_succeeds() {
    let dir = TempDir::new("invalid_number");

    let output = run_in(dir.path(), &[&data_path("03_invalid_number.json")]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr : {}",
        stderr(&output)
    );
    let err = stderr(&output);
    assert!(err.contains("avertissement"), "stderr : {err}");
    assert!(err.contains("06-12-34-56-78"), "stderr : {err}");
    assert!(err.contains("Eve"), "stderr : {err}");
    assert!(
        dir.path()
            .join("graph")
            .join("03_invalid_number.puml")
            .exists()
    );
}

#[test]
fn malformed_json_exits_with_1() {
    let dir = TempDir::new("malformed");

    let output = run_in(dir.path(), &[&data_path("04_malformed.json")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("JSON invalide"));
    assert!(!dir.path().join("graph").join("04_malformed.puml").exists());
}

#[test]
fn missing_file_exits_with_1() {
    let dir = TempDir::new("missing_file");

    let output = run_in(dir.path(), &[&data_path("inexistant.json")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("erreur"));
}

#[test]
fn missing_graph_directory_is_created() {
    let dir = TempDir::new("creates_graph");
    assert!(!dir.path().join("graph").exists());

    let output = run_in(dir.path(), &[&data_path("01_simple.json")]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr : {}",
        stderr(&output)
    );
    assert!(dir.path().join("graph").join("01_simple.puml").is_file());
}
