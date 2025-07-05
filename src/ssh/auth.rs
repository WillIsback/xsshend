// Module d'authentification SSH - Placeholder
// TODO: Implémenter avec ssh2 et rpassword

use anyhow::Result;

pub struct SshAuth {
    // TODO: Ajouter les champs nécessaires
}

impl SshAuth {
    pub fn new() -> Self {
        SshAuth {}
    }

    pub fn authenticate(&self, _username: &str, _host: &str) -> Result<()> {
        // TODO: Implémenter authentification par clé SSH
        Ok(())
    }
}
