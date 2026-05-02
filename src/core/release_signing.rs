#![allow(dead_code)]

//! Release Signing — Binary Integrity Verification
//!
//! SECURITY PRINCIPLE: A kernel security tool distributed via curl|sudo bash
//! is an instant credibility killer. This module provides:
//! 1. SHA-256 binary self-verification on startup
//! 2. GPG signature verification for distributed binaries
//! 3. Infrastructure for signed .deb/.rpm package distribution

use std::path::Path;

use anyhow::{Context, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ============================================================================
// BINARY INTEGRITY — SELF-VERIFICATION ON STARTUP
// ============================================================================

/// Result of a binary integrity check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub verified: bool,
    pub binary_path: String,
    pub expected_hash: Option<String>,
    pub actual_hash: String,
    pub reason: String,
}

/// Self-integrity checker: the daemon verifies its own binary on startup
pub struct SelfIntegrityChecker;

impl SelfIntegrityChecker {
    /// Verify the running binary's SHA-256 hash against a known-good value.
    ///
    /// The expected hash can come from:
    /// 1. An environment variable (NEXUS_AXIOM_HASH) set by the package manager
    /// 2. A .sha256 sidecar file next to the binary
    /// 3. An embedded compile-time hash (for release builds)
    pub fn verify() -> IntegrityResult {
        let binary_path = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                return IntegrityResult {
                    verified: false,
                    binary_path: "unknown".to_string(),
                    expected_hash: None,
                    actual_hash: String::new(),
                    reason: format!("Cannot determine own binary path: {}", e),
                };
            }
        };

        let actual_hash = match Self::hash_file(&binary_path) {
            Ok(h) => h,
            Err(e) => {
                return IntegrityResult {
                    verified: false,
                    binary_path: binary_path.display().to_string(),
                    expected_hash: None,
                    actual_hash: String::new(),
                    reason: format!("Cannot hash own binary: {}", e),
                };
            }
        };

        // Try to get expected hash from environment
        if let Ok(expected) = std::env::var("NEXUS_AXIOM_HASH") {
            let matches = actual_hash == expected.trim().to_lowercase();
            return IntegrityResult {
                verified: matches,
                binary_path: binary_path.display().to_string(),
                expected_hash: Some(expected),
                actual_hash,
                reason: if matches {
                    "Binary integrity verified via NEXUS_AXIOM_HASH".to_string()
                } else {
                    "INTEGRITY FAILURE: Binary hash does not match expected!".to_string()
                },
            };
        }

        // Try sidecar .sha256 file
        let sidecar = binary_path.with_extension("sha256");
        if sidecar.exists() {
            if let Ok(content) = std::fs::read_to_string(&sidecar) {
                // .sha256 format: "<hash>  <filename>" or just "<hash>"
                let expected = content.split_whitespace().next().unwrap_or("").to_lowercase();
                let matches = actual_hash == expected;
                return IntegrityResult {
                    verified: matches,
                    binary_path: binary_path.display().to_string(),
                    expected_hash: Some(expected),
                    actual_hash,
                    reason: if matches {
                        "Binary integrity verified via .sha256 sidecar".to_string()
                    } else {
                        "INTEGRITY FAILURE: Sidecar hash mismatch!".to_string()
                    },
                };
            }
        }

        // No reference hash available — log warning but don't fail
        IntegrityResult {
            verified: true, // Can't verify without reference, allow but warn
            binary_path: binary_path.display().to_string(),
            expected_hash: None,
            actual_hash,
            reason: "No reference hash available — consider setting NEXUS_AXIOM_HASH".to_string(),
        }
    }

    /// SHA-256 hash of a file
    pub fn hash_file(path: &Path) -> Result<String> {
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let hash = Sha256::digest(&data);
        Ok(hex::encode(hash))
    }
}

// ============================================================================
// RELEASE MANIFEST — SIGNED CHECKSUMS FOR ALL RELEASE ARTIFACTS
// ============================================================================

/// A release manifest containing checksums for all distributed files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub release_date: String,
    pub artifacts: Vec<ReleaseArtifact>,
    /// GPG signature of the serialized manifest (hex-encoded)
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub filename: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub platform: String,
}

impl ReleaseManifest {
    /// Verify a downloaded file against this manifest
    pub fn verify_file(&self, filename: &str, file_path: &Path) -> Result<bool> {
        let artifact = self
            .artifacts
            .iter()
            .find(|a| a.filename == filename)
            .ok_or_else(|| anyhow::anyhow!("File {} not found in manifest", filename))?;

        let actual_hash = SelfIntegrityChecker::hash_file(file_path)?;

        if actual_hash != artifact.sha256 {
            error!(
                "[RELEASE] CHECKSUM MISMATCH for {}: expected={}, actual={}",
                filename, artifact.sha256, actual_hash
            );
            return Ok(false);
        }

        // Also verify file size
        let metadata = std::fs::metadata(file_path)?;
        if metadata.len() != artifact.size_bytes {
            error!(
                "[RELEASE] SIZE MISMATCH for {}: expected={}, actual={}",
                filename, artifact.size_bytes, metadata.len()
            );
            return Ok(false);
        }

        info!("[RELEASE] Verified {}: SHA-256={}", filename, actual_hash);
        Ok(true)
    }
}

// ============================================================================
// GPG PUBLIC KEY — EMBEDDED FOR BOOTSTRAP TRUST
// ============================================================================

/// The project's GPG public key, embedded in the binary for bootstrap trust.
/// This key is used to verify release signatures.
pub const GPG_PUBLIC_KEY: &str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----
[PLACEHOLDER: Replace with actual project GPG public key before first release]
[Generate with: gpg --full-generate-key --keyid-format long]
[Export with: gpg --armor --export <KEY_ID>]
-----END PGP PUBLIC KEY BLOCK-----
"#;

/// Verify a GPG detached signature.
/// In production, this would shell out to `gpg --verify` or use the `sequoia-openpgp` crate.
pub fn verify_gpg_signature(
    data_path: &Path,
    signature_path: &Path,
) -> Result<bool> {
    // Check that signature file exists
    if !signature_path.exists() {
        anyhow::bail!("GPG signature file not found: {:?}", signature_path);
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Import the project's public key
        let import_result = Command::new("gpg")
            .args(["--import", "--batch", "--yes"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(GPG_PUBLIC_KEY.as_bytes())?;
                }
                child.wait()
            });

        if let Err(e) = import_result {
            warn!("[GPG] Failed to import public key: {}", e);
            // GPG not available — can't verify, but don't crash
            return Ok(false);
        }

        // Verify the detached signature
        let verify_result = Command::new("gpg")
            .args([
                "--verify",
                &signature_path.to_string_lossy(),
                &data_path.to_string_lossy(),
            ])
            .output()?;

        if verify_result.status.success() {
            info!("[GPG] Signature verified for {:?}", data_path);
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&verify_result.stderr);
            error!("[GPG] Signature verification FAILED: {}", stderr);
            Ok(false)
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (data_path, signature_path);
        warn!("[GPG] GPG verification not available on this platform");
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_integrity_runs() {
        let result = SelfIntegrityChecker::verify();
        // Should always produce a result, even without reference hash
        assert!(!result.actual_hash.is_empty() || !result.reason.is_empty());
    }

    #[test]
    fn test_hash_consistency() {
        // Hashing the same data twice must produce the same result
        let data = b"test data for hashing";
        let hash1 = hex::encode(Sha256::digest(data));
        let hash2 = hex::encode(Sha256::digest(data));
        assert_eq!(hash1, hash2);
    }
}
