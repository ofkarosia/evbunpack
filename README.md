# evbunpack
Enigma Virtual Box Unpacker written in Rust.

## Features
- Auto detection for:
  - VFS type: Modern, Legacy
  - PE variant: 10.70, 9.70, 7.80

## Limitations
- Does not support multiple root folders
- For root folders, only `%DEFAULT FOLDER%` is supported

## Acknowledgements
The [core library](https://github.com/ofkarosia/evbunpack_core) of this project is based on the logic and research of [the original Python implementation](https://github.com/mos9527/evbunpack).
Thanks to the original authors for their groundwork on the EVB format.

## License
[Apache 2.0](./LICENSE)
