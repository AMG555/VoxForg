# Security Policy

## Supported Versions

Only the latest active release and development branches receive security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

The VoxForg development team takes security vulnerabilities seriously. We appreciate the responsible disclosure of security issues by researchers and contributors.

### How to Report

Please do **NOT** report security vulnerabilities via public GitHub issues or public discussion forums.

Instead, submit your report privately via one of the following channels:

- **Security Email:** [amgt503@gmail.com](mailto:amgt503@gmail.com)
- **GitHub Private Vulnerability Reporting:** Open a private advisory under the Security tab of the [VoxForg Repository](https://github.com/AMG555/VoxForg/security/advisories).

### What to Include

To help us triage and resolve your report quickly, please provide:

1. **Description:** Type and nature of the vulnerability (e.g. CWE-208, CWE-79, CWE-22).
2. **Affected Code / Components:** Exact files, lines of code, and git commit hashes.
3. **Proof of Concept (PoC) / Reproduction Steps:** Minimal commands or scripts to reproduce the behavior safely.
4. **Impact:** Potential risks to deployment instances, confidentiality, integrity, or resource consumption.
5. **Remediation Suggestions:** Proposed patches, library recommendations, or architectural fixes if known.

### Response Timeframe & SLA

- **Initial Acknowledgment:** Within 24 hours of receiving your report.
- **Triage & Assessment:** Within 72 hours with an initial severity rating (CVSS v3.1).
- **Remediation & Patching:** High/Critical severity issues are patched with high priority before public disclosure.
- **Public Disclosure / Credit:** Security researchers will be credited in release notes and security advisories unless anonymity is requested.

## Security Baseline & Architecture

VoxForg adheres to an enterprise security baseline:
- **Constant-Time Verification:** Authentication tokens and cryptographic primitives are verified strictly using constant-time comparisons (`subtle::ConstantTimeEq`) to prevent timing side-channel attacks (CWE-208).
- **No Client-Side Secrets:** Third-party credentials, object storage keys, and master API tokens are strictly isolated to server-side configurations and never exposed to the frontend.
- **Input Validation & Sanitization:** All incoming requests, audio synthesis inputs, and SSML payloads are bounded and validated with strict schemas.
- **Strict Network Headers:** Automatic enforcement of `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Content-Security-Policy`, and strict CORS policies.
