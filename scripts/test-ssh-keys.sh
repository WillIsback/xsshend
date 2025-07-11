#!/bin/bash

# Script de test pour les nouvelles fonctionnalités de sélection de clés SSH

set -e

echo "🔑 Test des fonctionnalités de sélection de clés SSH xsshend"
echo "============================================================"

# Créer un fichier de test
echo "test content $(date)" > test_file.txt

echo ""
echo "1️⃣ Test de la sélection automatique (--ssh-key-auto)"
echo "------------------------------------------------------"
./target/release/xsshend upload test_file.txt --ssh-key-auto --dry-run

echo ""
echo "2️⃣ Test de la spécification par nom (--ssh-key)"
echo "------------------------------------------------"
echo "Liste des clés disponibles pour test :"

# Découvrir les clés disponibles
./target/release/xsshend upload test_file.txt --ssh-key-interactive --dry-run 2>/dev/null | grep "❯\|  " | head -5 || true

echo ""
echo "Test avec une clé spécifique (première clé trouvée) :"
KEY_NAME=$(ls ~/.ssh/ | grep -E '^id_' | grep -v '.pub' | head -1)
if [ ! -z "$KEY_NAME" ]; then
    echo "Utilisation de la clé: $KEY_NAME"
    ./target/release/xsshend upload test_file.txt --ssh-key "$KEY_NAME" --dry-run
else
    echo "Aucune clé standard trouvée dans ~/.ssh/"
fi

echo ""
echo "3️⃣ Test du comportement par défaut (sélection intelligente)"
echo "-----------------------------------------------------------"
./target/release/xsshend upload test_file.txt --dry-run

echo ""
echo "✅ Tests terminés !"
echo ""
echo "💡 Utilisation pratique :"
echo "  • Ajoutez --ssh-key-interactive pour un menu de sélection"
echo "  • Ajoutez --ssh-key <nom> pour spécifier une clé directement"
echo "  • Ajoutez --ssh-key-auto pour forcer la sélection de la meilleure clé"
echo "  • Mode par défaut : sélection automatique intelligente"

# Nettoyer
rm -f test_file.txt
