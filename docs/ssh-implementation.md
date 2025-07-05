# Guide d'implémentation SSH/SFTP

## Architecture

L'implémentation SSH/SFTP de xsshend est construite autour de trois modules principaux :

- **`ssh::client`** : Client SSH/SFTP bas niveau avec ssh2-rs
- **`ssh::transfer`** : Gestionnaire de transfert avec barres de progression  
- **`core::uploader`** : Orchestrateur principal pour les téléversements

## Client SSH (`ssh::client::SshClient`)

### Fonctionnalités
- Connexion TCP avec parsing automatique host:port
- Authentification par clé SSH (id_rsa, id_ed25519, id_ecdsa)
- Fallback vers l'agent SSH si les clés échouent
- Transfert SFTP avec callbacks de progression
- Gestion propre des connexions (auto-disconnect)

### Utilisation
```rust
let mut client = SshClient::new("example.com:22", "username")?;
client.connect()?;
let bytes = client.upload_file_with_progress(
    Path::new("/local/file.txt"),
    "/remote/file.txt",
    |written, total| println!("Progress: {}/{}", written, total)
)?;
```

## Gestionnaire de transfert (`ssh::transfer::FileTransfer`)

### Fonctionnalités
- Barres de progression individuelles avec indicatif
- Transfert parallèle vers plusieurs serveurs avec rayon
- Parsing intelligent des alias SSH (user@host:port)
- Affichage de résumés détaillés avec statistiques
- Formatage lisible des tailles de fichiers

### Architecture parallèle
```rust
let results = transfer.upload_parallel(file, remote_path, hosts)?;
// Chaque host est traité en parallèle avec sa propre barre de progression
```

## Orchestrateur (`core::uploader::Uploader`)

### Modes d'opération
1. **Direct** : Transfert immédiat sans confirmation
2. **Interactif** : Demande confirmation avec prompt
3. **Dry-run** : Simulation sans transfert réel

### Validation
- Vérification de l'existence des fichiers
- Calcul des tailles pour les barres de progression
- Validation des chemins de destination

## Gestion d'erreurs

### Niveaux d'erreurs
- **Connexion** : TCP/SSH handshake, authentification
- **Transfert** : SFTP, permissions, espace disque
- **Fichiers** : Lecture locale, écriture distante

### Récupération
- Échecs partiels tolérés (certains serveurs peuvent échouer)
- Rapports détaillés des succès/échecs par serveur
- Nettoyage automatique des connexions

## Optimisations

### Performance
- **Chunks de 64KB** pour le transfert SFTP
- **Rayon** pour la parallélisation automatique
- **Réutilisation** des connexions SSH pour plusieurs fichiers

### UX
- **Barres de progression** individuelles par serveur
- **Troncature intelligente** des noms d'hôtes longs
- **Formatage lisible** des tailles et vitesses

## Tests

### Tests unitaires
```bash
cargo test ssh::        # Tests des modules SSH
cargo test transfer::   # Tests de transfert
cargo test uploader::   # Tests d'orchestration
```

### Tests d'intégration
```bash
./target/debug/xsshend upload file.txt --dry-run  # Simulation
./target/debug/xsshend upload file.txt --env Dev  # Test réel
```

## Debugging

### Variables d'environnement
```bash
RUST_LOG=debug ./target/debug/xsshend upload ...
```

### Modes verbeux
```bash
./target/debug/xsshend upload file.txt -v  # Mode verbeux
```

## Extensibilité

### Authentification supplémentaire
- Support password avec prompt sécurisé
- Support des certificats SSH
- Intégration Kerberos/GSSAPI

### Protocoles additionnels
- SCP en plus de SFTP
- Rsync over SSH
- HTTP/S upload

### Fonctionnalités avancées
- Reprise de transfert interrompu
- Checksums pour vérification d'intégrité
- Compression à la volée
