use std::{env::var_os, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=FORCE_MVK_FROM_SOURCE");

    let force = match var_os("FORCE_MVK_FROM_SOURCE") {
        None => false,
        Some(value) => value == "yes" || value == "1",
    };

    let use_from_source = if force {
        true
    } else {
        unsafe { libloading::Library::new("libvulkan.dylib").is_err() }
    };

    let out_dir = var_os("OUT_DIR").expect("Failed to find OUT_DIR");
    let out_dir = Path::new(&out_dir);
    let target_lib_path = out_dir.join("libvulkan.dylib");

    if !use_from_source {
        if target_lib_path.exists() {
            let _ = std::fs::remove_file(&target_lib_path);
        }
        return;
    }

    let out_dir = var_os("OUT_DIR").expect("Failed to find OUT_DIR");
    let out_dir = Path::new(&out_dir);
    let mvk_dir = out_dir.join("MoltenVK");

    let (target_name, dylib_dir) = match std::env::var("CARGO_CFG_TARGET_OS") {
        Ok(target) => match target.as_ref() {
            "macos" => ("macos", "macOS"),
            "ios" => ("ios", "iOS"),
            target => panic!("Unknown target '{}'", target),
        },
        Err(e) => panic!("Failed to determinte target os '{}'", e),
    };

    if mvk_dir.exists() {
        let git_status = Command::new("git")
            .current_dir(&mvk_dir)
            .args(["pull", "--ff-only"])
            .spawn()
            .expect("Failed to run git")
            .wait()
            .expect("Failed to pull MoltenVK");

        assert!(git_status.success(), "Failed to get MoltenVK");
    } else {
        let git_status = Command::new("git")
            .arg("clone")
            .args(["--depth", "1"])
            .arg("https://github.com/KhronosGroup/MoltenVK.git")
            .arg(&mvk_dir)
            .spawn()
            .expect("Failed to run git")
            .wait()
            .expect("Failed to clone MoltenVK");

        assert!(git_status.success(), "Failed to get MoltenVK");
    };

    let status = Command::new("sh")
        .current_dir(&mvk_dir)
        .arg("fetchDependencies")
        .arg(format!("--{}", target_name))
        .spawn()
        .expect("Failed to run fetchDependencies script")
        .wait()
        .expect("Failed to fetch dependencies");

    assert!(status.success(), "Failed to fetch dependencies");

    let status = Command::new("make")
        .current_dir(&mvk_dir)
        .arg(target_name)
        .spawn()
        .expect("Failed to build MoltenVK")
        .wait()
        .expect("Failed to build MoltenVK");

    assert!(status.success(), "Failed to build MoltenVK");

    let dylib_path = mvk_dir
        .join("MoltenVK")
        .join("dylib")
        .join(dylib_dir)
        .join("libMoltenVK.dylib");

    std::fs::copy(&dylib_path, &target_lib_path).expect("Failed to copy MoltenVK dylib");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
}
