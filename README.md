# gsudo

Sudo GUI helper for macOS.

## Installation
```sh
$ make
$ sudo make install
```
or
```sh
$ brew tap x13a/tap
$ brew install x13a/tap/gsudo
```

## Usage
```text
gsudo [-h|V] <EXECUTABLE> [..<ARG>]
```

## Example

To exec with admin rights:
```sh
$ gsudo /bin/ls -la
```
