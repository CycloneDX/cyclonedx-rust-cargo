Describes anything about testing this crate.

# Examples

## Generate example SBOMs

### Aquasec Trivy

Check Trivy's version:

```
$ trivy --version
Version: 0.43.1
Vulnerability DB:
  Version: 2
  UpdatedAt: 2023-08-31 18:18:08.009329343 +0000 UTC
  NextUpdate: 2023-09-01 00:18:08.009328743 +0000 UTC
  DownloadedAt: 2023-08-31 18:51:48.47415409 +0000 UTC
```

Create an SBOM:

```
$ trivy image --scanners vuln ruby:2.4.0
2023-08-31T20:49:42.246+0200	INFO	Need to update DB
2023-08-31T20:49:42.246+0200	INFO	DB Repository: ghcr.io/aquasecurity/trivy-db
2023-08-31T20:49:42.246+0200	INFO	Downloading DB...
2023-08-31T20:56:02.539+0200	INFO	Vulnerability scanning is enabled
```

# Todos

* Some URLs in valid-patch-1.4.xml / valid-patch-1.4.json are invalid.
