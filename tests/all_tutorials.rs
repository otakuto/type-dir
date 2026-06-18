use std::path::PathBuf;

use assert_cmd::Command;

/// Integration test that launches the `dir-lint` binary and verifies its exit code.
///
/// Recursively scans `tutorials/` and `rule-tests/` to auto-collect every directory
/// that contains a `.dir-lint.yaml` as a fixture.
///
/// - Directories whose name ends with `-negative` or `_negative` → expect non-zero exit (error detected)
/// - All others → expect zero exit (no errors)
#[test]
fn all_fixtures_behave_as_expected() {
    // Arrange: collect fixture directories
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tutorials_root = manifest_dir.join("tutorials");
    let rule_tests_root = manifest_dir.join("rule-tests");

    let mut fixtures: Vec<PathBuf> = Vec::new();
    collect_fixtures(&tutorials_root, &mut fixtures);
    collect_fixtures(&rule_tests_root, &mut fixtures);
    fixtures.sort();

    // Guarantee a minimum fixture count to prevent vacuous passes
    assert!(
        fixtures.len() >= 10,
        "expected to scan many fixtures, only found {} — roots may be wrong: tutorials={tutorials_root:?} rule-tests={rule_tests_root:?}",
        fixtures.len(),
    );

    let mut failures: Vec<String> = Vec::new();

    for fixture in &fixtures {
        // Act: launch the binary and run check
        let dir_name = fixture
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let is_negative = dir_name.ends_with("-negative") || dir_name.ends_with("_negative");

        let output = Command::cargo_bin("dir-lint")
            .expect("dir-lint binary must be buildable")
            .current_dir(fixture)
            .arg("check")
            .output()
            .expect("failed to run dir-lint");

        // Assert: verify that the exit code matches the expectation
        let succeeded = output.status.success();
        if is_negative && succeeded {
            let stderr = String::from_utf8_lossy(&output.stderr);
            failures.push(format!(
                "[{}] (negative) expected failure (non-zero exit), but succeeded\nstderr: {stderr}",
                fixture.display(),
            ));
        } else if !is_negative && !succeeded {
            let stderr = String::from_utf8_lossy(&output.stderr);
            failures.push(format!(
                "[{}] (positive) expected success, but failed\nstderr: {stderr}",
                fixture.display(),
            ));
        }
    }

    // Assert: after scanning all fixtures, report all failures at once
    assert!(
        failures.is_empty(),
        "{} fixture(s) did not behave as expected:\n{}",
        failures.len(),
        failures.join("\n---\n"),
    );
}

/// Recursively scans `root` and appends every directory that contains `.dir-lint.yaml` to `out`.
fn collect_fixtures(root: &PathBuf, out: &mut Vec<PathBuf>) {
    let config_path = root.join(".dir-lint.yaml");
    if config_path.exists() {
        out.push(root.clone());
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            collect_fixtures(&entry.path(), out);
        }
    }
}
