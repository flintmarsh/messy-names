# messy-names

Name lists for random generation (fantasy character names, placeholder
data, raffle entries, whatever) almost never arrive clean. They're
copy-pasted from wiki tables, exported from spreadsheets, or typed by
hand across a few sessions, so you end up with inconsistent casing,
doubled-up spaces, stray commas, and no clear grouping. `messy-names`
takes that mess and turns it into a small, unambiguous source format,
then can draw a random entry from it.

## the format

```
# comments start with a hash
[dwarves]
thorin, BALIN
dwalin

[elves]
   legolas
```

Rules:

- Blank lines and `#` comments are ignored.
- Every name must belong to a `[category]` section.
- A line can hold one name or several, separated by commas.
- Each name has its whitespace collapsed and is title-cased, so
  `  thorin   oakenshield ` becomes `Thorin Oakenshield`.

## usage

```
$ cargo run -- dwarves.txt
[dwarves]
Balin
Dwalin
Thorin

[elves]
Legolas

$ cargo run -- dwarves.txt --pick dwarves
[dwarves]
Balin
Dwalin
Thorin

[elves]
Legolas

picked from [dwarves]: Dwalin
```

## errors

The one thing this tool is meant to get right: when a file is
malformed, you get the exact line and column, plus the offending
source line with a caret under it, instead of a vague "parse failed":

```
$ cat broken.txt
stray

[dwarves]
thorin,,dwalin

[elves

$ cargo run -- broken.txt
broken.txt:1:1: error: name given before any [category] header
  stray
  ^
broken.txt:4:8: error: empty name entry (check for a stray comma)
  thorin,,dwalin
         ^
broken.txt:6:7: error: category header is missing a closing ']'
  [elves
        ^
```

Parsing doesn't stop at the first problem; it collects every error in
the file so you can fix them all in one pass.

## known limitations

There is no way yet to weight names so some are picked more often
than others, and re-declaring the same `[category]` twice silently
merges into it rather than raising an error. See the roadmap for
what's planned.

## why no dependencies

This is a small, single-purpose tool. The standard library is enough
for parsing text and picking a random index, so that's all it uses.

## license

MIT, see [LICENSE](LICENSE).
