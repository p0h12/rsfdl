// Generates encrypted SFDL test fixture files.
// Run: go run tools/generate_fixture.go [filelist|bulkfolder]
//   filelist   — v3 with BulkFolderMode=false + FileList (default)
//   bulkfolder — v3 with BulkFolderMode=true + BulkFolderList
package main

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/md5"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"os"
)

func encrypt(password, plaintext string) string {
	hasher := md5.New()
	hasher.Write([]byte(password))
	key := hasher.Sum(nil)

	block, err := aes.NewCipher(key)
	if err != nil {
		panic(err)
	}

	plaintextBytes := pkcs7Pad([]byte(plaintext), aes.BlockSize)

	ciphertext := make([]byte, aes.BlockSize+len(plaintextBytes))
	iv := ciphertext[:aes.BlockSize]
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		panic(err)
	}

	mode := cipher.NewCBCEncrypter(block, iv)
	mode.CryptBlocks(ciphertext[aes.BlockSize:], plaintextBytes)

	return base64.StdEncoding.EncodeToString(ciphertext)
}

func pkcs7Pad(data []byte, blockSize int) []byte {
	padding := blockSize - len(data)%blockSize
	padded := make([]byte, len(data)+padding)
	copy(padded, data)
	for i := len(data); i < len(padded); i++ {
		padded[i] = byte(padding)
	}
	return padded
}

func printHeader(pw string, description string) {
	fmt.Println(`<?xml version="1.0" encoding="utf-8"?>`)
	fmt.Println(`<Container>`)
	fmt.Println(`  <ContainerVersion>10</ContainerVersion>`)
	fmt.Printf("  <Description>%s</Description>\n", encrypt(pw, description))
	fmt.Printf("  <Uploader>%s</Uploader>\n", encrypt(pw, "testuser"))
	fmt.Println(`  <Encrypted>true</Encrypted>`)
	fmt.Println(`  <MaxDownloadThreads>3</MaxDownloadThreads>`)
	fmt.Println(`  <Connection>`)
	fmt.Printf("    <Host>%s</Host>\n", encrypt(pw, "ftp.example.com"))
	fmt.Println(`    <Port>21</Port>`)
	fmt.Printf("    <Username>%s</Username>\n", encrypt(pw, "ftpuser"))
	fmt.Printf("    <Password>%s</Password>\n", encrypt(pw, "ftppass"))
	fmt.Println(`    <AuthRequired>true</AuthRequired>`)
	fmt.Println(`    <DataConnectionType>Passive</DataConnectionType>`)
	fmt.Println(`    <DataType>Binary</DataType>`)
	fmt.Println(`    <CharacterEncoding>UTF8</CharacterEncoding>`)
	fmt.Println(`    <SSLProtocol>None</SSLProtocol>`)
	fmt.Println(`    <ConnectTimeout>10</ConnectTimeout>`)
	fmt.Println(`    <CommandTimeout>10</CommandTimeout>`)
	fmt.Println(`  </Connection>`)
}

func printFooter() {
	fmt.Println(`  </Packages>`)
	fmt.Println(`</Container>`)
}

func generateFileList(pw string) {
	printHeader(pw, "Test.Release.2026.1080p")
	fmt.Println(`  <Packages>`)
	fmt.Println(`    <Package>`)
	fmt.Printf("      <Name>%s</Name>\n", encrypt(pw, "Package1"))
	fmt.Println(`      <BulkFolderMode>false</BulkFolderMode>`)
	fmt.Println(`      <FileList>`)
	fmt.Println(`        <FileItem>`)
	fmt.Printf("          <FileName>%s</FileName>\n", encrypt(pw, "movie.part1.rar"))
	fmt.Printf("          <DirectoryRoot>%s</DirectoryRoot>\n", encrypt(pw, "/"))
	fmt.Printf("          <DirectoryPath>%s</DirectoryPath>\n", encrypt(pw, "releases/test"))
	fmt.Printf("          <FullPath>%s</FullPath>\n", encrypt(pw, "/releases/test/movie.part1.rar"))
	fmt.Println(`          <FileSize>104857600</FileSize>`)
	fmt.Println(`          <HashType>MD5</HashType>`)
	fmt.Println(`          <FileHash>d41d8cd98f00b204e9800998ecf8427e</FileHash>`)
	fmt.Printf("          <PackageName>%s</PackageName>\n", encrypt(pw, "Package1"))
	fmt.Println(`        </FileItem>`)
	fmt.Println(`        <FileItem>`)
	fmt.Printf("          <FileName>%s</FileName>\n", encrypt(pw, "movie.part2.rar"))
	fmt.Printf("          <DirectoryRoot>%s</DirectoryRoot>\n", encrypt(pw, "/"))
	fmt.Printf("          <DirectoryPath>%s</DirectoryPath>\n", encrypt(pw, "releases/test"))
	fmt.Printf("          <FullPath>%s</FullPath>\n", encrypt(pw, "/releases/test/movie.part2.rar"))
	fmt.Println(`          <FileSize>52428800</FileSize>`)
	fmt.Println(`          <HashType>None</HashType>`)
	fmt.Println(`          <FileHash></FileHash>`)
	fmt.Printf("          <PackageName>%s</PackageName>\n", encrypt(pw, "Package1"))
	fmt.Println(`        </FileItem>`)
	fmt.Println(`      </FileList>`)
	fmt.Println(`      <BulkFolderList />`)
	fmt.Println(`    </Package>`)
	printFooter()
}

func generateBulkFolder(pw string) {
	printHeader(pw, "BulkFolder.Test.2026")
	fmt.Println(`  <Packages>`)
	fmt.Println(`    <Package>`)
	fmt.Printf("      <Name>%s</Name>\n", encrypt(pw, "BulkPkg1"))
	fmt.Println(`      <BulkFolderMode>true</BulkFolderMode>`)
	fmt.Println(`      <FileList />`)
	fmt.Println(`      <BulkFolderList>`)
	fmt.Println(`        <BulkFolder>`)
	fmt.Printf("          <BulkFolderPath>%s</BulkFolderPath>\n", encrypt(pw, "/releases/movie/"))
	fmt.Printf("          <PackageName>%s</PackageName>\n", encrypt(pw, "BulkPkg1"))
	fmt.Println(`        </BulkFolder>`)
	fmt.Println(`        <BulkFolder>`)
	fmt.Printf("          <BulkFolderPath>%s</BulkFolderPath>\n", encrypt(pw, "/releases/extras/"))
	fmt.Printf("          <PackageName>%s</PackageName>\n", encrypt(pw, "BulkPkg1"))
	fmt.Println(`        </BulkFolder>`)
	fmt.Println(`      </BulkFolderList>`)
	fmt.Println(`    </Package>`)
	printFooter()
}

func main() {
	mode := "filelist"
	if len(os.Args) > 1 {
		mode = os.Args[1]
	}

	pw := "test"

	switch mode {
	case "filelist":
		generateFileList(pw)
	case "bulkfolder":
		generateBulkFolder(pw)
	default:
		fmt.Fprintf(os.Stderr, "Usage: %s [filelist|bulkfolder]\n", os.Args[0])
		os.Exit(1)
	}
}
