#Bauanleitung
(Ja, auf deutsch; Unix/Linux Only)




## Rust installieren
via [https://rustup.rs/](https://rustup.rs/)

```bash
curl https://sh.rustup.rs -sSf | sh
```

Defaults sind okay.




## Enpacken

 tar.gz entpacken und cd in Ordner.
 Sieht dann ca. so aus:
```text
$ tree
.
├── Cargo.toml
.   ...
├── hs_crawler
│   ├── Cargo.lock
│   ├── Cargo.toml
│   └── src
│       ├── crawler
│       │   ├── canteen_plan.rs
│       │   ├── mod.rs
│       │   ├── timetable.rs
│       │   └── weather.rs
│       ├── lib.rs
│       └── util.rs
└── src
    ├── main.rs
    ├── testing.rs
    ├── tui
    │   ├── keys.rs
    │   ├── mod.rs
    │   ├── termutil.rs
    │   └── vterm.rs
    ├── ui
    │   ├── json.rs
    │   ├── mod.rs
    │   └── theme.rs
    └── util.rs

6 directories, 19 file

```




## Starten
!! Es wird Openssl benötigt.

```bash
$ cargo run
```
Dauert eine viertels Ewigkeit. Da wird ein Html5-Parser und
HTTP Client kompiliert.




## Bauen

```bash
$ cargo build --release
```
```bash
$ mv ./target/release/hs_app hs-app
```