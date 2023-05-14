# KSBD bot

## What

There's an awesome graphic novel https://killsixbilliondemons.com/. Am really into it and I want to be notified when a
new episodes are posted.

This project is a Telegram-bot covering that requirement. Also, it provides a simple navigation across the entire novel.

The bot is deployed to my Raspberry PI.

## Why

This is my first attempt to write code in Rust. The idea was to make it work as fast as possible and then improve along
with learning process.

And let's be honest, this code is still ugly AF :)

I'm still a bit frustrated with some ideas in language and/or libraries.

Questions and topics to my future self to cover along the way I learn Rust:

- Unit tests. Just write them.

- Passing callback functions. How? That would be an easiest way to inject dependencies. And will make a code really
  testable. Haven't found an easy way to do that.

- For instance, teloxide's way to make so-called "dependency injection" using that `dptree` lib makes me almost cry! If
  you'd not provide needed dependencies, those branches will fail in RUN time. How about make'em fail in COMPILE time?
  Nope. Sounds like a return to a "good-'ol" prehistoric Spring-times. Instead, I'll be more than happy to just pass'em
  along as a function params... you know, the one and only RIGHT way to inject dependency. Is there a way to do that?

```rust
let command_handler = teloxide::filter_command::<Command, _ > ()
.branch(case![Command::Start].endpoint(start))
.branch(case![Command::Help].endpoint(help))
...
```

- Cross-compilation to Raspberry... oh boy. Special kind of nightmare. I just used https://github.com/cross-rs/cross to
  make it happen. However, the topic is still open -- is there a way to leverage ALL the horsepower of my M1 instead of
  half-assed solution using docker mumbo-jumbo?
