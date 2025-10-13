#[cfg(test)]
mod cli_tests {
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    // Helper pour exécuter xsshend avec des arguments
    fn run_xsshend_with_args(args: &[&str]) -> std::process::Output {
        Command::new("./target/debug/xsshend")
            .args(args)
            .output()
            .expect("Failed to execute xsshend")
    }

    // Helper pour créer un fichier de test
    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_cli_help_command() {
        let output = run_xsshend_with_args(&["--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("xsshend"));
        assert!(stdout.contains("upload"));
        assert!(stdout.contains("list"));
        assert!(stdout.contains("init"));
    }

    #[test]
    fn test_cli_version_command() {
        let output = run_xsshend_with_args(&["--version"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("0.2.11"));
    }

    #[test]
    fn test_cli_upload_help() {
        let output = run_xsshend_with_args(&["upload", "--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Téléverse des fichiers"));
        assert!(stdout.contains("--env"));
        assert!(stdout.contains("--region"));
        assert!(stdout.contains("--type"));
        assert!(stdout.contains("--dest"));
        assert!(stdout.contains("--dry-run"));
        // Vérifier que --ssh-key n'est plus présent
        assert!(!stdout.contains("--ssh-key"));
    }

    #[test]
    fn test_cli_list_help() {
        let output = run_xsshend_with_args(&["list", "--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Liste les serveurs"));
    }

    #[test]
    fn test_cli_init_help() {
        let output = run_xsshend_with_args(&["init", "--help"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Initialise la configuration"));
        assert!(stdout.contains("--force"));
    }

    #[test]
    fn test_cli_no_arguments() {
        let output = run_xsshend_with_args(&[]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Utilisez 'xsshend --help'"));
        assert!(stdout.contains("Exemples:"));
    }

    #[test]
    fn test_cli_upload_missing_files() {
        let output = run_xsshend_with_args(&["upload"]);

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("required") || stderr.contains("FILE"));
    }

    #[test]
    fn test_cli_upload_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Mode dry-run"));
        assert!(stdout.contains("Simulation terminée"));
    }

    #[test]
    fn test_cli_upload_nonexistent_file() {
        let output = run_xsshend_with_args(&[
            "upload",
            "/nonexistent/file.txt",
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(!output.status.success());
        // Le programme devrait échouer car le fichier n'existe pas
    }

    #[test]
    fn test_cli_upload_with_filters() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Production",
            "--region",
            "Region-A",
            "--type",
            "Public",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Mode dry-run"));
    }

    #[test]
    fn test_cli_upload_custom_destination() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dest",
            "/custom/path/",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("/custom/path/"));
    }

    #[test]
    fn test_cli_upload_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = create_test_file(&temp_dir, "file1.txt", "content 1");
        let file2 = create_test_file(&temp_dir, "file2.txt", "content 2");

        let output = run_xsshend_with_args(&[
            "upload",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("file1.txt"));
        assert!(stdout.contains("file2.txt"));
    }

    #[test]
    fn test_cli_list_command() {
        let output = run_xsshend_with_args(&["list"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Liste des cibles SSH"));
        assert!(stdout.contains("Production") || stdout.contains("Development"));
    }

    #[test]
    fn test_cli_list_flag() {
        let output = run_xsshend_with_args(&["--list"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Liste des cibles SSH"));
    }

    #[test]
    fn test_cli_invalid_command() {
        let output = run_xsshend_with_args(&["invalid-command"]);

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("unrecognized subcommand") || stderr.contains("invalid"));
    }

    #[test]
    fn test_cli_upload_invalid_filter() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "NonExistentEnvironment",
            "--dry-run",
        ]);

        assert!(output.status.success()); // Le programme ne devrait pas crasher
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Aucun serveur trouvé") || stdout.contains("Mode dry-run"));
    }

    #[test]
    fn test_cli_empty_file_upload() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = create_test_file(&temp_dir, "empty.txt", "");

        let output = run_xsshend_with_args(&[
            "upload",
            empty_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("0 B")); // Fichier vide
    }

    #[test]
    fn test_cli_large_file_upload() {
        let temp_dir = TempDir::new().unwrap();
        let large_content = "x".repeat(1024 * 1024); // 1MB
        let large_file = create_test_file(&temp_dir, "large.txt", &large_content);

        let output = run_xsshend_with_args(&[
            "upload",
            large_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("1.0 MB")); // Fichier de 1MB
    }

    #[test]
    fn test_cli_special_characters_in_filename() {
        let temp_dir = TempDir::new().unwrap();
        let special_file =
            create_test_file(&temp_dir, "file with spaces & symbols!.txt", "content");

        let output = run_xsshend_with_args(&[
            "upload",
            special_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("file with spaces & symbols!.txt"));
    }

    // Tests pour vérifier que les anciennes options SSH key ont été supprimées
    #[test]
    fn test_cli_ssh_key_options_removed() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        // Test que --ssh-key n'est plus accepté
        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--ssh-key",
            "id_ed25519",
            "--dry-run",
        ]);

        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("unexpected argument") || stderr.contains("unrecognized"));
    }

    #[test]
    fn test_cli_destination_default() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt", "test content");

        let output = run_xsshend_with_args(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("/tmp/")); // Destination par défaut
    }
}
