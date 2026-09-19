use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = manifest_dir.join("assets/icon.ico");
    let resource_script = out_dir.join("fighting-stick.rc");
    let resource = out_dir.join("fighting-stick.res");
    let icon_path = icon.to_string_lossy().replace('\\', "\\\\");

    fs::write(&resource_script, format!("1 ICON \"{icon_path}\"\n"))
        .expect("não foi possível criar o script de recursos do Windows");

    let status = Command::new("rc.exe")
        .arg("/nologo")
        .arg(format!("/fo{}", resource.display()))
        .arg(&resource_script)
        .status()
        .expect("rc.exe não foi encontrado; instale o Windows SDK");

    assert!(status.success(), "rc.exe falhou ao incorporar o ícone");
    println!("cargo:rustc-link-arg={}", resource.display());
}
