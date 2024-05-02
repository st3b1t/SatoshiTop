
## Run in Docker

Build docker image from source code

```sh
$ docker build . -f ./docker/Dockerfile -t st3b1t/satoshitop:latest
```

Or using npm scripts:
```sh
$ npm run docker-build
$ npm run docker-save
```

In particular `docker-save` generare a new file name `satohitop.tar` can you copy and import in your Bitcon machine.


You need to assign host `net` and `pid` to access the metrics in the host machine.

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
