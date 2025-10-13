#[cfg(test)]
mod validator_tests {
    use std::fs;
    use tempfile::TempDir;
    use xsshend::core::validator::Validator;

    #[test]
    fn test_validate_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = Validator::validate_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        let result = Validator::validate_file(&nonexistent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();

        let result = Validator::validate_file(&dir_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).unwrap();

        let size = Validator::get_file_size(&file_path).unwrap();
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_get_file_size_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        let result = Validator::get_file_size(&nonexistent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(Validator::format_file_size(0), "0 B");
        assert_eq!(Validator::format_file_size(512), "512 B");
        assert_eq!(Validator::format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_format_file_size_kilobytes() {
        assert_eq!(Validator::format_file_size(1024), "1.0 KB");
        assert_eq!(Validator::format_file_size(1536), "1.5 KB");
        assert_eq!(Validator::format_file_size(10240), "10.0 KB");
    }

    #[test]
    fn test_format_file_size_megabytes() {
        assert_eq!(Validator::format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(Validator::format_file_size(1024 * 1024 * 2), "2.0 MB");
        assert_eq!(
            Validator::format_file_size(1024 * 1024 + 512 * 1024),
            "1.5 MB"
        );
    }

    #[test]
    fn test_format_file_size_gigabytes() {
        assert_eq!(Validator::format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(
            Validator::format_file_size(1024 * 1024 * 1024 * 3),
            "3.0 GB"
        );
    }

    #[test]
    fn test_format_file_size_edge_cases() {
        assert_eq!(Validator::format_file_size(1), "1 B");
        assert_eq!(Validator::format_file_size(1025), "1.0 KB");
        assert_eq!(Validator::format_file_size(1048577), "1.0 MB");
    }

    #[test]
    fn test_validate_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.txt");
        fs::write(&empty_file, "").unwrap();

        let result = Validator::validate_file(&empty_file);
        assert!(result.is_ok());

        let size = Validator::get_file_size(&empty_file).unwrap();
        assert_eq!(size, 0);
        assert_eq!(Validator::format_file_size(size), "0 B");
    }

    #[test]
    fn test_validate_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.txt");

        // Créer un fichier de 1MB
        let content = "x".repeat(1024 * 1024);
        fs::write(&large_file, &content).unwrap();

        let result = Validator::validate_file(&large_file);
        assert!(result.is_ok());

        let size = Validator::get_file_size(&large_file).unwrap();
        assert_eq!(size, 1024 * 1024);
        assert_eq!(Validator::format_file_size(size), "1.0 MB");
    }

    #[test]
    fn test_validate_file_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let special_file = temp_dir.path().join("file with spaces & symbols!.txt");
        fs::write(&special_file, "content").unwrap();

        let result = Validator::validate_file(&special_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        // Sur Unix, on peut tester les permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o644); // Lecture/écriture pour le propriétaire, lecture pour les autres
            fs::set_permissions(&file_path, perms).unwrap();

            let result = Validator::validate_file(&file_path);
            assert!(result.is_ok());
        }

        // Sur Windows, le test s'assure juste que le fichier est valide
        #[cfg(windows)]
        {
            let result = Validator::validate_file(&file_path);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        fs::write(&file3, "content3").unwrap();

        // Valider chaque fichier
        assert!(Validator::validate_file(&file1).is_ok());
        assert!(Validator::validate_file(&file2).is_ok());
        assert!(Validator::validate_file(&file3).is_ok());

        // Vérifier les tailles
        assert_eq!(Validator::get_file_size(&file1).unwrap(), 8);
        assert_eq!(Validator::get_file_size(&file2).unwrap(), 8);
        assert_eq!(Validator::get_file_size(&file3).unwrap(), 8);
    }

    #[test]
    fn test_format_file_size_precision() {
        // Tester la précision des calculs
        assert_eq!(Validator::format_file_size(1536), "1.5 KB"); // 1.5 * 1024
        assert_eq!(Validator::format_file_size(2560), "2.5 KB"); // 2.5 * 1024
        assert_eq!(Validator::format_file_size(1572864), "1.5 MB"); // 1.5 * 1024 * 1024
    }
}
