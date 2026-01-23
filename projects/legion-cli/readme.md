

### Configuration file query order

```sh
legion.toml
legion.json5
legion.yaml
legion.yml
legion.json
.config/*
```


v2024.0.1


legion update --dependencies -d
legion update --interactive -i
legion update --major       -M
legion update --minor       -m
legion update --patch       -P
legion update --pre-release -p


legion install a --override
legion upgrade

1.0.0-alpha -> 1.0.0-alpha.1


2024.0.1 -> 2024.0.1-patch -> 2024.0.1-patch.1 -> 2024.0.2


2024.0.0-patch.11 -> 1.0.0-patch.12

