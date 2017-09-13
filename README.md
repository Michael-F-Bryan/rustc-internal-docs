# Rustc Internal Docs

A continually updated copy of `rustc`'s internals hosted by GitHub pages.

https://michael-f-bryan.github.io/rustc-internal-docs/


# Getting Started

To use this you first need to install it:

```bash
$ cargo install --git https://github.com/Michael-F-Bryan/rustc-internal-docs
```

Then make sure you have a copy of the [config file] in your home directory.

```bash
$ curl -o- -L https://github.com/Michael-F-Bryan/rustc-internal-docs/raw/master/rustc-internal-docs.toml > ~/.rustc-internal-docs.toml
```

If you are just wanting to use this for personal use, you'll want to open the 
config file and set `skip-upload` to `true`. This means it'll still build the
internal compiler documentation in your `rust-dir`, but will skip the uploading
to GitHub Pages step.

Next, to get the best use out of this tool I'll add it as a cron job so I 
always have an up-to-date version of the docs. 

```
0 */6 * * * USE_SYSLOG=1 /home/michael/.cargo/bin/rustc-internal-docs -v 
```

> **Note:** The `USE_SYSLOG` environment variable tells `rustc-internal-docs` 
> to log to the system logger. This means I can view logs just like any other
> service on my system (`journalctl -e -t rustc-internal-docs`)... Plus `cron`
> will usually try to email you when *any* job prints something to 
> stdout/stderr.


[config file]: https://github.com/Michael-F-Bryan/rustc-internal-docs/blob/master/rustc-internal-docs.toml