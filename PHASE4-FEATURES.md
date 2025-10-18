# Phase 4 - Nouvelles Fonctionnalités ✨

La version 0.4.9 apporte les améliorations de la Phase 4 :

## 1. Barre de progression pour l'exécution séquentielle 📊

Lorsque vous exécutez des commandes en mode séquentiel (par défaut), une barre de progression s'affiche automatiquement :

```bash
xsshend command --inline "uptime" --env Production
```

La barre affiche :
- Le serveur en cours de traitement
- La progression (X/Total)
- Le temps écoulé
- Indicateur visuel de progression

## 2. Format de sortie JSON 🔧

Pour faciliter le parsing automatique des résultats, utilisez `--output-format json` :

```bash
xsshend command --inline "hostname" --env Test --output-format json
```

Structure JSON retournée :
```json
{
  "summary": {
    "total": 5,
    "success": 5,
    "failed": 0,
    "total_duration_secs": 2.34
  },
  "results": [
    {
      "host": "server1",
      "exit_code": 0,
      "stdout": "server1.example.com\n",
      "stderr": "",
      "duration": 0.45,
      "success": true
    }
  ]
}
```

Parfait pour les pipelines CI/CD et l'automatisation :

```bash
# Exemple : Extraire seulement les serveurs en échec
xsshend command --inline "test -f /app/ready" --env Production --output-format json \
  | jq -r '.results[] | select(.success == false) | .host'

# Exemple : Vérifier le taux de réussite
SUCCESS=$(xsshend command --inline "uptime" --env Staging --output-format json \
  | jq '.summary.success')
echo "Succès: $SUCCESS serveurs"
```

## 3. Logs détaillés avec RUST_LOG 🔍

Pour diagnostiquer les problèmes, activez les logs debug :

```bash
RUST_LOG=debug xsshend command --inline "whoami" --env Test
```

Les logs détaillent :
- Les connexions SSH établies
- L'exécution des commandes
- Les données reçues (stdout/stderr)
- Les codes de sortie
- Les durées d'exécution

Niveaux disponibles :
- `RUST_LOG=error` : Erreurs uniquement
- `RUST_LOG=warn` : Warnings + erreurs
- `RUST_LOG=info` : Informations générales
- `RUST_LOG=debug` : Détails de débogage
- `RUST_LOG=trace` : Traces détaillées (verbose)

Exemple de log debug :
```
[DEBUG] Début d'exécution sur server1 (user@192.168.1.10)
[DEBUG] Connexion SSH à user@192.168.1.10
[DEBUG] Exécution de la commande (timeout: 30s)
[DEBUG] execute_command: 'whoami'
[DEBUG] Ouverture d'un canal SSH
[DEBUG] Envoi de la commande au serveur
[DEBUG] Lecture de la sortie (timeout: 30s)
[TRACE] Reçu 5 octets sur stdout
[DEBUG] Code de sortie: 0
[DEBUG] Fin de la sortie (EOF)
[DEBUG] Commande terminée - stdout: 5 octets, stderr: 0 octets, exit: 0
[DEBUG] Commande terminée - Exit: 0, Durée: 453ms
```

## Exemples complets

### 1. Vérification de santé avec barre de progression
```bash
xsshend command --inline "systemctl is-active nginx" --env Production
```

### 2. Collecte de métriques en JSON
```bash
xsshend command --inline "df -h / | tail -1 | awk '{print \$5}'" \
  --env Production --output-format json > disk_usage.json
```

### 3. Debug d'un problème de connexion
```bash
RUST_LOG=debug xsshend command --inline "hostname" --env Staging --verbose
```

### 4. Exécution parallèle avec sortie JSON
```bash
xsshend command \
  --inline "curl -s http://localhost:8080/health" \
  --env Production \
  --parallel \
  --output-format json \
  | jq '.summary'
```

## Notes

- La barre de progression s'affiche uniquement en mode séquentiel (sans `--parallel`)
- Le format JSON désactive les affichages intermédiaires pour un JSON pur
- Les logs debug sont indépendants du format de sortie
- En mode JSON, la barre de progression est masquée automatiquement
