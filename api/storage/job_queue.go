package storage

import (
	"context"

	"github.com/redis/go-redis/v9"
)

type RedisQueue struct {
	Client *redis.Client
}

func (r RedisQueue) Enqueue(ctx context.Context, jobID string) error {
	_, err := r.Client.LPush(ctx, "image:process", jobID).Result()
	return err
}

func (r RedisQueue) Dequeue(ctx context.Context) (string, error) {
	return "", nil
}
