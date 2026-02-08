use std::{env, fs, path::PathBuf};
use zed_extension_api::{
    self as zed, Architecture, DownloadedFileType, LanguageServerId, Os, Result,
};

const GITHUB_REPOSITORY: &str = "dawsh2/zed-mermaid-preview";
const CACHE_ROOT: &str = "mermaid-lsp-cache";

struct MermaidPreviewExtension {
    lsp_path: Option<String>,
}

impl zed::Extension for MermaidPreviewExtension {
    fn new() -> Self {
        Self { lsp_path: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_path = self.get_lsp_path(worktree, language_server_id)?;
        eprintln!("Starting Mermaid LSP at: {lsp_path}");

        Ok(zed::Command {
            command: lsp_path,
            args: vec![],
            env: Default::default(),
        })
    }
}

impl MermaidPreviewExtension {
    fn get_lsp_path(
        &mut self,
        worktree: &zed::Worktree,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        if let Some(ref path) = self.lsp_path {
            return Ok(path.clone());
        }

        let extension_dir = env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {e}"))?;

        self.resolve_lsp_path(language_server_id, worktree, &extension_dir)
    }

    fn resolve_lsp_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
        extension_dir: &std::path::Path,
    ) -> Result<String> {
        // 1. Check MERMAID_LSP_PATH environment variable
        if let Ok(path) = env::var("MERMAID_LSP_PATH") {
            let candidate = PathBuf::from(&path);
            if candidate.is_file() {
                return Self::finalize_path(language_server_id, candidate, &mut self.lsp_path);
            }
        }

        // 2. Check worktree PATH
        if let Some(path) = worktree.which("mermaid-lsp") {
            return Self::finalize_path(
                language_server_id,
                PathBuf::from(path),
                &mut self.lsp_path,
            );
        }

        // 3. Check local candidate paths (bundled binaries)
        let binary_name = Self::binary_name();
        if let Some(path) = Self::candidate_paths(extension_dir, binary_name)
            .into_iter()
            .find(|p| p.is_file())
        {
            return Self::finalize_path(language_server_id, path, &mut self.lsp_path);
        }

        // 4. Download from GitHub Releases
        match self.download_lsp(language_server_id, extension_dir, binary_name) {
            Ok(path) if path.is_file() => {
                Self::finalize_path(language_server_id, path, &mut self.lsp_path)
            }
            Err(e) => Err(format!("Failed to download LSP binary: {e}")),
            _ => Err(format!(
                "LSP binary '{binary_name}' not found. Set MERMAID_LSP_PATH or publish a GitHub release."
            )),
        }
    }

    fn finalize_path(
        language_server_id: &LanguageServerId,
        path: PathBuf,
        cache: &mut Option<String>,
    ) -> Result<String> {
        let resolved = path
            .canonicalize()
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        *cache = Some(resolved.clone());

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        Ok(resolved)
    }

    fn candidate_paths(extension_dir: &std::path::Path, binary_name: &str) -> Vec<PathBuf> {
        let mut candidates = vec![
            extension_dir.join(binary_name),
            extension_dir.join("target/release").join(binary_name),
            extension_dir.join("target/debug").join(binary_name),
            extension_dir.join("bin").join(binary_name),
            extension_dir.join("lsp/target/release").join(binary_name),
        ];

        // Check cached versions
        let cache_root = extension_dir.join(CACHE_ROOT);
        if let Ok(entries) = fs::read_dir(cache_root) {
            for entry in entries.flatten() {
                candidates.push(entry.path().join(binary_name));
            }
        }

        candidates
    }

    fn download_lsp(
        &mut self,
        language_server_id: &LanguageServerId,
        extension_dir: &std::path::Path,
        binary_name: &str,
    ) -> Result<PathBuf> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            GITHUB_REPOSITORY,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset = Self::match_asset(&release)?;
        let version_dir = extension_dir.join(CACHE_ROOT).join(&release.version);
        let binary_path = version_dir.join(binary_name);

        // Already have this version
        if binary_path.is_file() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );
            return Ok(binary_path);
        }

        // Create cache directory
        fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create cache directory: {e}"))?;

        // Download
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        zed::download_file(
            &asset.download_url,
            version_dir
                .to_str()
                .ok_or_else(|| "Failed to stringify cache path".to_string())?,
            DownloadedFileType::Zip,
        )
        .map_err(|e| format!("Failed to download mermaid-lsp: {e}"))?;

        if !binary_path.is_file() {
            return Err(format!(
                "Downloaded asset '{}' did not contain expected binary '{binary_name}'",
                asset.name
            ));
        }

        zed::make_file_executable(
            binary_path
                .to_str()
                .ok_or_else(|| "Failed to stringify binary path".to_string())?,
        )?;

        // Purge old versions
        Self::purge_old_cache_versions(extension_dir, &release.version);

        eprintln!("Mermaid LSP v{} installed", release.version);
        Ok(binary_path)
    }

    fn match_asset(release: &zed::GithubRelease) -> Result<zed::GithubReleaseAsset> {
        let (os, arch) = zed::current_platform();

        let arch_str = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X86 => "x86",
            Architecture::X8664 => "x86_64",
        };

        let os_str = match os {
            Os::Mac => "apple-darwin",
            Os::Linux => "unknown-linux-gnu",
            Os::Windows => "pc-windows-msvc",
        };

        let expected = format!("mermaid-lsp-{arch_str}-{os_str}.zip");

        release
            .assets
            .iter()
            .find(|a| a.name == expected)
            .cloned()
            .ok_or_else(|| {
                let available: Vec<_> = release.assets.iter().map(|a| a.name.as_str()).collect();
                format!("No asset '{expected}' found. Available: {available:?}")
            })
    }

    fn purge_old_cache_versions(extension_dir: &std::path::Path, keep_version: &str) {
        let cache_root = extension_dir.join(CACHE_ROOT);
        if let Ok(entries) = fs::read_dir(&cache_root) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|v| v != keep_version)
                    .unwrap_or(false)
                {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
    }

    fn binary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "mermaid-lsp.exe"
        } else {
            "mermaid-lsp"
        }
    }
}

zed_extension_api::register_extension!(MermaidPreviewExtension);
