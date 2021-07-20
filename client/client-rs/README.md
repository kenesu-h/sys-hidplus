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

## Modifications
- Adaptation to Rust (kind of a given).
- Refactoring that consists mostly of separating functionality into individual structs and
abstracting controller polling.
- Polling is event-driven and single-threaded as opposed to being multi-threaded. This may have
unintended side-effects, but so far it's doing okay.

## Things I haven't gotten to and/or are still a mystery to me
- I haven't implemented stick states changing based on the sideways joy-cons used.
- For some reason, I can't seem to disconnect disconnected (by the Switch) controllers by setting
their type to None (or 0). In fact, I usually need to reboot my Switch to get them working again.

Aside from that, this rewrite is still pretty functional. I have no idea if the same issues from the
original apply - such as stick inversion on Linux and input lag on demanding games - since I don't
have many games on the Switch to test this with. I don't have a computer with Linux on it to test
either, but that might be mitigated from using `gilrs`, which is supposed to be better in terms of
cross-platform functionality.

# Contact
If you want to contact me about this, you can reach me at Kenesu#2586 on Discord.

# Credits
Credits go to Ignaclo(?) for sys-hidplus as a whole and everyone else who helped them out. I really
mean no offense with this fork, and besides, I wouldn't be even be doing this if it weren't for all
their hard work making sys-hidplus as great as it already is.