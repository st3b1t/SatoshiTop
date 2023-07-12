
## Run in Docker

You need to assign host `net` and `pid` to access the metrics in the host machine.

```sh
$ docker build . -f ./docker/Dockerfile -t st3b1t/satoshitop
```

```sh
$ docker run --rm -it --name satoshitop --net="host" --pid="host" st3b1t/satoshitop
```

Or using Docker Compose
```sh
cd docker
docker compose up
```