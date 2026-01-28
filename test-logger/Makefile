.PHONY: build run stop logs clean

build:
	docker build -t test-logger .

run: build
	docker run -d --name test-logger test-logger

stop:
	docker stop test-logger || true
	docker rm test-logger || true

logs:
	docker logs -f test-logger

clean: stop
	docker rmi test-logger || true

# Quick test with docker-compose
up:
	docker-compose up --build -d

down:
	docker-compose down
