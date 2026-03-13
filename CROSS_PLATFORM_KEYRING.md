# Cross-Platform Keyring Testing Guide

This document explains how the keychain storage works across different platforms and how to verify functionality on each platform.

## Platform-Specific Implementations

The `keyring` crate provides cross-platform keychain access using platform-native APIs:

### Windows (Credential Manager)

**Storage Location**: Windows Credential Manager
**Entry Format**: `Target=com.chamber.app_anthropic_credential`
**Access Method**: Win32 CredRead/CredWrite APIs

**Verification**:
```powershell
# List Chamber credentials
cmdkey /list | findstr chamber

# Remove specific credential (for testing)
cmdkey /delete:Target:com.chamber.app_anthropic_credential
```

**File Location**: Credentials are stored in the Windows Credential Manager database:
- Encrypted using DPAPI (Data Protection API)
- Tied to Windows user account
- Survives system restarts
- Backed up if credential history is enabled

### macOS (Keychain Access)

**Storage Location**: Keychain Access (login keychain by default)
**Entry Format**:
- Service Name: `com.chamber.app`
- Account Name: `anthropic_credential`
- Where: `com.chamber.app` (service)

**Verification**:
```bash
# List all Chamber credentials in keychain
security find-generic-password -s "com.chamber.app" 2>/dev/null | head

# Search for specific provider credential
security find-generic-password -s "com.chamber.app" -a "anthropic_credential"

# Delete specific credential (for testing)
security delete-generic-password -s "com.chamber.app" -a "anthropic_credential"
```

**File Location**: `~/Library/Keychains/login.keychain-db` (or login.keychain)

**GUI Access**:
- Open Keychain Access.app
- Search for "com.chamber.app"
- Can view, modify, or delete credentials

### Linux (Secret Service)

**Storage Location**: libsecret (gnome-keyring or kwallet)
**Entry Format**:
- Collection: "default" or "login"
- Label: `com.chamber.app_anthropic_credential`
- Attributes: service=`com.chamber.app`, username=`anthropic_credential`

**Requirements**:
```bash
# Ubuntu/Debian
sudo apt-get install libsecret-1-0 gnome-keyring

# Fedora
sudo dnf install libsecret gnome-keyring

# Arch
sudo pacman -S libsecret gnome-keyring
```

**Verification**:
```bash
# Using secret-tool (part of libsecret)
secret-tool search --all service com.chamber.app

# List all credentials
secret-tool search --all service com.chamber.app | grep -A 10 "anthropic"

# Delete specific credential (for testing)
secret-tool clear --service com.chamber.app --anthropic_credential
```

**GUI Access**:
- Seahorse (Passwords and Keys)
- GNOME Keyring (Seahorse in newer versions)

## Platform-Specific Testing

### Testing Checklist

Run these tests on each platform to verify keychain functionality:

#### 1. Credential Storage
- [ ] Create credential using settings UI
- [ ] Verify credential appears in platform keychain
- [ ] Restart application
- [ ] Verify credential is still accessible
- [ ] Attempt to create duplicate credential (should handle gracefully)

#### 2. Credential Retrieval
- [ ] Load credentials on application startup
- [ ] Verify correct credentials are retrieved for each provider
- [ ] Test with multiple providers configured
- [ ] Test with no credentials configured

#### 3. Credential Deletion
- [ ] Delete credential from settings UI
- [ ] Verify credential is removed from keychain
- [ ] Restart application and verify it's not recreated

#### 4. OAuth Token Storage
- [ ] Complete OAuth flow
- [ ] Verify tokens stored in keychain (access + refresh)
- [ ] Verify token expiry information stored
- [ ] Test token refresh (wait 5 minutes or force refresh)

#### 5. Migration from .env
- [ ] Run migration tool
- [ ] Verify credentials moved to keychain
- [ ] Verify .env file still exists (not deleted)
- [ ] Test application with migrated credentials

### Platform-Specific Tests

#### Windows Tests
```powershell
# Test credential manager access
$keyring = [System.DirectoryServices.DirectorySearcher]::new("SearchPath")
# Note: This requires elevated privileges

# Simpler test: Check if process can access credential manager
rundll32.exe keymgr.dll,KRShowKeyMgr
```

#### macOS Tests
```bash
# Test if keychain is accessible
security dump-keychain login.keychain-db | grep chamber

# Test read/write (as current user)
echo "test" | security add-generic-password -s "com.chamber.app" -a "test_user" -w -
security find-generic-password -s "com.chamber.app" -a "test_user" -w
security delete-generic-password -s "com.chamber.app" -a "test_user"
```

#### Linux Tests
```bash
# Check if secret service is running
ps aux | grep -i keyring

# Test secret-tool availability
which secret-tool

# Test basic functionality
echo "test" | secret-tool store --service com.chamber.app --key test --label test
secret-tool lookup --service com.chamber.app --key test
secret-tool clear --service com.chamber.app --key test
```

## Known Platform Issues

### Windows
- **Issue**: Some antivirus software may block keyring access
- **Solution**: Add exception for Chamber or disable during testing
- **Issue**: Credential Manager may prompt for access
- **Solution**: User must grant permission

### macOS
- **Issue**: Keychain Access may prompt for permission
- **Solution**: Click "Always Allow" when prompted
- **Issue**: Credentials may go to iCloud keychain instead of local
- **Solution**: Check Keychain Access settings to ensure local storage

### Linux
- **Issue**: No keyring daemon running
- **Solution**: Install gnome-keyring or kwallet
- **Issue**: Multiple keyring backends
- **Solution**: Ensure one is properly configured
- **Issue**: Headless environments
- **Solution**: Use environment variables instead

## CI/CD Considerations

For automated testing across platforms:

```yaml
# Example GitHub Actions
test-windows:
  runs-on: windows-latest
  steps:
    - run: cargo test --package chamber

test-macos:
  runs-on: macos-latest
  steps:
    - run: cargo test --package chamber

test-linux:
  runs-on: ubuntu-latest
  steps:
    - run: sudo apt-get install -y libsecret-1-0 gnome-keyring
    - run: cargo test --package chamber
```

## Debugging Keyring Issues

### Enable Keyring Debug Logging

```rust
// In development, set RUST_LOG environment variable
env RUST_LOG=keyring=debug cargo test
```

### Check Keyring Backend

```bash
# Linux: Check which backend is being used
secret-tool backend check

# This will show if libsecret is properly configured
```

### Test Without Keyring

For testing purposes, you can temporarily disable keychain storage:

```rust
// In credential_manager.rs, use mock storage
#[cfg(test)]
pub struct MockCredentialManager {
    // In-memory storage for tests
}
```

## Security Considerations

### Keyring Encryption
- **Windows**: DPAPI with user-specific key
- **macOS**: AES-128 in Keychain with system master key
- **Linux**: libsecret with GNOME Keyring encryption

### Access Control
- Credentials only accessible to the user who stored them
- Encrypted at rest using platform-native encryption
- Requires system authentication to access (on some platforms)

### Backup Considerations
- **Windows**: Included in Windows Backup
- **macOS**: Included in Time Machine (Keychain included)
- **Linux**: Depends on keyring backend backup configuration

## Testing Commands

### Run All Tests
```bash
cd src-tauri
cargo test
```

### Run Keyring-Specific Tests
```bash
cargo test --package chamber credential
```

### Run with Logging
```bash
RUST_LOG=debug cargo test --package chamber credential
```

### Integration Test
```bash
# Build and run application
npm run tauri:dev

# Test credential flow manually
# 1. Open settings
# 2. Click "Connect with OAuth" for a provider
# 3. Complete authorization
# 4. Verify in platform keychain that credential exists
# 5. Restart application
# 6. Verify credential persists
```

## Platform Verification Status

| Platform | Keyring Backend | Tested | Status |
|----------|-----------------|---------|---------|
| Windows 10/11 | Win32 CredAPI | ⏳ Pending | Not yet verified |
| macOS 12+  | Keychain Framework | ⏳ Pending | Not yet verified |
| Ubuntu 22.04| libsecret/gnome-keyring | ⏳ Pending | Not yet verified |
| Fedora 38+ | libsecret/gnome-keyring | ⏳ Pending | Not yet verified |

To update this table, run the verification steps on each platform and update the status.
