# Welcome to imap-codec's (and imap-types') contributing guide

Thanks for investing your time to help with this project! Keep in mind that this project is driven by volunteers. Be patient and polite, and empower others to improve. Always use your best judgment and be excellent to each other.

## Technicalities

### Formatting of code

We use [`rustfmt`] through `cargo +nightly fmt` configured by `rustfmt.toml` to format our code. This helps to minimize diffs and eases onboarding. Code formatting is automatically checked by GitHub Actions through our CI.

### Misuse resistance

We make use of strong-typing to [eliminate invalid state]. Ask yourself: Can I instantiate a type that has an invalid setting of variables? If yes, consider how to eliminate it. If you're unsure, let's figure it out together.

### Usage of features

IMAP is extensible. Thus, we use [Cargo features] to enable/disable extensions to the core IMAP protocol. Feature-gating helps to reduce the amount of exposed code and serves as documentation for the supported extensions.

[`rustfmt`]: https://github.com/rust-lang/rustfmt
[eliminate invalid state]: https://duesee.dev/p/type-driven-development/
[Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
