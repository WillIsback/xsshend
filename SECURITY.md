# Politique de Sécurité

## Vulnérabilités Connues

### RUSTSEC-2023-0071: Marvin Attack (RSA Timing Sidechannel)

**Statut**: ⚠️ Limitation connue - Pas de correctif disponible  
**Sévérité**: Moyenne (5.9/10)  
**Date de découverte**: 22 novembre 2023  
**Affecte**: `rsa 0.9.8` (dépendance transitive via `russh`)

#### Description

La crate `rsa` version 0.9.8 contient une implémentation non-constant-time qui peut permettre la fuite d'informations sur la clé privée via des canaux temporels observables sur le réseau. Un attaquant pourrait potentiellement utiliser ces informations pour récupérer la clé.

Cette vulnérabilité fait partie de l'attaque ["Marvin Attack"](https://people.redhat.com/~hkario/marvin/), qui a révélé que plusieurs implémentations RSA (incluant OpenSSL) n'avaient pas correctement mitigé les attaques par canaux temporels.

#### Chaîne de Dépendance

```
xsshend 0.4.1
└── russh 0.54.6
    ├── rsa 0.9.8 ⚠️
    └── internal-russh-forked-ssh-key 0.6.11
        └── rsa 0.9.8 ⚠️
```

#### Correctif

**Aucun correctif n'est actuellement disponible.**

Le projet [RustCrypto/RSA](https://github.com/RustCrypto/RSA) travaille sur une migration vers une implémentation entièrement constant-time, mais aucune version corrigée n'a encore été publiée.

**Suivi**:
- Issue RustCrypto: https://github.com/RustCrypto/RSA/issues/19
- Issue russh: https://github.com/Eugeny/russh/issues/337
- Advisory: https://rustsec.org/advisories/RUSTSEC-2023-0071

#### Solutions de Contournement

**La seule solution disponible est d'éviter d'utiliser la crate `rsa` dans des environnements où des attaquants peuvent observer les informations de timing**, par exemple :

✅ **Utilisation SÉCURISÉE** :
- Utilisation locale sur un ordinateur non compromis
- Réseaux privés et de confiance
- Environnements où l'attaquant n'a pas accès aux observations de timing réseau
- Connexions SSH sur localhost ou réseaux isolés

⚠️ **Utilisation À RISQUE** :
- Serveurs exposés publiquement sur Internet
- Connexions SSH sur des réseaux non sécurisés (WiFi public, etc.)
- Environnements où un attaquant actif peut observer le trafic réseau

#### Recommandations pour xsshend

1. **Utilisation en environnement contrôlé**: xsshend est principalement conçu pour une utilisation en environnement de développement ou d'administration système interne.

2. **Clés Ed25519 préférées**: Utilisez des clés Ed25519 plutôt que RSA lorsque c'est possible :
   ```bash
   ssh-keygen -t ed25519 -C "votre_email@example.com"
   ```

3. **Surveillance**: Surveillez l'advisory RUSTSEC-2023-0071 et les mises à jour de `russh` pour un correctif futur.

4. **Atténuation réseau**: 
   - Utilisez des connexions VPN pour les opérations sensibles
   - Limitez l'utilisation aux réseaux de confiance
   - Évitez l'utilisation sur des réseaux publics non sécurisés

#### Note Technique

Cette vulnérabilité affecte spécifiquement l'utilisation de clés **RSA**. Les autres algorithmes de clés supportés par SSH (Ed25519, ECDSA) ne sont **pas affectés** par cette vulnérabilité particulière.

## Signalement de Vulnérabilités

Si vous découvrez une vulnérabilité de sécurité dans xsshend, veuillez la signaler de manière responsable :

1. **NE PAS** créer une issue publique sur GitHub
2. Envoyer un email à : willisback@example.com
3. Inclure une description détaillée de la vulnérabilité
4. Laisser un délai raisonnable (90 jours) pour un correctif avant divulgation publique

## Historique des Vulnérabilités

| Date | Version | CVE/ID | Sévérité | Statut |
|------|---------|--------|----------|--------|
| 2023-11-22 | Toutes | RUSTSEC-2023-0071 | Moyenne (5.9) | ⚠️ Pas de correctif (dépendance transitive) |

## Versions Supportées

| Version | Support de Sécurité |
|---------|---------------------|
| 0.4.x   | ✅ Supportée        |
| 0.3.x   | ⚠️ Mises à jour critiques uniquement |
| < 0.3.0 | ❌ Non supportée    |

## Mises à Jour de Sécurité

Pour rester informé des mises à jour de sécurité :

1. Surveillez les [GitHub Releases](https://github.com/WillIsback/xsshend/releases)
2. Activez les [Security Advisories](https://github.com/WillIsback/xsshend/security/advisories) sur GitHub
3. Suivez le [CHANGELOG.md](CHANGELOG.md) pour les notes de version

---

**Dernière mise à jour**: 18 octobre 2025  
**Version du document**: 1.0
