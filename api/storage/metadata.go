package storage

import (
	"github.com/celaltas/media-processor/api/job"
	"github.com/redis/go-redis/v9"
	"context"
)

type RedisMetadataStore struct {
	Client *redis.Client
}

func (r RedisMetadataStore) StoreMetadata(ctx context.Context, jobID string, job job.Job) error {
	_, err := r.Client.HSet(ctx, jobID, job).Result()
	return err
}
func (r RedisMetadataStore) GetMetadata(ctx context.Context, jobID string) (map[string]string, error) {
	return r.Client.HGetAll(ctx, jobID).Result()
}
