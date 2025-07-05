# Solution Rapide - Problème PATH Multipass

## 🔍 Problème Identifié

Multipass est installé via snap mais `/snap/bin` n'est pas dans votre PATH.

## ✅ Solution Immédiate

### 1. Pour cette session uniquement :
```bash
export PATH="/snap/bin:$PATH"
```

### 2. Solution permanente (recommandée) :

**Pour Bash :**
```bash
echo 'export PATH="/snap/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Pour Zsh :**
```bash
echo 'export PATH="/snap/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## 🧪 Test après correction

```bash
# Vérifier que multipass fonctionne
multipass version

# Lancer la démonstration
cd /home/william/projet/xsshend/test
./demo.sh
```

## 📋 Alternatives

### Méthode 1 : Diagnostic automatique
```bash
./check-multipass.sh
```

### Méthode 2 : Setup manuel
```bash
export PATH="/snap/bin:$PATH"
./test-vms.sh generate-keys
./test-vms.sh launch-all
./test-vms.sh generate-config
./run-integration-tests.sh
```

### Méthode 3 : Tests simples
```bash
# Test dry-run seulement
cd ..
cargo build
HOME=test/configs ./target/debug/xsshend upload test/data/simple.txt --env Development --dry-run
```

## ⚠️ Notes

- Le problème est spécifique aux applications snap
- La solution est standard et sûre
- Après correction, tous les scripts fonctionneront normalement
- La démonstration créera 5 VMs (peut prendre 5-10 minutes)

## 🎯 Résultat Attendu

Après correction du PATH :
- ✅ `multipass version` fonctionne
- ✅ `./demo.sh` lance la démonstration complète
- ✅ Tests d'intégration sur vraies VMs Ubuntu
- ✅ Validation complète de xsshend
