use gix::bstr::BString;
    use crate::object::tree::diff::{added, deleted, modified, store, tree_named};
        let from = tree_named(&repo, "@^{/r3-simple}~1");
        let to = tree_named(&repo, ":/r3-simple");

    #[test]
    fn realistic_renames() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r1-change}~1");
        let to = tree_named(&repo, ":/r1-change");

        let mut actual = Vec::new();
        let mut other = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(
                Rewrites {
                    copies: Some(Copies::default()),
                    limit: 1,
                    ..Default::default()
                }
                .into(),
            )
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    if let Event::Rewrite {
                        source_location, copy, ..
                    } = change.event
                    {
                        actual.push(source_location.to_owned());
                        actual.push(change.location.to_owned());
                        assert!(!copy);
                    } else {
                        other.push(store(&change));
                    }
                }
                Ok(Default::default())
            })?;

        assert_eq!(actual, vec!["git-index/src/file.rs", "git-index/src/file/mod.rs"]);
        assert_eq!(
            other,
            vec![
                added("git-index/tests/index/file/access.rs"),
                modified("git-index/tests/index/file/mod.rs")
            ]
        );

        #[cfg(not(windows))]
        {
            let actual = std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline.with-renames"))?;
            let expected = r#"commit 6974f2b5181772977a9d7d34a566414508552650
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r1-change

diff --git a/git-index/src/file.rs b/git-index/src/file/mod.rs
similarity index 100%
rename from git-index/src/file.rs
rename to git-index/src/file/mod.rs
diff --git a/git-index/tests/index/file/access.rs b/git-index/tests/index/file/access.rs
new file mode 100644
index 0000000..e69de29
diff --git a/git-index/tests/index/file/mod.rs b/git-index/tests/index/file/mod.rs
index e69de29..8ba3a16 100644
--- a/git-index/tests/index/file/mod.rs
+++ b/git-index/tests/index/file/mod.rs
@@ -0,0 +1 @@
+n
"#;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn realistic_renames_disabled() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r1-change}~1");
        let to = tree_named(&repo, ":/r1-change");

        let mut actual = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(None)
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    actual.push(store(&change));
                    if let Event::Rewrite { .. } = change.event {
                        unreachable!("it's disabled, so cannot happen")
                    }
                }
                Ok(Default::default())
            })?;

        assert_eq!(
            actual,
            vec![
                deleted("git-index/src/file.rs"),
                added("git-index/src/file/mod.rs"),
                added("git-index/tests/index/file/access.rs"),
                modified("git-index/tests/index/file/mod.rs")
            ]
        );

        #[cfg(not(windows))]
        {
            let actual = std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline.no-renames"))?;
            let expected = r#"commit 6974f2b5181772977a9d7d34a566414508552650
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r1-change

diff --git a/git-index/src/file.rs b/git-index/src/file.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-index/src/file/mod.rs b/git-index/src/file/mod.rs
new file mode 100644
index 0000000..e69de29
diff --git a/git-index/tests/index/file/access.rs b/git-index/tests/index/file/access.rs
new file mode 100644
index 0000000..e69de29
diff --git a/git-index/tests/index/file/mod.rs b/git-index/tests/index/file/mod.rs
index e69de29..8ba3a16 100644
--- a/git-index/tests/index/file/mod.rs
+++ b/git-index/tests/index/file/mod.rs
@@ -0,0 +1 @@
+n
"#;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn realistic_renames_disabled_2() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r2-change}~1");
        let to = tree_named(&repo, ":/r2-change");

        let mut actual = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(None)
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    actual.push(store(&change));
                    if let Event::Rewrite { .. } = change.event {
                        unreachable!("it's disabled, so cannot happen")
                    }
                }
                Ok(Default::default())
            })?;

        #[cfg(not(windows))]
        {
            let expected = r#"commit 72de3500e1bff816e56432bee8de02946d3e784b
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r2-change

diff --git a/git-sec/CHANGELOG.md b/git-sec/CHANGELOG.md
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/Cargo.toml b/git-sec/Cargo.toml
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/src/identity.rs b/git-sec/src/identity.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/src/lib.rs b/git-sec/src/lib.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/src/permission.rs b/git-sec/src/permission.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/src/trust.rs b/git-sec/src/trust.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/tests/identity/mod.rs b/git-sec/tests/identity/mod.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/git-sec/tests/sec.rs b/git-sec/tests/sec.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/gix-sec/CHANGELOG.md b/gix-sec/CHANGELOG.md
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/Cargo.toml b/gix-sec/Cargo.toml
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/src/identity.rs b/gix-sec/src/identity.rs
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/src/lib.rs b/gix-sec/src/lib.rs
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/src/permission.rs b/gix-sec/src/permission.rs
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/src/trust.rs b/gix-sec/src/trust.rs
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/tests/identity/mod.rs b/gix-sec/tests/identity/mod.rs
new file mode 100644
index 0000000..e69de29
diff --git a/gix-sec/tests/sec.rs b/gix-sec/tests/sec.rs
new file mode 100644
index 0000000..e69de29
"#;
            assert_eq!(
                std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline-2.no-renames"))?,
                expected
            );
        }

        assert_eq!(
            actual,
            vec![
                deleted("git-sec/CHANGELOG.md"),
                deleted("git-sec/Cargo.toml"),
                added("gix-sec/CHANGELOG.md"),
                added("gix-sec/Cargo.toml"),
                deleted("git-sec/src/identity.rs"),
                deleted("git-sec/src/lib.rs"),
                deleted("git-sec/src/permission.rs"),
                deleted("git-sec/src/trust.rs"),
                deleted("git-sec/tests/sec.rs"),
                added("gix-sec/src/identity.rs"),
                added("gix-sec/src/lib.rs"),
                added("gix-sec/src/permission.rs"),
                added("gix-sec/src/trust.rs"),
                added("gix-sec/tests/sec.rs"),
                deleted("git-sec/tests/identity/mod.rs"),
                added("gix-sec/tests/identity/mod.rs"),
            ]
        );

        Ok(())
    }

    #[test]
    fn realistic_renames_disabled_3() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r3-change}~1");
        let to = tree_named(&repo, ":/r3-change");

        let mut actual = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(None)
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    actual.push(store(&change));
                    if let Event::Rewrite { .. } = change.event {
                        unreachable!("it's disabled, so cannot happen")
                    }
                }
                Ok(Default::default())
            })?;

        #[cfg(not(windows))]
        {
            let expected = r#"commit dee00f5a20957db20d8d2e0050210716d6b44879
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r3-change

diff --git a/src/ein.rs b/src/ein.rs
new file mode 100644
index 0000000..e69de29
diff --git a/src/gix.rs b/src/gix.rs
new file mode 100644
index 0000000..e69de29
diff --git a/src/plumbing-cli.rs b/src/plumbing-cli.rs
deleted file mode 100644
index e69de29..0000000
diff --git a/src/porcelain-cli.rs b/src/porcelain-cli.rs
deleted file mode 100644
index e69de29..0000000
"#;

            assert_eq!(
                std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline-3.no-renames"))?,
                expected
            );
        }
        assert_eq!(
            actual,
            vec![
                added("src/ein.rs"),
                added("src/gix.rs"),
                deleted("src/plumbing-cli.rs"),
                deleted("src/porcelain-cli.rs"),
            ]
        );

        Ok(())
    }

    #[test]
    fn realistic_renames_3() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r3-change}~1");
        let to = tree_named(&repo, ":/r3-change");

        let mut actual = Vec::new();
        let mut other = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(
                Rewrites {
                    copies: Some(Copies::default()),
                    limit: 1,
                    ..Default::default()
                }
                .into(),
            )
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    if let Event::Rewrite {
                        source_location, copy, ..
                    } = change.event
                    {
                        actual.push(source_location.to_owned());
                        actual.push(change.location.to_owned());
                        assert!(!copy);
                    } else {
                        other.push(store(&change));
                    }
                }
                Ok(Default::default())
            })?;

        #[cfg(not(windows))]
        {
            let expected = r#"commit dee00f5a20957db20d8d2e0050210716d6b44879
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r3-change

diff --git a/src/plumbing-cli.rs b/src/ein.rs
similarity index 100%
rename from src/plumbing-cli.rs
rename to src/ein.rs
diff --git a/src/porcelain-cli.rs b/src/gix.rs
similarity index 100%
rename from src/porcelain-cli.rs
rename to src/gix.rs
"#;
            assert_eq!(
                std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline-3.with-renames"))?,
                expected
            );
        }
        assert_eq!(
            actual,
            vec![
                "src/plumbing-cli.rs",
                "src/ein.rs",
                "src/porcelain-cli.rs",
                "src/gix.rs"
            ]
        );
        assert!(other.is_empty());

        Ok(())
    }

    #[test]
    fn realistic_renames_2() -> crate::Result {
        let repo = named_repo("make_diff_repo.sh")?;
        let from = tree_named(&repo, "@^{/r2-change}~1");
        let to = tree_named(&repo, ":/r2-change");

        let mut actual = Vec::new();
        from.changes()?
            .track_path()
            .track_rewrites(
                Rewrites {
                    copies: Some(Copies::default()),
                    limit: 1,
                    ..Default::default()
                }
                .into(),
            )
            .for_each_to_obtain_tree(&to, |change| -> Result<_, Infallible> {
                if !change.event.entry_mode().is_tree() {
                    if let Event::Rewrite {
                        source_location, copy, ..
                    } = change.event
                    {
                        actual.push(source_location.to_owned());
                        actual.push(change.location.to_owned());
                        assert!(!copy);
                    } else {
                        unreachable!("everything is a rewrite");
                    }
                }
                Ok(Default::default())
            })?;

        #[cfg(not(windows))]
        {
            let expected = r#"commit 72de3500e1bff816e56432bee8de02946d3e784b
Author: author <author@example.com>
Date:   Sat Jan 1 00:00:00 2000 +0000

    r2-change

diff --git a/git-sec/CHANGELOG.md b/gix-sec/CHANGELOG.md
similarity index 100%
rename from git-sec/CHANGELOG.md
rename to gix-sec/CHANGELOG.md
diff --git a/git-sec/Cargo.toml b/gix-sec/Cargo.toml
similarity index 100%
rename from git-sec/Cargo.toml
rename to gix-sec/Cargo.toml
diff --git a/git-sec/src/identity.rs b/gix-sec/src/identity.rs
similarity index 100%
rename from git-sec/src/identity.rs
rename to gix-sec/src/identity.rs
diff --git a/git-sec/src/lib.rs b/gix-sec/src/lib.rs
similarity index 100%
rename from git-sec/src/lib.rs
rename to gix-sec/src/lib.rs
diff --git a/git-sec/src/permission.rs b/gix-sec/src/permission.rs
similarity index 100%
rename from git-sec/src/permission.rs
rename to gix-sec/src/permission.rs
diff --git a/git-sec/src/trust.rs b/gix-sec/src/trust.rs
similarity index 100%
rename from git-sec/src/trust.rs
rename to gix-sec/src/trust.rs
diff --git a/git-sec/tests/identity/mod.rs b/gix-sec/tests/identity/mod.rs
similarity index 100%
rename from git-sec/tests/identity/mod.rs
rename to gix-sec/tests/identity/mod.rs
diff --git a/git-sec/tests/sec.rs b/gix-sec/tests/sec.rs
similarity index 100%
rename from git-sec/tests/sec.rs
rename to gix-sec/tests/sec.rs
"#;
            assert_eq!(
                std::fs::read_to_string(repo.work_dir().expect("non-bare").join("baseline-2.with-renames"))?,
                expected
            );
        }

        assert_eq!(
            actual,
            vec![
                "git-sec/CHANGELOG.md",
                "gix-sec/CHANGELOG.md",
                "git-sec/Cargo.toml",
                "gix-sec/Cargo.toml",
                "git-sec/src/identity.rs",
                "gix-sec/src/identity.rs",
                "git-sec/src/lib.rs",
                "gix-sec/src/lib.rs",
                "git-sec/src/permission.rs",
                "gix-sec/src/permission.rs",
                "git-sec/src/trust.rs",
                "gix-sec/src/trust.rs",
                "git-sec/tests/sec.rs",
                "gix-sec/tests/sec.rs",
                "git-sec/tests/identity/mod.rs",
                "gix-sec/tests/identity/mod.rs"
            ]
        );

        Ok(())
    }
}
fn store(change: &gix::object::tree::diff::Change<'_, '_, '_>) -> (char, BString) {
    (shorthand(&change.event), change.location.to_owned())
}

fn added(path: &str) -> (char, BString) {
    ('A', path.into())
}

fn deleted(path: &str) -> (char, BString) {
    ('D', path.into())
}

fn modified(path: &str) -> (char, BString) {
    ('M', path.into())
}

fn shorthand(change: &Event) -> char {
    match change {
        Event::Addition { .. } => 'A',
        Event::Deletion { .. } => 'D',
        Event::Modification { .. } => 'M',
        Event::Rewrite { .. } => 'R',
    }