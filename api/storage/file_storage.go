package storage

import (
	"os"
	"path/filepath"
)

type DiskStorage struct {
}

func (d DiskStorage) Save(fileName string, data []byte) (string, error) {
	dir, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	imagesDirPath := filepath.Join(dir, "image-process")
	if err := os.MkdirAll(imagesDirPath, os.ModePerm); err != nil {
		return "", err
	}
	path := filepath.Join(imagesDirPath, fileName)
	err = os.WriteFile(path, data, 0644)
	if err != nil {
		return "", err
	}
	return path, nil
}
func (d DiskStorage) Get(filePath string) ([]byte, error) {
	return nil, nil
}

func (d DiskStorage) Delete(filePath string) error {
	return nil
}
