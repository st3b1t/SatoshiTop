# Showing satop on the machine's console

How to start `satop` from a remote shell (SSH) and have it drawn on the screen
attached to the machine: a virtual console such as `/dev/tty8`, typically a
small display on a node without a desktop.

## Why a plain redirection is not enough

```sh
satop </dev/tty8 >/dev/tty8     # wrong size
```

This draws on the console, but `satop` reads the terminal size from its
*controlling terminal*, which is still the SSH session. Unless both happen to
have the same size, the dashboard is laid out for the wrong one and the screen
is garbled.

The process must have the console as its controlling terminal, or none at all.
The three ways below take care of that.

## Permissions on the tty device

`satop` must open the device for **writing** (to draw) and for **reading** (to
receive the keys and to switch the terminal to raw mode). Check who may do
that:

```sh
ls -l /dev/tty8
```

```
crw--w---- 1 root tty 4, 8 ... /dev/tty8
```

| device | owner, group, mode | who can read and write |
|---|---|---|
| free virtual console (`/dev/tty1`…) | `root:tty`, `0620` | only root |
| virtual console with a user logged in | `<user>:tty`, `0620` | that user, and root |
| serial line (`/dev/ttyS0`, `/dev/ttyUSB0`) | `root:dialout`, `0660` | root and the members of `dialout` |

The `tty` group can only write, so adding your user to it is not enough: the
device still cannot be opened for reading (`Permission denied`). What to do
depends on how you start `satop`:

- **systemd-run, systemd unit:** nothing. systemd opens the device as root and
  hands it to the user given with `--uid` / `User=`, making that user its owner
  for as long as the service runs.
- **openvt:** nothing. It opens the device as root and `satop` inherits it
  already open.
- **Without root:** you must own the device, that is, be logged in on that
  console; for a serial line, be a member of `dialout`
  (`sudo usermod -aG dialout "$USER"`, then log in again).

Do not `chmod` the device to open it to everyone: any user could then read
what is typed on that console, and the mode is reset at the next login anyway.

## With systemd-run (recommended)

One command from the SSH session, no file to create:

```sh
sudo systemd-run --unit=satop --uid="$USER" \
  -p TTYPath=/dev/tty8 -p StandardInput=tty -p StandardOutput=tty \
  -p TTYReset=yes -p TTYVHangup=yes -E TERM=linux \
  /usr/local/bin/satop
sudo chvt 8                  # bring console 8 to the screen
```

- `--uid` runs `satop` as your user, so it reads your `~/.satoprc` and can
  read the cookie file you configured. Use the user that owns the node's
  cookie if that is a different one.
- Options go after the path, environment variables with `-E`:
  `-E RPCPORT=18332`, or `/usr/local/bin/satop --rpcport=18332`.
- Console 8 is used because login prompts are only started on consoles 1 to 6,
  so nothing else reads the keyboard there. To use `tty1`, stop its login
  prompt first: `sudo systemctl stop getty@tty1`.

To stop it, press `q` on the keyboard attached to the machine, or from the
remote shell:

```sh
sudo systemctl stop satop
```

`systemctl status satop` shows whether it is running.

### Starting at boot

Put the same settings in `/etc/systemd/system/satop.service`:

```ini
[Unit]
Description=SatoshiTop dashboard on the console
After=network-online.target bitcoind.service
Wants=network-online.target

[Service]
User=bitcoin
Environment=TERM=linux
ExecStart=/usr/local/bin/satop
StandardInput=tty
StandardOutput=tty
TTYPath=/dev/tty8
TTYReset=yes
TTYVHangup=yes
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Change `User=` to the user whose `~/.satoprc` should be used, and drop
`bitcoind.service` if the node runs elsewhere or under another unit name. Then:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now satop
sudo chvt 8
```

With `Restart=always` the dashboard comes back five seconds after `q`; stop it
with `sudo systemctl stop satop`.

To show console 8 at boot without typing `chvt`, add
`ExecStartPre=+/usr/bin/chvt 8` to the `[Service]` section.

## With openvt

`openvt` runs a command on a virtual console and makes that console its
controlling terminal:

```sh
sudo openvt -c 8 -s -- sudo -u "$USER" /usr/local/bin/satop
```

`-c 8` selects the console, `-s` switches the screen to it. The inner `sudo -u`
runs `satop` as your user rather than as root. Stop it with `q` on the
console's keyboard or with `pkill satop`.

## Without root

Log in on `tty1` at the machine (the console then belongs to you, see
[Permissions](#permissions-on-the-tty-device)), then from the remote shell, as
the same user:

```sh
setsid -f satop </dev/tty1 >/dev/tty1 2>&1
```

`setsid` starts `satop` without a controlling terminal, so it takes the size
from the console it draws on. The shell already running on `tty1` still reads
the same keyboard: a key pressed there may go to the shell instead of `satop`,
so stop it from the remote shell with `pkill satop`.

The same command works with any other terminal you can read and write, such as
a serial line (`/dev/ttyS0`, `/dev/ttyUSB0`) driving an external display.

## Screen size: rows and columns

`satop` lays the dashboard out for the number of character rows and columns of
the terminal it draws on. On a console that number is small and easy to get
wrong, so check it first (as root, or as the owner of the console):

```sh
sudo stty size </dev/tty8     # prints: rows columns
```

```
30 100
```

| size (columns x rows) | result |
|---|---|
| below 40x12 | only the message `Terminal too small` / `WxH (min 40x12)` |
| 40x12 to about 80x24 | every panel, with shortened labels and few rows per table |
| 120x40 or more | every panel comfortably |

### Getting more rows and columns

The size of a console is the resolution of the screen divided by the size of
one character of the font: a 480x320 display with the usual 8x16 font is 60x20,
with a 6x12 font it is 80x26.

- **Smaller font, now:** `sudo setfont -C /dev/tty8 Lat15-Terminus12x6`
  (fonts are in `/usr/share/consolefonts/`, from the `console-setup` package;
  the name ends with the height and width of a character).
- **Smaller font, at every boot:** set `FONTFACE` and `FONTSIZE` in
  `/etc/default/console-setup`, for example `FONTFACE="Terminus"` and
  `FONTSIZE="6x12"`, or run `sudo dpkg-reconfigure console-setup`.
- **Higher resolution:** if the display supports it, with the kernel parameter
  `video=`, for example `video=1024x600`. `fbset` prints the current one.
- **Rotated display:** the kernel parameter `fbcon=rotate:1` (or `2`, `3`)
  turns the console, swapping rows and columns.

`satop` reads the size again at every redraw, so it follows a change of font
within a few seconds, without a restart.

### When the size is wrong

- **The screen is garbled, with lines wrapping:** `satop` was started with a
  plain redirection and uses the size of the remote terminal (see the first
  section). Start it in one of the three ways above.
- **Serial lines** do not know the size of the display at the other end and
  report `0 0`. Set it by hand before starting `satop`, with the real size of
  the display:

  ```sh
  stty -F /dev/ttyS0 rows 24 cols 80
  ```

- **The dashboard is cut or leaves an empty band:** the size reported by
  `stty size` does not match what the display really shows, typically after a
  change of resolution or rotation. Fix it with `stty -F /dev/tty8 rows R cols C`
  or by setting the font again with `setfont`.

## Limits of the Linux console

The text console is poorer than a terminal emulator:

- **Characters.** A console font holds at most 512 glyphs. Braille dots, used
  for the lines of the Cpu chart, are not among them, and neither may be the
  partial blocks at the top of the bars (`▂`, `▆`): they show up as diamonds,
  question marks or blanks. A framebuffer terminal with a Unicode font fixes
  this, for example `fbterm` (`sudo apt install fbterm`) or `kmscon`.
- **Colors.** The console has 16 colors: the panel borders are drawn in the
  nearest one, cyan.
- **Screen blanking.** The console may blank after some minutes without key
  presses. `setterm --blank 0 --powerdown 0 >/dev/tty8` (as root) disables
  it, as does the kernel parameter `consoleblank=0`.
