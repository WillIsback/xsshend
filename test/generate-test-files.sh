#!/bin/bash

# Générateur de fichiers de test pour xsshend
# Crée différents types et tailles de fichiers pour tester les transferts

TEST_DATA_DIR="$(dirname "$0")/data"
mkdir -p "$TEST_DATA_DIR"

echo "🔧 Génération des fichiers de test..."

# Fichier texte simple
echo "Hello xsshend! Test file created at $(date)" > "$TEST_DATA_DIR/simple.txt"

# Fichier JSON de configuration
cat > "$TEST_DATA_DIR/config.json" << 'EOF'
{
  "app": "xsshend-test",
  "version": "0.1.0", 
  "timestamp": "2025-07-05T00:00:00Z",
  "settings": {
    "upload_batch_size": 10,
    "max_parallel_connections": 5,
    "timeout_seconds": 30
  },
  "test_data": {
    "lorem": "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
    "numbers": [1, 2, 3, 4, 5],
    "nested": {
      "deep": {
        "value": "test"
      }
    }
  }
}
EOF

# Script shell de test
cat > "$TEST_DATA_DIR/deploy.sh" << 'EOF'
#!/bin/bash
echo "🚀 Script de déploiement xsshend"
echo "Timestamp: $(date)"
echo "Hostname: $(hostname)"
echo "User: $(whoami)"
echo "Working directory: $(pwd)"

# Créer un fichier de vérification
echo "Déploiement réussi - $(date)" > /tmp/xsshend-deploy-success.txt
echo "✅ Déploiement terminé!"
EOF

chmod +x "$TEST_DATA_DIR/deploy.sh"

# Fichier HTML de test
cat > "$TEST_DATA_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>xsshend Test Page</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { color: #333; border-bottom: 2px solid #007acc; }
        .info { background: #f0f8ff; padding: 15px; border-radius: 5px; }
        .timestamp { color: #666; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1 class="header">🚀 xsshend Test Page</h1>
    <div class="info">
        <p><strong>Status:</strong> Test file successfully uploaded!</p>
        <p><strong>Tool:</strong> xsshend v0.1.0</p>
        <p><strong>Purpose:</strong> SSH/SFTP multi-server upload testing</p>
        <p class="timestamp">Generated: $(date)</p>
    </div>
    <script>
        console.log('xsshend test page loaded at', new Date());
    </script>
</body>
</html>
EOF

# Fichiers de différentes tailles
echo "📦 Création de fichiers de test de différentes tailles..."

# Petit fichier (1KB)
head -c 1024 /dev/urandom | base64 > "$TEST_DATA_DIR/small-1kb.txt"

# Fichier moyen (100KB) 
head -c 102400 /dev/urandom | base64 > "$TEST_DATA_DIR/medium-100kb.txt"

# Gros fichier (1MB)
head -c 1048576 /dev/urandom | base64 > "$TEST_DATA_DIR/large-1mb.txt"

# Fichier CSV de test
cat > "$TEST_DATA_DIR/test-data.csv" << 'EOF'
id,name,environment,ip,status,last_update
1,xsshend-dev,Development,192.168.1.10,active,2025-07-05T10:00:00Z
2,xsshend-staging,Staging,192.168.1.20,active,2025-07-05T10:00:00Z
3,xsshend-prod-web,Production,192.168.1.30,active,2025-07-05T10:00:00Z
4,xsshend-prod-api,Production,192.168.1.31,active,2025-07-05T10:00:00Z
5,xsshend-prod-db,Production,192.168.1.32,maintenance,2025-07-05T09:30:00Z
EOF

# Archive tar.gz de test
cd "$TEST_DATA_DIR"
tar -czf archive-test.tar.gz *.txt *.json *.html *.csv *.sh
cd - > /dev/null

# Résumé
echo ""
echo "✅ Fichiers de test générés dans $TEST_DATA_DIR:"
ls -lah "$TEST_DATA_DIR"
echo ""
echo "📊 Résumé des fichiers:"
echo "  • simple.txt       - Fichier texte basique"
echo "  • config.json      - Configuration JSON" 
echo "  • deploy.sh        - Script de déploiement"
echo "  • index.html       - Page web de test"
echo "  • small-1kb.txt    - Fichier 1KB (test rapide)"
echo "  • medium-100kb.txt - Fichier 100KB (test progression)"
echo "  • large-1mb.txt    - Fichier 1MB (test gros volume)"
echo "  • test-data.csv    - Données CSV"
echo "  • archive-test.tar.gz - Archive compressée"
echo ""
echo "🚀 Prêt pour les tests xsshend!"
