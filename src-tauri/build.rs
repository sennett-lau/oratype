fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    } else if target_os == "windows" && target_env == "gnu" {
        // ggml-cpu.cpp calls RegOpenKeyExA/RegQueryValueExA/RegCloseKey directly, while
        // windows-targets uses raw-dylib and omits -ladvapi32 from the GNU linker line.
        println!("cargo:rustc-link-arg=-ladvapi32");
        println!("cargo:rerun-if-env-changed=ORATYPE_WIN_VULKAN_SDK");
        let sdk_dir = std::env::var("ORATYPE_WIN_VULKAN_SDK")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "/tmp/oratype-vulkan-sdk".to_string());
        let sdk_lib = std::path::Path::new(&sdk_dir).join("lib");
        if sdk_lib.join("libvulkan.dll.a").exists() {
            println!("cargo:rustc-link-search=native={}", sdk_lib.display());
        }
    }

    #[cfg(target_os = "macos")]
    link_clang_runtime();

    // Plan `ai-polish-setup`: the built-in AI server binary is named after the target triple
    // (`llama-server-aarch64-apple-darwin`), as Tauri external binaries are.
    println!(
        "cargo:rustc-env=ORATYPE_TARGET_TRIPLE={}",
        std::env::var("TARGET").unwrap_or_default()
    );

    tauri_build::build()
}

/// Plan `quick-speech-setup`: whisper.cpp's Metal code uses Objective-C `@available(...)` checks.
/// When the app is built for an older macOS than the one it is built on (the release bundle targets
/// `minimumSystemVersion`), clang turns those checks into calls to
/// `__isPlatformVersionAtLeast`, which lives in clang's runtime library `libclang_rt.osx.a`.
/// Rust links with `-nodefaultlibs`, so that library is added here by hand.
#[cfg(target_os = "macos")]
fn link_clang_runtime() {
    let output = std::process::Command::new("cc")
        .arg("--print-runtime-dir")
        .output();
    let Ok(output) = output else {
        println!("cargo:warning=could not ask cc for the clang runtime directory");
        return;
    };
    let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if dir.is_empty()
        || !std::path::Path::new(&dir)
            .join("libclang_rt.osx.a")
            .is_file()
    {
        println!("cargo:warning=libclang_rt.osx.a not found in {dir:?}");
        return;
    }
    println!("cargo:rustc-link-search=native={dir}");
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
}
