package job

type Job struct {
	ID     string `redis:"id"`
	Status string `redis:"status"`
	Path   string `redis:"path"`
}

func New(id, status, path string) Job {
	return Job{id, status, path}
}
