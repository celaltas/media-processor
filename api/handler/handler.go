package handler

import "github.com/celaltas/media-processor/api/storage"

type Handler struct {
	FileStorage   storage.FileStorage
	JobQueue      storage.JobQueue
	MetadataStore storage.JobMetadataStore
}

func New(
	fs storage.FileStorage,
	metaStore storage.JobMetadataStore,
	queue storage.JobQueue,
) *Handler {
	return &Handler{
		FileStorage:   fs,
		MetadataStore: metaStore,
		JobQueue:      queue,
	}
}
