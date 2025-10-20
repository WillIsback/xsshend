#[cfg(test)]
mod tests {
    use xsshend::utils::path_expansion::expand_path;

    #[test]
    fn test_tilde_expansion_default_home() {
        // Tilde seul avec HOME par défaut
        assert_eq!(
            expand_path("~/work/tmp", "alice", None).unwrap(),
            "/home/alice/work/tmp"
        );

        // Tilde avec utilisateur
        assert_eq!(
            expand_path("~bob/files", "alice", None).unwrap(),
            "/home/bob/files"
        );

        // Tilde avec utilisateur sans slash
        assert_eq!(
            expand_path("~charlie", "alice", None).unwrap(),
            "/home/charlie"
        );
    }

    #[test]
    fn test_tilde_expansion_custom_home() {
        // Tilde avec HOME personnalisé (comme /appli/002/user)
        assert_eq!(
            expand_path("~/work/tmp", "alice", Some("/appli/002/alice")).unwrap(),
            "/appli/002/alice/work/tmp"
        );
        assert_eq!(
            expand_path("~/documents", "bob", Some("/opt/users/bob")).unwrap(),
            "/opt/users/bob/documents"
        );
    }

    #[test]
    fn test_variable_expansion() {
        // Variables avec HOME par défaut
        assert_eq!(
            expand_path("$HOME/documents", "alice", None).unwrap(),
            "/home/alice/documents"
        );
        assert_eq!(expand_path("$USER/tmp", "bob", None).unwrap(), "bob/tmp");

        // Variables avec HOME personnalisé
        assert_eq!(
            expand_path("$HOME/work", "charlie", Some("/appli/002/charlie")).unwrap(),
            "/appli/002/charlie/work"
        );
    }

    #[test]
    fn test_absolute_paths_unchanged() {
        assert_eq!(
            expand_path("/tmp/files", "alice", None).unwrap(),
            "/tmp/files"
        );
        assert_eq!(
            expand_path("/opt/work", "bob", Some("/custom/home")).unwrap(),
            "/opt/work"
        );
    }

    #[test]
    fn test_mixed_expansion() {
        // Combinaisons multiples avec HOME personnalisé
        assert_eq!(
            expand_path("$HOME/work/$USER", "alice", Some("/appli/002/alice")).unwrap(),
            "/appli/002/alice/work/$USER"
        ); // $HOME/ est remplacé mais pas $USER seul
    }

    #[test]
    fn test_unknown_variables_preserved() {
        // Variables inconnues laissées telles quelles
        assert_eq!(
            expand_path("$WORK/tmp", "alice", None).unwrap(),
            "$WORK/tmp"
        );
        assert_eq!(
            expand_path("/opt/$APP/logs", "bob", None).unwrap(),
            "/opt/$APP/logs"
        );
    }
}
