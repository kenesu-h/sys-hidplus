# sys-hidplus-client-rs
A Rust rewrite of the original input client for sys-hidplus.

After noticing that sys-hidplus wasn't updated in a while, plus the fact that the input client was
written in Python, I figured I could take some time to refactor and potentially add to it. No insult
to Ignaclo(?) (I believe they acknowledged this too), but the original client had repetitive code
and...  wasn't the most readable.

The original intent was to refactor it all using typed Python, but my attempt at implementation was
painfully slow in terms of runtime, possibly because of some of the many modules I tried to use
(such as `enum`).

# Why Rust?
I mostly wanted to rewrite the client using Rust because:
- I'm currently learning it and want to practice it more.
- It's memory and thread-safe while maintaining decent performance.
- In my experience, Rust crates are documented way better (and sometimes more capable) than Python
modules.
  - `gilrs` has proved to be more feature-rich and better-documented than `inputs`.
- Dependency management is a lot easier thanks to Cargo.
- No need for users to install languages and modules since this is compiled to a binary.

# Differences
The rewrite is mostly the same as the original (including 4th controller support) with an addition
here and there, but it also excludes some features that I haven't gotten to.

## Additions
- Controllers can be hotplugged.
- Controller slots will be preserved (and still work) even if they're disconnected, then
reconnected. This slot can be overridden by other controllers if another is assigned while it's
disconnected.
- A controller can be assigned to the first available slot by pressing LTrigger (ZL) + RTrigger
(ZR). This also means controllers are not assigned when the client is started.
- Controller configuration is now in a separate config file as opposed to being within the main
script.
- Controllers will now be disconnected when using Ctrl-C to close the client.

## Modifications
- Adaptation to Rust (kind of a given).
- Refactoring that consists mostly of separating functionality into individual structs and
abstracting controller polling.
- Polling is event-driven and single-threaded as opposed to being multi-threaded. This may have
unintended side-effects, but so far it's doing okay.
- Unfortunately, the client itself isn't as universally accessible since building it via Cargo only
makes it runnable for the OS that did so; a client executable built on Windows will only work on
Windows, Linux for Linux, etc. Luckily, Rust is somewhat easy to install thanks to methods like
`rustup`.

## Things that don't work and/or are still a mystery to me
- Sideways joy-cons don't work. I'm not sure what the reason is, but this also happened with the
original input client.
- If a controller is disconnected by the Switch - like through the "Change Grip/Order" menu or the
new "disconnect" button in Smash Ultimate - you cannot reconnect it and there seems to be nothing
you can do about it. This renders some games unplayable. The only way to fix or alleviate this is to
restart your Switch if it happens, and avoid any menus (if possible) that may disconnect your
controllers forcefully.

Aside from that, this rewrite is still pretty functional. I have no idea if the same issues from the
original apply - such as stick inversion on Linux and input lag on demanding games - since I don't
have many games on the Switch to test this with. I don't have a computer with Linux on it to test
either, but that might be mitigated from using `gilrs`, which is supposed to be better in terms of
cross-platform functionality.

# Compiling
Compilation assumes you have Rust installed, which should come with Cargo.

Compiling is as easy as running `cargo build` within the root. You should be left with a binary
executable in `target/debug/` runnable only by your OS.

# Contact
If you want to contact me about this, you can reach me at Kenesu#2586 on Discord.

# Credits
Credits go to Ignaclo(?) for sys-hidplus as a whole and everyone else who helped them out. I really
mean no offense with this fork, and besides, I wouldn't be even be doing this if it weren't for all
their hard work making sys-hidplus as great as it already is.