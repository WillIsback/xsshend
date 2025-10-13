#[cfg(test)]
mod uploader_tests {
    use std::fs;
    use tempfile::TempDir;
    use xsshend::config::HostEntry;
    use xsshend::core::uploader::Uploader;

    fn create_test_file(content: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, content).unwrap();
        temp_dir
    }

    fn create_test_hosts() -> Vec<(String, HostEntry)> {
        vec![
            (
                "Production:Region-A:Public:WEB_SERVER_01".to_string(),
                HostEntry {
                    alias: "web01@prod-web-01.example.com".to_string(),
                    env: "PROD".to_string(),
                },
            ),
            (
                "Production:Region-A:Public:API_SERVER_01".to_string(),
                HostEntry {
                    alias: "api01@prod-api-01.example.com".to_string(),
                    env: "PROD".to_string(),
                },
            ),
        ]
    }

    #[test]
    fn test_uploader_creation() {
        let _uploader = Uploader::new();
        // Test que l'uploader peut être créé sans erreur
        // L'uploader est maintenant une structure vide, donc ce test vérifie juste la création
    }

    #[test]
    fn test_uploader_default() {
        let _uploader = Uploader;
        // Test que Default trait fonctionne
    }

    #[test]
    fn test_parse_server_alias_valid() {
        // Test avec un alias valide
        let (username, host) = Uploader::parse_server_alias("user@example.com").unwrap();
        assert_eq!(username, "user");
        assert_eq!(host, "example.com");

        // Test avec port
        let (username, host) = Uploader::parse_server_alias("admin@server.local:2222").unwrap();
        assert_eq!(username, "admin");
        assert_eq!(host, "server.local:2222");
    }

    #[test]
    fn test_parse_server_alias_invalid() {
        // Test avec alias invalide (pas de @)
        let result = Uploader::parse_server_alias("invalid-alias");
        assert!(result.is_err());

        // Test avec alias vide
        let result = Uploader::parse_server_alias("");
        assert!(result.is_err());

        // Test avec seulement @
        let result = Uploader::parse_server_alias("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_dry_run_basic() {
        let temp_dir = create_test_file("test content");
        let file_path = temp_dir.path().join("test_file.txt");
        let file_refs = vec![file_path.as_path()];

        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Le dry-run ne devrait pas échouer
        let result = Uploader::new().dry_run(&file_refs, &host_refs, "/tmp/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dry_run_with_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("nonexistent.txt");
        let file_refs = vec![invalid_file.as_path()];

        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Devrait échouer car le fichier n'existe pas
        let result = Uploader::new().dry_run(&file_refs, &host_refs, "/tmp/");
        assert!(result.is_err());
    }

    #[test]
    fn test_dry_run_with_empty_hosts() {
        let temp_dir = create_test_file("test content");
        let file_path = temp_dir.path().join("test_file.txt");
        let file_refs = vec![file_path.as_path()];

        let empty_hosts = vec![];

        let _uploader = Uploader::new();

        // Devrait fonctionner même avec une liste vide d'hôtes
        let result = Uploader::new().dry_run(&file_refs, &empty_hosts, "/tmp/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dry_run_with_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        // Créer plusieurs fichiers de test
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "content 1").unwrap();
        fs::write(&file2, "content 2").unwrap();

        let file_refs = vec![file1.as_path(), file2.as_path()];
        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        let result = Uploader::new().dry_run(&file_refs, &host_refs, "/tmp/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dry_run_different_destinations() {
        let temp_dir = create_test_file("test content");
        let file_path = temp_dir.path().join("test_file.txt");
        let file_refs = vec![file_path.as_path()];

        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Test avec différentes destinations
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/tmp/")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/home/user/")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/var/www/")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/opt/app/")
                .is_ok()
        );
    }

    #[test]
    fn test_upload_files_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Créer un fichier valide
        let valid_file = temp_dir.path().join("valid.txt");
        fs::write(&valid_file, "valid content").unwrap();

        // Créer une référence vers un fichier invalide
        let invalid_file = temp_dir.path().join("nonexistent.txt");

        let _valid_refs = [valid_file.as_path()];
        let invalid_refs = vec![invalid_file.as_path()];

        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Note: upload_files essaiera de se connecter réellement aux serveurs
        // Pour un vrai test unitaire, on devrait mocker la connexion SSH
        // Ici on teste seulement la validation des fichiers

        // Le test avec un fichier invalide devrait échouer pendant la validation
        let result = Uploader::new().upload_files(&invalid_refs, &host_refs, "/tmp/");
        assert!(result.is_err());
    }

    // Tests pour les méthodes privées via des tests indirects

    #[test]
    fn test_server_alias_parsing_edge_cases() {
        let _uploader = Uploader::new();

        // Test avec des caractères spéciaux
        let result = Uploader::parse_server_alias("user-name@server-name.example.com");
        assert!(result.is_ok());
        let (username, host) = result.unwrap();
        assert_eq!(username, "user-name");
        assert_eq!(host, "server-name.example.com");

        // Test avec des numéros
        let result = Uploader::parse_server_alias("user123@192.168.1.100");
        assert!(result.is_ok());
        let (username, host) = result.unwrap();
        assert_eq!(username, "user123");
        assert_eq!(host, "192.168.1.100");

        // Test avec sous-domaines
        let result = Uploader::parse_server_alias("admin@app.staging.example.com");
        assert!(result.is_ok());
        let (username, host) = result.unwrap();
        assert_eq!(username, "admin");
        assert_eq!(host, "app.staging.example.com");
    }

    #[test]
    fn test_destination_path_handling() {
        let temp_dir = create_test_file("test content");
        let file_path = temp_dir.path().join("test_file.txt");
        let file_refs = vec![file_path.as_path()];

        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Test avec différents formats de destination
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/tmp")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "/tmp/")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "~/uploads")
                .is_ok()
        );
        assert!(
            Uploader::new()
                .dry_run(&file_refs, &host_refs, "./files")
                .is_ok()
        );
    }

    #[test]
    fn test_large_host_list() {
        let temp_dir = create_test_file("test content");
        let file_path = temp_dir.path().join("test_file.txt");
        let file_refs = vec![file_path.as_path()];

        // Créer une grande liste d'hôtes
        let mut large_host_list = Vec::new();
        for i in 0..100 {
            large_host_list.push((
                format!("Server_{}", i),
                HostEntry {
                    alias: format!("user{}@server{}.example.com", i, i),
                    env: "TEST".to_string(),
                },
            ));
        }

        let host_refs: Vec<(String, &HostEntry)> = large_host_list
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Le dry-run devrait gérer une grande liste d'hôtes
        let result = Uploader::new().dry_run(&file_refs, &host_refs, "/tmp/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_size_calculation() {
        let temp_dir = TempDir::new().unwrap();

        // Créer des fichiers de différentes tailles
        let small_file = temp_dir.path().join("small.txt");
        let large_file = temp_dir.path().join("large.txt");

        fs::write(&small_file, "small").unwrap();
        fs::write(&large_file, "x".repeat(10000)).unwrap();

        // Vérifier que les fichiers existent et ont la bonne taille
        assert_eq!(fs::metadata(&small_file).unwrap().len(), 5);
        assert_eq!(fs::metadata(&large_file).unwrap().len(), 10000);

        let file_refs = vec![small_file.as_path(), large_file.as_path()];
        let hosts = create_test_hosts();
        let host_refs: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        let _uploader = Uploader::new();

        // Le dry-run devrait fonctionner avec des fichiers de différentes tailles
        let result = Uploader::new().dry_run(&file_refs, &host_refs, "/tmp/");
        assert!(result.is_ok());
    }
}
