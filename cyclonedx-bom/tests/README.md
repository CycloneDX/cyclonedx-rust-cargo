Following best practices from software engineering, we choose an approach to testing 
covering verification and validation.

1. Verification: testing against the CycloneDX specification.
2. Validation: testing against real world SBOMs generated by available tools.

# Verification

The CycloneDX project provides a specification of its BOM format in a dedicated 
[repository](https://github.com/CycloneDX/specification). The specification is defined 
in [schemas](https://github.com/CycloneDX/specification/tree/master/schema) 
of different formats, including JSON and XML.

Within that repository there is also 
[test data](https://github.com/CycloneDX/specification/tree/master/tools/src/test/resources) 
for all released versions.

## Preparing

Take the following steps to use test data provided by CycloneDX for verification:

* create a folder in tests/data/VERSION to hold the test data.
* within that folder, create another folder called not_yet_supported to limit the verification's scope.
* copy the test data to tests/data/VERSION.
* in folder test, create a specification_tests_vX_Y.rs, to implement the tests used for verification.

## Testing

Run verification of all versions:

```
cargo test --test 'spec*'
```

To verify only a specific version run, setting X and Y appropriately:

```
cargo test --test 'spec*vX_Y'
``` 

# Validation

The CycloneDX project also provides various BOM examples in a dedicated 
[repository](https://github.com/CycloneDX/bom-examples). The repository covers various BOM use cases and 
offers BOMs in different, partly outdated, versions. Unfortunately, most of the BOMs have been created 
in a non-reproducible way by different CycloneDX tools.

Anyway, the following folders are useful for validation tests because BOMs of up-to-date 
versions are included:

* SBOM/juice-shop/via_npm/bare
* SBOM/juice-shop/via_npm/flat
* SBOM/laravel-7.12.0
* SaaSBOM/apigateway-microservices-datastores
* OBOM/Example-1-Decoupled

In addition, BOMs generated by 3rd party tools are used for validation. A popular example is Aquasec's trivy. 
Digesting BOMs from 3rd party tools comes with some challenges. However, to ensure interoperability and 
identify issues early, it is worth the effort.

## Preparing

Take the following steps to use valid or invalid BOMs for validation (VERSION is in format X.Y):

* create a folder in tests/examples/VERSION to hold the test data.
* within that folder, create another folder called not_yet_supported to limit the validation's scope.
* copy the test data to tests/examples/VERSION.
* in folder test, create a examples_tests_vX_Y.rs, to implement the tests used for validation.

## Testing

Run validation of all versions:

```
cargo test --test 'ex*'
```

To validate only a specific version run, setting X and Y appropriately:

```
cargo test --test 'ex*vX_Y'
```

## Create sample SBOMs with trivy

Trivy used to generate license strings invalid from an SPDX perspective. This also affected 
the CycloneDX Format.

This issue was fixed in 0.38.3.

So, 0.38.2, is expected to generate invalid SBOMs, while 0.38.3 should be okay.

The tracking issue was: https://github.com/aquasecurity/trivy/issues/3267.

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

trivy --format cyclonedx --timeout 10m --debug image cyclonedx/cyclonedx-bom-repo-server:4.0.0 > valid-bom-1.4_aquasec-trivy-0.44.1_cyclonedx-bom-repo-server-4.0.0.cyclonedx

trivy image python:3.4-alpine

trivy image --format json --output result.json alpine:3.15

SBOM_VERSION="1.4"
TRIVY_VERSION="0.42.1"
IMAGE="alpine:3.15"
PRODUCT="alpine"
PRODUCT_VERSION="3.15"
FILENAME="xvalid_sbom-${SBOM_VERSION}_trivy-${TRIVY_VERSION}_${PRODUCT}-${PRODUCT_VERSION}.cdx.json"
TRIVY_BIN="./trivy"

# Set used scanners / checks explicitly
# Note: 'vuln' is required to scan for vulnerabilities when using the CycloneDX format

# Including 0.36.1 this option is used:
TRIVY_OPTIONS="--security-checks vuln,config,license"

# In 0.38.2, this option is used:
TRIVY_OPTIONS="--scanners vuln,config,license"

$TRIVY_BIN image --format cyclonedx --output $FILENAME $TRIVY_OPTIONS $IMAGE

SBOM_VERSION="1.4"
TRIVY_VERSION="0.36.1"
IMAGE="alpine:3.13.1"
PRODUCT="alpine"
PRODUCT_VERSION="3.13.1"
FILENAME="xvalid_sbom-${SBOM_VERSION}_trivy-${TRIVY_VERSION}_${PRODUCT}-${PRODUCT_VERSION}.cdx.json"
TRIVY_BIN="./trivy"
TRIVY_OPTIONS="--security-checks vuln,config,license"
$TRIVY_BIN image --format cyclonedx --output $FILENAME $TRIVY_OPTIONS $IMAGE

SBOM_VERSION="1.4"
TRIVY_VERSION="0.36.1"
IMAGE="ruby:2.4.10-slim"
PRODUCT="ruby"
PRODUCT_VERSION="2.4.10-slim"
FILENAME="xvalid_sbom-${SBOM_VERSION}_trivy-${TRIVY_VERSION}_${PRODUCT}-${PRODUCT_VERSION}.cdx.json"
TRIVY_BIN="./trivy"
TRIVY_OPTIONS="--security-checks vuln,config,license"
$TRIVY_BIN image --format cyclonedx --output $FILENAME $TRIVY_OPTIONS $IMAGE

# Todos

* Some URLs in valid-patch-1.4.xml / valid-patch-1.4.json are invalid.


# Links

https://github.com/CycloneDX/specification

https://github.com/CycloneDX/specification/tree/master/tools/src/test/resources
