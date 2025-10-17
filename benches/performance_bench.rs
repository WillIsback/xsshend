// Benchmarks de performance pour xsshend
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use tempfile::TempDir;
use xsshend::config::{HostEntry, HostsConfig};
use xsshend::core::validator::Validator;
use xsshend::ssh::keys::SshKey;

fn benchmark_config_parsing(c: &mut Criterion) {
    // Créer une grande configuration JSON
    let large_config = create_large_config_json(1000); // 1000 serveurs

    c.bench_function("config_parsing_large", |b| {
        b.iter(|| {
            let _config: HostsConfig = serde_json::from_str(black_box(&large_config)).unwrap();
        })
    });
}

fn benchmark_config_filtering(c: &mut Criterion) {
    let config = create_large_config(100); // 100 serveurs

    c.bench_function("config_filtering_env", |b| {
        b.iter(|| {
            let _filtered =
                config.filter_hosts(Some(&black_box("Production".to_string())), None, None);
        })
    });

    c.bench_function("config_filtering_combined", |b| {
        b.iter(|| {
            let _filtered = config.filter_hosts(
                Some(&black_box("Production".to_string())),
                Some(&black_box("Region-A".to_string())),
                Some(&black_box("Public".to_string())),
            );
        })
    });
}

fn benchmark_file_validation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    // Créer des fichiers de différentes tailles
    let small_file = temp_dir.path().join("small.txt");
    let medium_file = temp_dir.path().join("medium.txt");
    let large_file = temp_dir.path().join("large.txt");

    fs::write(&small_file, "small content").unwrap();
    fs::write(&medium_file, "x".repeat(1024 * 10)).unwrap(); // 10KB
    fs::write(&large_file, "x".repeat(1024 * 1024)).unwrap(); // 1MB

    c.bench_function("file_validation_small", |b| {
        b.iter(|| {
            Validator::validate_file(black_box(&small_file)).unwrap();
        })
    });

    c.bench_function("file_validation_medium", |b| {
        b.iter(|| {
            Validator::validate_file(black_box(&medium_file)).unwrap();
        })
    });

    c.bench_function("file_validation_large", |b| {
        b.iter(|| {
            Validator::validate_file(black_box(&large_file)).unwrap();
        })
    });
}

fn benchmark_file_size_formatting(c: &mut Criterion) {
    let sizes = vec![0u64, 1024, 1024 * 1024, 1024 * 1024 * 1024, 1024u64.pow(4)];

    c.bench_function("file_size_formatting", |b| {
        b.iter(|| {
            for &size in &sizes {
                black_box(Validator::format_file_size(black_box(size)));
            }
        })
    });
}

fn benchmark_ssh_key_detection(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).unwrap();

    // Créer plusieurs clés de test
    for i in 0..10 {
        let key_content = format!(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nkey{}\n-----END OPENSSH PRIVATE KEY-----",
            i
        );
        let pub_content = format!("ssh-ed25519 AAAAC3... test{}@example.com", i);

        fs::write(ssh_dir.join(format!("id_ed25519_{}", i)), key_content).unwrap();
        fs::write(ssh_dir.join(format!("id_ed25519_{}.pub", i)), pub_content).unwrap();
    }

    c.bench_function("ssh_key_discovery", |b| {
        b.iter(|| {
            // Simuler la découverte de clés
            for i in 0..10 {
                let key_path = ssh_dir.join(format!("id_ed25519_{}", i));
                if key_path.exists() {
                    let _key = SshKey::new(format!("id_ed25519_{}", i), key_path).unwrap();
                }
            }
        })
    });
}

fn benchmark_ssh_key_selection(c: &mut Criterion) {
    // Créer une liste de clés simulées
    let temp_dir = TempDir::new().unwrap();
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).unwrap();

    let mut keys = Vec::new();
    for i in 0..100 {
        let key_path = ssh_dir.join(format!("key_{}", i));
        fs::write(
            &key_path,
            "-----BEGIN OPENSSH PRIVATE KEY-----\n-----END OPENSSH PRIVATE KEY-----",
        )
        .unwrap();
        let key = SshKey::new(format!("key_{}", i), key_path).unwrap();
        keys.push(key);
    }

    c.bench_function("ssh_key_selection", |b| {
        b.iter(|| {
            // Simuler la sélection de la meilleure clé
            let mut best_key = &keys[0];
            for key in black_box(&keys) {
                if let xsshend::ssh::keys::SshKeyType::Ed25519 = key.key_type {
                    best_key = key;
                    break;
                }
            }
            black_box(best_key);
        })
    });
}

fn benchmark_server_alias_parsing(c: &mut Criterion) {
    let aliases = vec![
        "user@example.com",
        "admin@server.local",
        "deploy@app.staging.company.com",
        "root@192.168.1.100",
        "service@very-long-hostname.subdomain.example.org",
    ];

    c.bench_function("server_alias_parsing", |b| {
        b.iter(|| {
            for alias in black_box(&aliases) {
                if let Some(at_pos) = alias.find('@') {
                    let _username = &alias[..at_pos];
                    let _host = &alias[at_pos + 1..];
                }
            }
        })
    });
}

// Helpers pour créer des données de test

fn create_large_config_json(server_count: usize) -> String {
    let mut json = String::from("{\n");

    for env_i in 0..3 {
        json.push_str(&format!("  \"Env_{}\": {{\n", env_i));
        for region_i in 0..2 {
            json.push_str(&format!("    \"Region_{}\": {{\n", region_i));
            for type_i in 0..2 {
                json.push_str(&format!("      \"Type_{}\": {{\n", type_i));

                let servers_per_section = server_count / 12; // 3 env * 2 region * 2 type = 12
                for server_i in 0..servers_per_section {
                    json.push_str(&format!(
                        "        \"SERVER_{}_{}_{}_{}\": {{ \"alias\": \"user{}@server{}.com\", \"env\": \"ENV{}\" }}",
                        env_i, region_i, type_i, server_i, server_i, server_i, env_i
                    ));
                    if server_i < servers_per_section - 1 {
                        json.push(',');
                    }
                    json.push('\n');
                }

                json.push_str("      }");
                if type_i < 1 {
                    json.push(',');
                }
                json.push('\n');
            }
            json.push_str("    }");
            if region_i < 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  }");
        if env_i < 2 {
            json.push(',');
        }
        json.push('\n');
    }

    json.push_str("}\n");
    json
}

fn create_large_config(server_count: usize) -> HostsConfig {
    let mut environments = std::collections::HashMap::new();

    for env_i in 0..3 {
        let mut regions = std::collections::HashMap::new();
        for region_i in 0..2 {
            let mut types = std::collections::HashMap::new();
            for type_i in 0..2 {
                let mut servers = std::collections::HashMap::new();

                let servers_per_section = server_count / 12;
                for server_i in 0..servers_per_section {
                    servers.insert(
                        format!("SERVER_{}_{}_{}_{}", env_i, region_i, type_i, server_i),
                        HostEntry {
                            alias: format!("user{}@server{}.com", server_i, server_i),
                            env: format!("ENV{}", env_i),
                        },
                    );
                }

                types.insert(format!("Type_{}", type_i), servers);
            }
            regions.insert(format!("Region_{}", region_i), types);
        }
        environments.insert(format!("Env_{}", env_i), regions);
    }

    HostsConfig { environments }
}

criterion_group!(
    benches,
    benchmark_config_parsing,
    benchmark_config_filtering,
    benchmark_file_validation,
    benchmark_file_size_formatting,
    benchmark_ssh_key_detection,
    benchmark_ssh_key_selection,
    benchmark_server_alias_parsing
);

criterion_main!(benches);
