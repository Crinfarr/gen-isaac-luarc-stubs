# This is a really light readme because this is a really light tool
## Installation
* Run `cargo install --git https://github.com/crinfarr/gen-isaac-luarc-stubs --locked` and make sure `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on win) is in your $PATH
## Usage
Run `gen-isaac-luarc-stubs` in your project folder to generate lua stubs and add them to your linter as a library
If you specify `NO_REPENTAGON` in your env (or REPENTAGOFF or REPENTAGONE), stubs will be generated without repentagon support.
You can softlink the `.luarc.json` from your data directory to any project to enable completion there.
| OS | Data dir |
| -- | ---------- |
| Windows | `%USERPROFILE%\AppData\Roaming\isaac_luarc` |
| Mac | `~/Library/Application Support/isaac_luarc` |
| Linux | `~/.local/share/isaac_luarc` |
