# Code Signing & Verified Publisher Guide for VoxForg

## Why Windows Shows "Publisher: Unknown" & SmartScreen Warnings

When users download `voxforg-windows-x86_64.exe` or `VoxForg-Setup.exe`, Windows Defender SmartScreen and web browsers (Edge, Chrome) flag:
- **"Publisher: Unknown"**
- **"voxforg-windows-x86_64.exe isn't commonly downloaded"**
- **"Windows protected your PC"**

### How Windows Trust Actually Works:
1. **File Metadata Strings (CompanyName, etc.) are unverified**: Windows allows any file to claim any company name in its metadata. Therefore, Windows security systems **completely ignore** text strings inside the executable.
2. **Authenticode Cryptographic Signatures**: Windows requires an Authenticode digital signature issued by a trusted Certificate Authority (CA). Without this signature, Windows displays `Publisher: Unknown`.
3. **SmartScreen Cloud Reputation**: SmartScreen builds reputation over time based on:
   - Certificate reputation (EV certificates give instant trust; OV certificates build trust quickly).
   - Global download volume and telemetry across millions of Windows machines.

---

## 3 Paths to Verified Publisher & Zero Warnings

### Option 1: SignPath.io (100% Free for Open Source — Recommended)

The **SignPath Foundation** provides free code-signing certificates to qualifying open-source projects. Major open source projects (Greenshot, Git Extensions, etc.) use SignPath.

#### Steps to Setup:
1. Go to [about.signpath.io/open-source](https://about.signpath.io/open-source).
2. Click **Apply for free code signing**.
3. Provide your repository link: `https://github.com/AMG555/VoxForg`.
4. Once approved, SignPath provides a project token and organization ID.
5. In GitHub repo **Settings -> Secrets and variables -> Actions**, add:
   - `SIGNPATH_API_TOKEN`
   - `SIGNPATH_ORGANIZATION_ID`
   - `SIGNPATH_PROJECT_SLUG`
6. Release binaries built via GitHub Actions are signed automatically during release.

---

### Option 2: Submit to Windows Package Manager (`winget`) (100% Free & Immediate Trust)

Submitting VoxForg to Microsoft's official `winget-pkgs` catalog bypasses browser download warnings entirely. Users can install VoxForg via PowerShell or Terminal:

```powershell
winget install VoxForg
```

#### Why Winget Works So Well:
- Microsoft scans and verifies the installer in their cloud sandbox during PR review.
- No browser SmartScreen download warnings.
- Users trust `winget` installations.
- Discoverable by millions of Windows developers.

#### How to Submit:
1. Install the official `wingetcreate` tool:
   ```powershell
   winget install Microsoft.WingetCreate
   ```
2. Submit the latest release installer:
   ```powershell
   wingetcreate new https://github.com/AMG555/VoxForg/releases/download/v0.1.0/VoxForg-Setup-v0.1.0-windows-x64.exe
   ```
3. `wingetcreate` will automatically download the binary, calculate the SHA-256 hash, generate the YAML manifests, and submit a Pull Request to `microsoft/winget-pkgs`.

---

### Option 3: Standard Code Signing Certificate (Paid)

If you wish to hold an independent certificate:
- **Certum Open Source Code Signing**: ~$28–$50/year (requires developer identity verification).
- **Standard Commercial Code Signing (OV/EV)**: DigiCert, Sectigo, SSL.com ($200–$400/year).

#### Adding Certificate to GitHub Actions:
Once you have an exportable `.pfx` certificate file:
1. Base64 encode the `.pfx` file:
   ```powershell
   [Convert]::ToBase64String([IO.File]::ReadAllBytes("certificate.pfx")) | Set-Clipboard
   ```
2. Add secrets to GitHub repository:
   - `WINDOWS_CERT_BASE64`: Paste the Base64 string.
   - `WINDOWS_CERT_PASSWORD`: Certificate password.
3. The VoxForg release workflow automatically signs all `.exe` binaries and installers on every new release.
