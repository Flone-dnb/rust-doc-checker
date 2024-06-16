#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::doc_checker::DocChecker;

    fn get_project_root() -> PathBuf {
        let mut path = std::env::current_dir().unwrap();

        loop {
            // Check if cargo exists in this directory.
            let test_path = path.join("Cargo.lock");
            if test_path.exists() {
                return path;
            }

            // Go to parent directory.
            path = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => panic!(),
            }
        }
    }

    fn test_doc_check_fail_success(test_dir: &str) {
        let doc_checker = DocChecker::new();

        let path_to_res = get_project_root().join("tests").join(test_dir);

        let mut paths_to_fail = Vec::new();
        let mut paths_to_success = Vec::new();

        let path_to_fail = path_to_res.join("fail.rs");
        let path_to_success = path_to_res.join("success.rs");

        if !path_to_fail.exists() && !path_to_success.exists() {
            if path_to_res.join("fail1.rs").exists() {
                // Add fail files.
                let mut test_file_number = 1usize;
                loop {
                    // Check if exists.
                    let path = path_to_res.join(format!("fail{}.rs", test_file_number));
                    if !path.exists() {
                        break;
                    }

                    // Add.
                    paths_to_fail.push(path);
                    test_file_number += 1;
                }
            }

            if path_to_res.join("success1.rs").exists() {
                // Add success files.
                let mut test_file_number = 1usize;
                loop {
                    // Check if exists.
                    let path = path_to_res.join(format!("success{}.rs", test_file_number));
                    if !path.exists() {
                        break;
                    }

                    // Add.
                    paths_to_success.push(path);
                    test_file_number += 1;
                }
            }
        } else {
            paths_to_fail.push(path_to_fail);
            paths_to_success.push(path_to_success);
        }

        assert!(!paths_to_fail.is_empty() || !paths_to_success.is_empty());

        for path in &paths_to_fail {
            assert!(path.exists());
            assert!(!path.is_dir());
        }
        for path in &paths_to_success {
            assert!(path.exists());
            assert!(!path.is_dir());
        }

        // Test fail.
        for path in paths_to_fail {
            let input = std::fs::read_to_string(path.clone()).unwrap();

            match doc_checker.check_documentation(&input, false) {
                Ok(_) => panic!("expected the test to fail (file {})", path.display()),
                Err(_) => {}
            }
        }

        // Test success.
        for path in paths_to_success {
            let input = std::fs::read_to_string(path.clone()).unwrap();

            match doc_checker.check_documentation(&input, false) {
                Ok(_) => {}
                Err(msg) => panic!("{} (file {})", msg, path.display()),
            }
        }
    }

    #[test]
    fn func_docs() {
        test_doc_check_fail_success("func_docs");
    }

    #[test]
    fn struct_docs() {
        test_doc_check_fail_success("struct_docs");
    }
}
