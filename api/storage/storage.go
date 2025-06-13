package storage

import (
	"context"

	"github.com/celaltas/media-processor/api/job"
)

type FileStorage interface {
	Save(fileName string, data []byte) (string, error)
	Get(filePath string) ([]byte, error)
	Delete(filePath string) error
}

type JobQueue interface {
	Enqueue(ctx context.Context, jobID string) error
	Dequeue(ctx context.Context) (string, error)
}
type JobMetadataStore interface {
	StoreMetadata(ctx context.Context, jobID string, job job.Job) error
	GetMetadata(ctx context.Context, jobID string) (map[string]string, error)
}
