<div align="center">
  <h1> 🚧 Work In Progress 🚧 </h1>
  <br>
  <h2> TUI version of dmidecode </h2>
  <br>
</div>

<img width="935" height="862" src="https://github.com/user-attachments/assets/2f6c1642-3b5a-4ac6-ba0d-9769fd12cf53" />

<br>

## What is dmidecode ?

From `dmidecode` man page:

> **`dmidecode`** is a tool for dumping a computer's DMI (some say SMBIOS) table contents in a human-readable format. This table contains a description of the system's hardware components, as well as other useful pieces of information such as serial numbers and BIOS revision. Thanks to this table, you can retrieve this information without having to probe for the actual hardware.

## Why dmitui then ?

`dmitui` is a TUI (Text User Interface) version that allows for easy navigation between sections, unlike `dmidecode`, which requires you to specify the section as a command-line option. Additionally, `dmitui` presents information in a well-organized and visually appealing manner.

## 💡 Prerequisites

A Linux based OS.

## 🚀 Installation

### ⚒️ Build from source

To build `dmitui`:

```
cargo build --release
```

This will produce an executable file at `target/release/dmitui` that you can copy to a directory in your `$PATH`.

## 🪄 Usage

Run the following command to start `dmitui`:

```
sudo dmitui
```

## 📌 Supported DMI types

- [x] Firmware (type 0)
- [x] System (type 1)

## ⚖️ License

GNU General Public License v3.0 or later
