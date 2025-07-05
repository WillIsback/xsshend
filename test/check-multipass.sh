#!/bin/bash

# Script de diagnostic et correction pour Multipass
# Aide à résoudre les problèmes de PATH avec les applications snap

set -euo pipefail

echo "🔍 Diagnostic Multipass pour xsshend"
echo "======================================"
echo ""

# Vérification de l'installation
echo "1. Vérification de l'installation Multipass..."
if snap list | grep -q multipass; then
    echo "✅ Multipass installé via snap"
    snap list | grep multipass
else
    echo "❌ Multipass non trouvé dans snap"
    echo "Installation: sudo snap install multipass --classic"
    exit 1
fi

echo ""

# Vérification du PATH
echo "2. Vérification du PATH..."
echo "PATH actuel: $PATH"

if echo "$PATH" | grep -q "/snap/bin"; then
    echo "✅ /snap/bin présent dans le PATH"
else
    echo "⚠️  /snap/bin absent du PATH"
fi

echo ""

# Test de la commande multipass
echo "3. Test de la commande multipass..."

if command -v multipass &> /dev/null; then
    echo "✅ multipass accessible directement"
    multipass version
elif [[ -f "/snap/bin/multipass" ]]; then
    echo "⚠️  multipass trouvé dans /snap/bin mais pas dans PATH"
    echo "Version: $(/snap/bin/multipass version)"
    
    echo ""
    echo "💡 SOLUTION RECOMMANDÉE:"
    echo "Ajoutez cette ligne à votre ~/.bashrc ou ~/.zshrc :"
    echo "export PATH=\"/snap/bin:\$PATH\""
    echo ""
    echo "Puis rechargez votre shell:"
    echo "source ~/.bashrc  # ou ~/.zshrc"
    echo ""
    echo "Ou pour cette session uniquement:"
    echo "export PATH=\"/snap/bin:\$PATH\""
    
    # Application automatique pour cette session
    export PATH="/snap/bin:$PATH"
    echo ""
    echo "✅ PATH mis à jour pour cette session"
else
    echo "❌ multipass introuvable"
    exit 1
fi

echo ""

# Test de fonctionnement
echo "4. Test de fonctionnement..."
if multipass version &> /dev/null; then
    echo "✅ Multipass fonctionne correctement"
    multipass version
    echo ""
    if multipass list &> /dev/null; then
        echo "✅ Commande 'multipass list' fonctionne"
        multipass list
    else
        echo "⚠️  Problème avec 'multipass list' - permissions?"
    fi
else
    echo "❌ Multipass ne fonctionne pas"
    echo "Essayez: sudo snap refresh multipass"
    exit 1
fi

echo ""
echo "🚀 Diagnostic terminé!"
echo ""
echo "Si tout est ✅, vous pouvez maintenant exécuter:"
echo "  ./demo.sh"
echo "  ./test-vms.sh help"
