package main

import (
	"github.com/celaltas/media-processor/api/handler"
	"github.com/celaltas/media-processor/api/storage"
	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
)

func main() {
	router := gin.Default()
	router.MaxMultipartMemory = 8 << 20
	opt, err := redis.ParseURL("redis://default:secret_passwd@localhost:6379/0")
	if err != nil {
		panic(err)
	}

	client := redis.NewClient(opt)

	fs := storage.DiskStorage{}
	metaStore := storage.RedisMetadataStore{Client: client}
	queue := storage.RedisQueue{Client: client}
	handler := handler.New(fs, metaStore, queue)
	router.POST("/upload", handler.Upload)
	router.GET("/status/:job_id", handler.Status)
	router.Run()
}
