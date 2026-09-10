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

1. **Description:** Type and nature of the vulnerability (e.g. CWE-208, CWE-918, CWE-79, CWE-400).
2. **Affected Code / Components:** Exact files, lines of code, and git commit hashes.
3. **Proof of Concept (PoC) / Reproduction Steps:** Minimal commands or scripts to reproduce the behavior safely.
4. **Impact:** Potential risks to deployment instances, confidentiality, integrity, or resource consumption.
5. **Remediation Suggestions:** Proposed patches, library recommendations, or architectural fixes if known.

### Response Timeframe & SLA

- **Initial Acknowledgment:** Within 24 hours of receiving your report.
- **Triage & Assessment:** Within 72 hours with an initial severity rating (CVSS v3.1).
- **Remediation & Patching:** High/Critical severity issues are patched with high priority before public disclosure.
- **Public Disclosure / Credit:** Security researchers will be credited in release notes and security advisories unless anonymity is requested.

---

## Security Baseline & Architecture

VoxForg adheres to an enterprise security baseline:

### 1. Server-Side Request Forgery (SSRF) Prevention (CWE-918)
- **Cloud Metadata Blocking:** The model router (`OpenAiRouterEngine`) unconditionally rejects all requests targeting cloud instance metadata services (`169.254.169.254`, `metadata.google.internal`, link-local IPv6).
- **Private Subnet Restrictions:** Requests targeting RFC 1918 private IPv4 subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`) or loopback IPv6 are rejected by default unless explicit permission is granted via `--allow-private-ips`.
- **Scheme Enforcement:** Only `http` and `https` schemes are permitted; internal schemes (`file://`, `gopher://`, `dict://`) are strictly rejected.

### 2. Secret Masking & Redaction (CWE-532)
- Upstream router API keys and Bearer tokens are scrubbed from all logging, traces, and metrics output (`sk-***1234`).
- Error payloads and RFC 7807 `ProblemDetails` never echo sensitive authorization headers or credential tokens back to the caller.

### 3. Denial of Service & Resource Caps (CWE-400)
- **Request Body Limits:** Enforced 2MB maximum payload limit (`DefaultBodyLimit::max(2 * 1024 * 1024)`) across all API routes to prevent memory exhaustion.
- **Input Text Bounding:** Synthesis requests are capped at 10,000 characters per request, returning RFC 7807 `ProblemDetails` with HTTP 422 Unprocessable Entity if violated.
- **Speed & Pitch Bounds:** Floating-point parameters are strictly clamped (`speed`: 0.25 to 4.0; `pitch`: -50.0 to 50.0 st) to prevent NaN/infinite filter instability.
- **Circuit Breakers:** Upstream routing failures trigger an automated circuit breaker after 5 consecutive errors, protecting against thread pool depletion and cascading network delays.

### 4. Constant-Time Authentication Verification (CWE-208)
- API Bearer tokens are verified strictly using constant-time comparisons (`subtle::ConstantTimeEq`) to eliminate timing side-channel attacks.

### 5. Strict Security Headers
- Automatic header enforcement across all HTTP responses:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY` (or `SAMEORIGIN` for embedded `/docs`)
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `Content-Security-Policy: default-src 'self'` (with tightly scoped CDNs for Scalar API documentation)
  - `Referrer-Policy: strict-origin-when-cross-origin`
