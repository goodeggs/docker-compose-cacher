docker-compose-cacher
---------------------

Docker Compose files are quite useful in CI/CD pipelines, as they're a great way to explicitly specify and run dependencies.  The performance of Docker Registries, and the inconsistency of public internet download speeds in general, are less great.  `docker-compose-cacher` aims to help, by managing a local cache directory of the Docker Images you depend on.  This directory can be treated as a cacheable resource, which most CI/CD pipelines have provisions for.


Example Timings
===============

In one of our apps that depends on MongoDB, RabbitMQ, and Redis:

  - empty cache: `83 seconds`
  - full cache: `21 seconds`
  - subsequent runs: `6 seconds` (where the Docker Images had already been loaded)

Releasing
===========

```
$ brew install goodeggs/delivery-eng/ghr goodeggs/delivery-eng/gitsem FiloSottile/musl-cross/musl-cross
$ gitsem {patch,minor,major}
$ git push
$ ./release.sh
```
