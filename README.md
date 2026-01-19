# chkc's help

it's a really simple library inspired by [clap-help](https://github.com/Canop/clap-help) but
with support for submodules, and custom Markdown guides. it prints your clap help as markdown
with a bit of color and will open a scrollable view if the output doesn't fit on screen.

## what ships in the crate?
- `help_command` / `help_command_docs` and friends plug straight into your clap match arms
- `HelpArgs` is a clap `Args` struct you can attach to your own `help` command
- `DocRegistry` + `CommandDoc` store extra blurbs, examples, notes, and guides keyed by
  dot-separated paths (`""` points to the program itself)
- `HelpTheme` wraps a `termimad::MadSkin` with an accent color
- `HelpPage`, `render_command_help`, and `run_scrollable_help` let you render things yourself if
  you want something lower-level

## how do i use it?

you have two main ways of using this library, and you can do them at the same time!

### specific `help` command
you can add a snippet like this to your clap command matching (using `HelpArgs`):
```rust
Some(Commands::Help(args)) => 
    { 
        chkc_help::help_command(
            "My Cool Project",
            Some(env!("CARGO_PKG_VERSION")),
            &Cli::command(),
            &theme,
            &args
        )
    },
```
where `args` comes from
```rust
use chkc_help::{Color, HelpArgs, HelpTheme};
```

### help when no command specified
you can add this to your command matching:
```rust
None => {
            chkc_help::help_command_program(
                "My Cool Project",
                Some(env!("CARGO_PKG_VERSION")),
                &Cli::command(),
                &theme
            )?;
            Ok(())
        }
```

### help + markdown guides (additional command data)
 both `help_command` and `help_command_program` have a `_docs` variant that additionally takes
in a DocRegistry, which you can create by doing so before parsing:
```rust
let mut docs = chkc_help::DocRegistry::new();
```
you can then register commands or guides by doing `docs.register_command(key, cmd)` 
and `docs.register_guide(key, guide)` respectively. the key is a value referring to the
"namespace" or command, which should be separated by dots in the case of subcommands
(e.g: `git.commit`)

having a doc registry is really recommended, because you can add a description
(leave empty to use the one extracted from doc comments), examples of your command usage,
and even notes.

for example:
```rust
registry.register_command(
        "commit",
        CommandDoc::new(
            "Create a new commit containing the current contents of the index and
            the given log message describing the changes.",
            [
                "Commit a repository: `git -m ~~'commit message'~~`",
            ],
            ["If you make a commit and then find a mistake immediately after that, you can recover from it with git reset."],
        ),
    );
```
note that an empty key in the doc registry symbolizes the
main program itself, so `program help guide` is still
valid

### but i dont want to use HelpArgs!
oh, well too bad. just kidding, you can use the lower level
```rust
run_help_topic(app_name, app_version, root, &docs, theme, &topic)
```
where topic is a `Vec<String>`

### docs registry cheat sheet
- `CommandDoc::new(desc, examples, notes)` will drop an empty description and keep your clap doc
  comments instead
- `register_command("foo.bar", doc)` attaches data to a subcommand
- `register_guide("git.rebase", include_str!("docs/rebase.md"))` stashes arbitrary markdown you
  can open with `help git rebase guide`
- pass the registry to the `_docs` helpers to make it all show up

### rendering bits
- "Usage:" is trimmed off clap's output, leaving just the syntax in backticks
- subcommands and options are printed as small tables; strikethrough text uses the accent color
  (better than squinting at a ~~strike~~ that doesn't render everywhere)
- if the markdown has more lines than the terminal, you'll get a scrollable view with arrows,
  page up/down, and `q` / `esc` to quit

### theming
you've probably seen `&theme` around, but what is it? well, it's simple.
you create it like this:
```rust
use chkc_help::{Color, HelpTheme, /* other imports */};

// in your main function
let theme = HelpTheme::default(Color::Blue);
```

you can specify your accent color of your choice, of course. `HelpTheme::light`, `dark` and
`new` all exist too, with the latter taking a `mut skin: MadSkin` in case you want some more
customization. the accent color will still override some stuff though. if you want to render
the markdown yourself, `render_command_help` and `run_scrollable_help` are exported as well.

---

alright thats all, bye
