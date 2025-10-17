# Correctif Version 0.3.2 - Résolution DNS

## Problème Identifié

L'application crashait avec l'erreur suivante lors de la connexion à des serveurs SSH :

```
thread 'main' panicked at src/ssh/client.rs:33:77:
called `Result::unwrap()` on an `Err` value: AddrParseError(Socket)
```

### Cause Racine

Le code essayait de parser directement un hostname + port (`"hostname:22"`) comme une `SocketAddr`, ce qui ne fonctionne qu'avec des adresses IP. Avec des FQDN (Fully Qualified Domain Names) comme `pp-grhnic-4gl01.sigrh.hp.in.phm.education.gouv.fr`, le parsing échouait.

```rust
// ❌ Code problématique (avant)
let tcp = TcpStream::connect_timeout(
    &format!("{}:22", self.host).parse().unwrap(),  // ← Panic ici avec un hostname
    timeout
)?;
```

## Solution Appliquée

Utilisation de `ToSocketAddrs` pour résoudre correctement le hostname en adresse IP avant de se connecter :

```rust
// ✅ Code corrigé (après)
use std::net::ToSocketAddrs;

let addr = format!("{}:22", self.host)
    .to_socket_addrs()
    .with_context(|| format!("Impossible de résoudre l'adresse {}", self.host))?
    .next()
    .ok_or_else(|| anyhow::anyhow!("Aucune adresse IP trouvée pour {}", self.host))?;

let tcp = TcpStream::connect_timeout(&addr, timeout)
    .with_context(|| format!("Impossible de se connecter à {}:22 ({})", self.host, addr))?;
```

## Améliorations

1. **Résolution DNS automatique** : Les hostnames sont maintenant correctement résolus en adresses IP
2. **Gestion d'erreurs robuste** : Messages d'erreur clairs en cas de problème de résolution DNS
3. **Support complet des FQDN** : Fonctionne maintenant avec n'importe quel nom de domaine valide
4. **Messages de débogage** : L'adresse IP résolue est affichée dans les messages d'erreur

## Tests

- ✅ Compilation : OK
- ✅ Tests unitaires : 93 tests passants
- ✅ Cargo check : OK

## Déploiement

Pour mettre à jour sur votre machine distante :

```bash
cargo install xsshend --force
```

Ou si vous avez déjà installé la version 0.3.2 avec le bug :

```bash
cargo uninstall xsshend
cargo install xsshend
```

## Fichiers Modifiés

- `src/ssh/client.rs` : Correction de la méthode `connect_with_timeout()`
- `CHANGELOG.md` : Documentation du correctif version 0.3.2
