
docker-build:
	 docker build -f deploy/Dockerfile . -t rust-supermarket

docker-compose:docker-build
	docker-compose -f ./deploy/docker-compose.yml up