# sling

Distribute python packages using cloud storage services

## Cloud Providers support

 - Google Storage - `--driver gs`
 - AWS S3 - `--driver s3`

## Usage

When running sling, you'll need to indicate which `--bucket` you want to
use and which storage `--driver` you want to use. The tool will save
your preferences automatically on your system.

Download and install a python package:

```shell
sling --driver s3 --bucket my-s3-bucket get pkg@0.1.0
```

Upload a compiled python package:

```shell
sling --driver gs --bucket my-s3-bucket put ./dist/pkg-0.1.0.tar.gz
```

## TODO

 - [ ] Profile support
 - [ ] Full version specification support. i.e. `pkg@^1.0`
 - [ ] Wheel support
     * Upload .whl files during `put`
     * Download install system appropriate .whl file during `get`
 - [ ] Dependency resolution
 - [ ] Integration with `pip install`
     * Not sure how to accomplish this
