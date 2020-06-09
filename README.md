# vortex-rx

[![](https://img.shields.io/crates/v/vortex?style=for-the-badge)](https://crates.io/crates/vortex)

```
cargo install vortex
```


Example yaml config:
```
addr: 0.0.0.0
port: 8080

routes:
  - respond:
      path: ^/no.*
      status-code: 400
      content-type: plain/text
      body-string: "No"

  - respond:
      path: .*
      status-code: 200
      content-type: application/json
      body-string: >-
        {
          "Goodnight": "world!"
        }
```