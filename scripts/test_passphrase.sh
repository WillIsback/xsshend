#!/bin/bash

# Script de test pour vérifier la gestion des passphrases SSH
# Génère une clé SSH protégée par passphrase et teste xsshend

set -e

# Couleurs pour les messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Test de la gestion des passphrases SSH dans xsshend${NC}"

# Créer un répertoire temporaire pour les tests
TEST_DIR="/tmp/xsshend_passphrase_test"
mkdir -p "$TEST_DIR"

# Variables
KEY_NAME="test_passphrase_key"
KEY_PATH="$TEST_DIR/$KEY_NAME"
PASSPHRASE="test123"

echo -e "${YELLOW}📁 Répertoire de test: $TEST_DIR${NC}"

# Nettoyer les clés existantes si elles existent
if [ -f "$KEY_PATH" ]; then
    echo -e "${YELLOW}🧹 Suppression des clés de test existantes${NC}"
    rm -f "$KEY_PATH" "$KEY_PATH.pub"
fi

# Générer une clé SSH protégée par passphrase
echo -e "${BLUE}🔑 Génération d'une clé SSH Ed25519 protégée par passphrase...${NC}"
ssh-keygen -t ed25519 -f "$KEY_PATH" -N "$PASSPHRASE" -C "test-key-for-passphrase" <<< ""

# Vérifier que la clé a été créée
if [ ! -f "$KEY_PATH" ] || [ ! -f "$KEY_PATH.pub" ]; then
    echo -e "${RED}❌ Erreur: Les clés SSH n'ont pas été créées correctement${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Clé SSH créée avec passphrase '$PASSPHRASE'${NC}"

# Copier la clé dans ~/.ssh pour que xsshend la trouve
echo -e "${BLUE}📋 Copie de la clé de test dans ~/.ssh/...${NC}"
cp "$KEY_PATH" ~/.ssh/test_passphrase_key
cp "$KEY_PATH.pub" ~/.ssh/test_passphrase_key.pub

# Afficher des informations sur la clé
echo -e "${BLUE}📊 Informations sur la clé créée:${NC}"
echo "Chemin privé: ~/.ssh/test_passphrase_key"
echo "Chemin public: ~/.ssh/test_passphrase_key.pub"
echo "Passphrase: $PASSPHRASE"

# Vérifier que la clé nécessite bien une passphrase
echo -e "${BLUE}🔍 Vérification que la clé nécessite une passphrase...${NC}"
if ssh-keygen -y -f "$KEY_PATH" >/dev/null 2>&1; then
    echo -e "${RED}❌ Erreur: La clé ne semble pas protégée par passphrase${NC}"
    exit 1
else
    echo -e "${GREEN}✅ La clé est bien protégée par passphrase${NC}"
fi

echo ""
echo -e "${GREEN}🎯 Clé de test prête !${NC}"
echo ""
echo -e "${YELLOW}📋 Instructions pour tester xsshend:${NC}"
echo ""
echo "1. Mode CLI avec sélection interactive de clé:"
echo "   cargo run -- upload --ssh-key-interactive /tmp/test-file.txt --env test"
echo ""
echo "2. Mode CLI avec nom de clé spécifique:"
echo "   cargo run -- upload --ssh-key test_passphrase_key /tmp/test-file.txt --env test"
echo ""
echo "3. Mode TUI interactif:"
echo "   cargo run"
echo ""
echo -e "${BLUE}💡 Attendu: xsshend devrait demander la passphrase '$PASSPHRASE'${NC}"
echo ""

# Créer un fichier de test simple
TEST_FILE="/tmp/test-file.txt"
echo "Contenu de test pour xsshend" > "$TEST_FILE"
echo -e "${GREEN}📄 Fichier de test créé: $TEST_FILE${NC}"

echo ""
echo -e "${YELLOW}⚠️  N'oubliez pas de nettoyer après les tests:${NC}"
echo "   rm ~/.ssh/test_passphrase_key ~/.ssh/test_passphrase_key.pub"
echo "   rm -rf $TEST_DIR"
echo "   rm $TEST_FILE"

# Fonction de nettoyage
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Nettoyage des fichiers de test...${NC}"
    rm -f ~/.ssh/test_passphrase_key ~/.ssh/test_passphrase_key.pub
    rm -rf "$TEST_DIR"
    rm -f "$TEST_FILE"
    echo -e "${GREEN}✅ Nettoyage terminé${NC}"
}

# Demander si l'utilisateur veut nettoyer maintenant
echo ""
read -p "$(echo -e ${YELLOW}Voulez-vous nettoyer les fichiers de test maintenant ? [y/N]: ${NC})" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cleanup
else
    echo -e "${BLUE}💡 Pour nettoyer plus tard, utilisez:${NC}"
    echo "   ./scripts/test_passphrase.sh --cleanup"
fi
