# SFDL File Format Specification

SFDL files are XML-based containers that store FTP connection details and file lists.
Fields can be optionally encrypted with AES-128-CBC.

## Version Detection

Detection uses the XML element name, then the numeric value:

| ContainerVersion | SFDL Version | Status |
|------------------|-------------|--------|
| 0 | ungueltig | Fehler |
| 1–5 | v1 | nicht unterstuetzt |
| 6–9 | v2 (legacy) | wird zu v3 konvertiert |
| **10** | **v3 (aktuell)** | unterstuetzt |
| >10 | ungueltig | Fehler |

- **v3**: XML-Element `<ContainerVersion>` vorhanden (Wert = 10)
- **v2**: XML-Element `<SFDLFileVersion>` vorhanden (Wert = 6–9)
- Quelle: SFDL.NET `MainViewModel.vb` Select Case

## v3 Format (ContainerVersion=10)

```xml
<?xml version="1.0" encoding="utf-8"?>
<Container>
    <ContainerVersion>10</ContainerVersion>
    <Description>[encrypted if Encrypted=true]</Description>
    <Uploader>[encrypted]</Uploader>
    <Encrypted>true/false</Encrypted>
    <MaxDownloadThreads>3</MaxDownloadThreads>

    <Connection>
        <Host>[encrypted]</Host>
        <Port>21</Port>
        <Username>[encrypted]</Username>
        <Password>[encrypted]</Password>
        <AuthRequired>true/false</AuthRequired>
        <DataConnectionType>Passive|Active|ExtendedPassive</DataConnectionType>
        <DataType>Binary|ASCII</DataType>
        <CharacterEncoding>Standard|ASCII|UTF7|UTF8</CharacterEncoding>
        <SSLProtocol>None|Tls|Tls11|Tls12|Ssl2|Ssl3</SSLProtocol>
        <ConnectTimeout>10</ConnectTimeout>
        <CommandTimeout>10</CommandTimeout>
    </Connection>

    <Packages>
        <Package>
            <Name>[encrypted]</Name>
            <BulkFolderMode>false</BulkFolderMode>

            <FileList>
                <FileItem>
                    <FileName>[encrypted]</FileName>
                    <DirectoryRoot>[encrypted]</DirectoryRoot>
                    <DirectoryPath>[encrypted]</DirectoryPath>
                    <FullPath>[encrypted]</FullPath>
                    <FileSize>1024000</FileSize>
                    <HashType>MD5|CRC|SHA1|default</HashType>
                    <FileHash>[hash value]</FileHash>
                    <PackageName>[encrypted]</PackageName>
                </FileItem>
            </FileList>

            <BulkFolderList>
                <BulkFolder>
                    <BulkFolderPath>[encrypted]</BulkFolderPath>
                    <PackageName>[encrypted]</PackageName>
                </BulkFolder>
            </BulkFolderList>
        </Package>
    </Packages>
</Container>
```

## v2 Format (Legacy)

```xml

<SFDLFile>
    <Description>...</Description>
    <Uploader>...</Uploader>
    <SFDLFileVersion>1.0</SFDLFileVersion>
    <Encrypted>true/false</Encrypted>
    <ConnectionInfo>
        <Host>...</Host>
        <Port>21</Port>
        <Username>...</Username>
        <Password>...</Password>
    </ConnectionInfo>
    <Packages>
        <SFDLPackage>
            <BulkFolderList>
                <BulkFolder>
                    <BulkFolderPath>/path/to/files</BulkFolderPath>
                </BulkFolder>
            </BulkFolderList>
        </SFDLPackage>
    </Packages>
</SFDLFile>
```

## Encryption

- **Algorithm**: AES-128-CBC (Rijndael)
- **Key derivation**: `MD5(password_utf8_bytes)` → 16-byte key
- **IV**: First 16 bytes of Base64-decoded ciphertext
- **Ciphertext**: Remaining bytes after IV extraction
- **Padding**: PKCS7
- **Encoding**: Base64

### Encrypted fields (v3)

- `Description`, `Uploader`
- `Connection.Host`, `Connection.Username`, `Connection.Password`
- `Package.Name`
- `FileItem.FileName`, `FileItem.DirectoryRoot`, `FileItem.DirectoryPath`, `FileItem.FullPath`, `FileItem.PackageName`
- `BulkFolder.BulkFolderPath`, `BulkFolder.PackageName`

### Decryption pseudocode

```
function decrypt(ciphertext_b64, password):
    key = MD5(password.encode("utf-8"))     // 16 bytes
    decoded = base64_decode(ciphertext_b64)
    iv = decoded[0..16]
    ciphertext = decoded[16..]
    plaintext = AES_CBC_decrypt(ciphertext, key, iv, PKCS7)
    return plaintext.decode("utf-8")
```

### Password validation

Try decrypting the `Host` field. If result looks like a valid hostname/IP, the password is correct.

### Encoding note

The canonical VB.NET implementation uses UTF-8 for MD5 key derivation.
Some Python implementations use Latin-1. For maximum compatibility, try UTF-8 first, then fallback to Latin-1.

## Reference implementations

- [SFDL.Container](https://github.com/n0ix/SFDL.Container) — VB.NET: Canonical data models + encryption
- [SFDL.NET](https://github.com/n0ix/SFDL.NET) — VB.NET/WPF: Official Windows app
- [goSFDLSauger](https://github.com/DoctorW00/goSFDLSauger) — Go: Clean v2 parser + crypto reference
- [pySFDLSauger](https://github.com/DoctorW00/pySFDLSauger) — Python: Simple downloader
- [SFDLSaugerCLI](https://github.com/DoctorW00/SFDLSaugerCLI) — C++/Qt: CLI downloader
- [SFDLSaugerPro](https://github.com/DoctorW00/SFDLSaugerPro) — C++/Qt: Full GUI app
- [sfdl-medialoader](https://github.com/efc5c264/sfdl-medialoader) — Python: Media intelligence + TMDB
