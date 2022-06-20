use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use bstr::BString;
use dirs::home_dir;
use git_config::File;
use serial_test::serial;
use tempfile::{tempdir, tempdir_in};

use crate::file::from_paths::includes::conditional::create_symlink;
use crate::file::{
    cow_str,
    from_paths::{escape_backslashes, includes::conditional::options_with_git_dir},
};

#[derive(Eq, PartialEq)]
enum ConfigLocation {
    Repo,
    User,
}

enum Value {
    Original,
    Override,
}

struct Options<'a> {
    condition: &'a str,
    env: EnvOverwrite,
    expected: Value,
    config_location: ConfigLocation,
}

struct EnvOverwrite {
    git_dir: PathBuf,
    repo_dir: PathBuf,
    home_dir: PathBuf,
}

impl Default for EnvOverwrite {
    fn default() -> Self {
        Self {
            git_dir: PathBuf::default(),
            repo_dir: PathBuf::default(),
            home_dir: home(),
        }
    }
}

impl EnvOverwrite {
    fn new(git_dir: PathBuf, home_dir: Option<PathBuf>) -> Self {
        let repo_dir = git_dir.parent().unwrap().into();
        Self {
            git_dir,
            repo_dir,
            home_dir: match home_dir {
                Some(home_dir) => home_dir,
                None => home(),
            },
        }
    }
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            condition: "",
            env: EnvOverwrite::default(),
            expected: Value::Override,
            config_location: ConfigLocation::Repo,
        }
    }
}

fn assert_section_value(
    Options {
        condition,
        expected,
        env,
        config_location,
    }: Options,
) {
    write_config(condition, &env, &config_location);

    git_assert_eq(&expected, &env);

    let mut paths = vec![env.git_dir.join("config")];
    if config_location == ConfigLocation::User {
        paths.push(env.home_dir.join(".gitconfig"));
    }

    let config = File::from_paths(paths, options_with_git_dir(&env.git_dir)).unwrap();

    assert_eq!(
        config.string("section", None, "value"),
        Some(cow_str(match expected {
            Value::Original => "base-value",
            Value::Override => "override-value",
        })),
        "gitoxide assert equals",
    );
}

fn git_dir(root_dir: &Path, git_dir_suffix: &str) -> PathBuf {
    let git_dir = root_dir.join(git_dir_suffix).join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    git_dir
}

fn write_config(condition: &str, env: &EnvOverwrite, overwrite_config_location: &ConfigLocation) {
    let override_config_dir_file = write_override_config(env.repo_dir.as_path());
    write_main_config(condition, override_config_dir_file, env, overwrite_config_location);
}

fn write_override_config(root_path: &Path) -> PathBuf {
    let include_path = root_path.join("include.path");
    fs::create_dir_all(root_path).unwrap();
    fs::write(
        include_path.as_path(),
        "
[section]
  value = override-value",
    )
    .unwrap();
    include_path
}

fn git_assert_eq(expected: &Value, env: &EnvOverwrite) {
    let output = Command::new("git")
        .args(["config", "--get", "section.value"])
        .env("HOME", env.home_dir.as_path())
        .env("GIT_DIR", env.git_dir.as_path())
        .current_dir(env.git_dir.as_path())
        .output()
        .unwrap();

    assert_eq!(output.stderr.len(), 0);
    let git_output = BString::from(output.stdout.strip_suffix(b"\n").unwrap());
    assert_eq!(
        git_output,
        match expected {
            Value::Original => "base-value",
            Value::Override => "override-value",
        },
        "git assert equals"
    );
}

fn write_main_config(
    condition: &str,
    override_config_dir_file: PathBuf,
    env: &EnvOverwrite,
    overwrite_config_location: &ConfigLocation,
) {
    Command::new("git")
        .args(["init", env.repo_dir.to_str().unwrap()])
        .output()
        .unwrap();

    if overwrite_config_location == &ConfigLocation::Repo {
        Command::new("git")
            .args(["config", "section.value", "base-value"])
            .env("GIT_DIR", env.git_dir.as_path())
            .output()
            .unwrap();
    }

    Command::new("git")
        .args([
            "config",
            match overwrite_config_location {
                ConfigLocation::User => "--global",
                ConfigLocation::Repo => "--local",
            },
            &format!("includeIf.{}.path", condition),
            &escape_backslashes(override_config_dir_file.as_path()),
        ])
        .current_dir(env.git_dir.as_path())
        .env("HOME", env.home_dir.as_path())
        .output()
        .unwrap();
}

fn home() -> PathBuf {
    home_dir().unwrap()
}

#[test]
#[serial]
fn relative_path_with_trailing_slash() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir:foo/",
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
fn tilde_expansion() {
    let tmp_dir = tempdir_in(home()).unwrap();
    let root = tmp_dir
        .path()
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();
    let git_dir = git_dir(tmp_dir.path(), "foo");

    assert_section_value(Options {
        condition: &format!("gitdir:~/{}/foo/", root),
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
fn star_star_prefix_and_suffix() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir:**/foo/**",
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
fn dot_path_slash() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir:./",
        env: EnvOverwrite::new(git_dir, Some(temp_dir.path().into())),
        config_location: ConfigLocation::User,
        ..Default::default()
    });
}

#[test]
#[serial]
fn dot_path() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir:./foo/.git",
        env: EnvOverwrite::new(git_dir, Some(temp_dir.path().into())),
        config_location: ConfigLocation::User,
        ..Default::default()
    });
}

#[test]
#[serial]
fn case_insensitive() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir/i:FOO/",
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
#[ignore]
fn pattern_with_backslash() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo");
    assert_section_value(Options {
        condition: "gitdir:\\foo/",
        env: EnvOverwrite {
            git_dir,
            ..Default::default()
        },
        expected: Value::Original,
        ..Default::default()
    });
}

#[test]
#[serial]
fn star_star_in_the_middle() {
    let temp_dir = tempdir().unwrap();
    let git_dir = git_dir(temp_dir.path(), "foo/bar");
    assert_section_value(Options {
        condition: "gitdir:**/foo/**/bar/**",
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn tilde_expansion_with_symlink() {
    let tmp_dir = tempdir_in(home()).unwrap();
    let root = tmp_dir
        .path()
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    let _ = git_dir(tmp_dir.path(), "foo");
    create_symlink(
        tmp_dir.path().join("bar").as_path(),
        tmp_dir.path().join("foo").as_path(),
    );
    let git_dir = tmp_dir.path().join("bar").join(".git");

    assert_section_value(Options {
        condition: &format!("gitdir:~/{}/bar/", root),
        env: EnvOverwrite::new(git_dir, None),
        ..Default::default()
    });
}

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn dot_path_with_symlink() {
    let tmp_dir = tempdir_in(home()).unwrap();
    let _ = git_dir(tmp_dir.path(), "foo");
    create_symlink(
        tmp_dir.path().join("bar").as_path(),
        tmp_dir.path().join("foo").as_path(),
    );
    let git_dir = tmp_dir.path().join("bar").join(".git");

    assert_section_value(Options {
        condition: "gitdir:./bar/.git",
        env: EnvOverwrite::new(git_dir, Some(tmp_dir.path().into())),
        config_location: ConfigLocation::User,
        ..Default::default()
    });
}

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn dot_path_matching_symlink() {
    let tmp_dir = tempdir_in(home()).unwrap();
    let _ = git_dir(tmp_dir.path(), "foo");
    create_symlink(
        tmp_dir.path().join("bar").as_path(),
        tmp_dir.path().join("foo").as_path(),
    );
    let git_dir = tmp_dir.path().join("bar").join(".git");

    assert_section_value(Options {
        condition: "gitdir:bar/",
        env: EnvOverwrite::new(git_dir, Some(tmp_dir.path().into())),
        config_location: ConfigLocation::User,
        ..Default::default()
    });
}

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn dot_path_matching_symlink_with_icase() {
    let tmp_dir = tempdir_in(home()).unwrap();
    let _ = git_dir(tmp_dir.path(), "foo");
    create_symlink(
        tmp_dir.path().join("bar").as_path(),
        tmp_dir.path().join("foo").as_path(),
    );

    let git_dir = tmp_dir.path().join("bar").join(".git");
    assert_section_value(Options {
        condition: "gitdir/i:BAR/",
        env: EnvOverwrite::new(git_dir, Some(tmp_dir.path().into())),
        config_location: ConfigLocation::User,
        ..Default::default()
    });
}