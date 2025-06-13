package handler

import (
	"context"
	"net/http"

	"github.com/gin-gonic/gin"
)

func (h *Handler) Status(c *gin.Context) {
	jobID := c.Param("job_id")
	ctx := context.Background()
	data, err := h.MetadataStore.GetMetadata(ctx, jobID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"error":   "get_metadata_job_error",
			"message": "Failed to get metadata",
		})
		return
	}
	c.JSON(200, gin.H{
		"message": data,
	})
}
