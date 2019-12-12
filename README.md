# sqnz
A simple number sequence server. Common use-cases include central version number services, such as build numbers in Multibranch Pipelines.

## How to run

Start `sqnz`, which will case it to bind to `0.0.0.0:8080`. You can set the port with the `SQNZ_PORT` environment variable.

A folder named "sequences" will be created in the current working directory.

## How to use

To consume a sequence number:

```shell
curl -X POST http://sqnz/${project}/${tag}
```

To peek at a sequence number:

```shell
curl http://sqnz/${project}/${tag}
```
