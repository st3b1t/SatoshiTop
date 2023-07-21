
## Run in Docker

You need to assign host `net` and `pid` to access the metrics in the host machine.

```sh
$ docker build . -f ./docker/Dockerfile -t st3b1t/satoshitop:latest
```

```sh
$ docker run --rm -it --name satoshitop --net="host" --pid="host" st3b1t/satoshitop
```

Or using Docker Compose
```sh
cd docker
docker compose up
```

**Please Note:** for now, starting SatoshiTop requires rebooting the container in foreground and then re-running the docker command.
in the future it may be possible to keep the container running in the background and then access the output of the `satop` command executable within the container
