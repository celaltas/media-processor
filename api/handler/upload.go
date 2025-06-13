package handler

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/celaltas/media-processor/api/job"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

func (h *Handler) Upload(c *gin.Context) {
	ctx := context.Background()
	fileHeader, err := c.FormFile("file")
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":   "file_upload_error",
			"message": "Failed to get uploaded file. Please ensure you're sending a valid file upload.",
		})
		return
	}

	f, err := fileHeader.Open()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "file_read_error",
			"message": "Could not read the uploaded file content.",
		})
		return
	}
	defer f.Close()

	buffer := bytes.NewBuffer(nil)
	if _, err := io.Copy(buffer, f); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "file_buffer_error",
			"message": "Failed to process file content.",
		})
		return
	}

	path, err := h.FileStorage.Save(fileHeader.Filename, buffer.Bytes())
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "storage_error",
			"message": fmt.Sprintf("Failed to save file '%s' to storage.", fileHeader.Filename),
		})
		return
	}

	id := uuid.New().String()
	job := job.New(id, "queued", path)

	if err := h.processJob(ctx, job); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "job_processing_error",
			"message": "Failed to process your upload. Please try again later.",
			"details": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"status":  "success",
		"message": fmt.Sprintf("File '%s' uploaded successfully and queued for processing.", fileHeader.Filename),
		"job_id":  id,
		"path":    path,
	})
}

func (h *Handler) processJob(ctx context.Context, j job.Job) error {
	if err := h.MetadataStore.StoreMetadata(ctx, j.ID, j); err != nil {
		return fmt.Errorf("failed to store job metadata: %w", err)
	}

	if err := h.JobQueue.Enqueue(ctx, j.ID); err != nil {
		return fmt.Errorf("failed to enqueue job: %w", err)
	}
	return nil
}
