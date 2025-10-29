```diff
diff --git a/.github/workflows/security.yml b/.github/workflows/security.yml
@@ sbom job
-      - name: Generate CycloneDX SBOM
-        run: cargo cyclonedx --all --all-features --format xml --output-file sbom.xml
+      - name: Generate CycloneDX SBOM
+        run: cargo cyclonedx --workspace --all-features --format xml --output sbom.xml
```

## Action pinning suggestions (do not edit workflows yet)
- .github/workflows/ci-matrix.yml:18 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/ci-matrix.yml:21 — pin `dtolnay/rust-toolchain@master` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/ci.yml:14 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/ci.yml:22 — pin `dtolnay/rust-toolchain@master` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/codeql.yml:20 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/codeql.yml:23 — pin `github/codeql-action/init@v3` to a commit SHA (`github/codeql-action/init@<sha>`) for reproducibility.
- .github/workflows/codeql.yml:28 — pin `github/codeql-action/autobuild@v3` to a commit SHA (`github/codeql-action/autobuild@<sha>`) for reproducibility.
- .github/workflows/codeql.yml:31 — pin `github/codeql-action/analyze@v3` to a commit SHA (`github/codeql-action/analyze@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:21 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:44 — pin `actions/upload-artifact@v4` to a commit SHA (`actions/upload-artifact@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:65 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:71 — pin `actions/download-artifact@v4` to a commit SHA (`actions/download-artifact@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:77 — pin `actions/setup-python@v5` to a commit SHA (`actions/setup-python@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:101 — pin `dtolnay/rust-toolchain@stable` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/regenerate-fixtures.yml:192 — pin `actions/upload-artifact@v4` to a commit SHA (`actions/upload-artifact@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:21 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:73 — pin `actions/upload-artifact@v4` to a commit SHA (`actions/upload-artifact@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:104 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:110 — pin `actions/download-artifact@v4` to a commit SHA (`actions/download-artifact@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:116 — pin `actions/setup-python@v5` to a commit SHA (`actions/setup-python@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:140 — pin `dtolnay/rust-toolchain@stable` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/regenerate-goldens.yml:215 — pin `actions/upload-artifact@v4` to a commit SHA (`actions/upload-artifact@<sha>`) for reproducibility.
- .github/workflows/security.yml:16 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/security.yml:17 — pin `dtolnay/rust-toolchain@stable` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/security.yml:26 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/security.yml:27 — pin `dtolnay/rust-toolchain@stable` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/security.yml:40 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/security.yml:44 — pin `gitleaks/gitleaks-action@v2` to a commit SHA (`gitleaks/gitleaks-action@<sha>`) for reproducibility.
- .github/workflows/security.yml:54 — pin `actions/checkout@v4` to a commit SHA (`actions/checkout@<sha>`) for reproducibility.
- .github/workflows/security.yml:55 — pin `dtolnay/rust-toolchain@stable` to a commit SHA (`dtolnay/rust-toolchain@<sha>`) for reproducibility.
- .github/workflows/security.yml:61 — pin `actions/upload-artifact@v4` to a commit SHA (`actions/upload-artifact@<sha>`) for reproducibility.
