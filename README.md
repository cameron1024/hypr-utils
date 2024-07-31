# Hypr utils

A collection of tools I use to make scripting in hyprland easier, though there is no dependency on hyprland at all.

## Installation

I'm too lazy to put it in package managers, you can build from source with `cargo`:
 - install `cargo`
 - clone the repo
 - `cargo install --path .`

You can also use the nix flake

First, add it to your inputs:
```nix
# flake.nix
{
  inputs.hypr-utils.url = "github:cameorn1024/hypr-utils";
  // ...
}
```

Then, add to either `environment.systemPackages` (if using NixOS) or `home.packages` (if using Home Manager):
```nix
{pkgs, inputs, ...}: {
  home.packages = [
    inputs.hypr-utils.packages.${pkgs.system}.default
  ];
}
```
In order for nix to automatically inject the `input` parameter, it needs to be added to `specialArgs` (if using NixOS) or `extraSpecialArgs` (if using Home Manager).



## Features

### Store

A simple database, backed by sqlite:

```bash
> hypr-utils store get foo
# prints nothing

> hypr-utils store set foo 123
> hypr-utils store get foo
123

# keys are strings, values are arbitrary json
# if json parsing fails, falls back to string
> hypr-utils store set foo hello
> hypr-utils store get foo
hello

# if passing the output to a tool that expects valid JSON (e.g. jq),
# pass `--json-format accurate`
> hypr-utils --json-format accurate store get foo 
"hello"

> hypr-utils store set foo hello
> hypr-utils store set bar '[1, true, "hello"]'
> hypr-utils store list
foo hello
bar [1,true,"hello"]
```

There is also a "special" store command: `cached`.

This is useful in contexts where you want to get the output of a command, but if the command fails, the last successful value is acceptable.

```bash
> hypr-utils store cached "curl 'wttr.in/London?format=3'"
London: ⛅️  +19°C

# now disable internet
> curl 'wttr.in/London?format=3'
curl: (6) Could not resolve host: wttr.in

> hypr-utils store cached "curl 'wttr.in/London?format=3'"
London: ⛅️  +19°C
```

### System

Right now there's only one command: `battery`, which pretty-prints your current battery level:

```bash
> hypr-utils system battery
 35%

# you can override the charging status and battery level
# this can be useful if you want to format the output of a different tool
> hypr-utils system battery --override-percentage 100
 100%

> hypr-utils system battery --override-charging true
 34%
```
