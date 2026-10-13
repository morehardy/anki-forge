# Use Anki Forge with an AI coding assistant

For a zero-install start, [choose a language and copy a task brief](https://ankiforge.dev/#ai-quickstart).
Paste it into your coding assistant, replace the task placeholder, and run the
generated program and verification steps. The website assembles text locally;
it does not call an AI API or store your request. A chat without project access
can provide files and commands for you to run.

This entry targets public package version 0.3.0. On 2026-10-12, the official npm
registry listed only 0.2.0 for `ankiforge`; the Node 0.3.0 installation could not
be executed. Publish the matching Node wrapper and native packages before
deploying its 0.3.0 brief. Rust and Python 0.3.0 consumers were available for
local validation. See [official npm metadata](https://registry.npmjs.org/ankiforge).

## Load the Skill

The platform-neutral source is [skills/anki-forge](../skills/anki-forge/SKILL.md).
Copy the whole folder, including `references`, into your tool's supported Skill
directory. `skills/` alone is not an automatically discovered directory in every tool.

| Coding assistant | Project location | Personal location | Official loading documentation |
| --- | --- | --- | --- |
| Codex | `.agents/skills/anki-forge/` | `~/.agents/skills/anki-forge/` | [OpenAI skill documentation](https://learn.chatgpt.com/docs/build-skills) |
| Claude Code | `.claude/skills/anki-forge/` | `~/.claude/skills/anki-forge/` | [Claude Code skills](https://code.claude.com/docs/en/skills) |

These paths were checked against official documentation on 2026-10-12.
The Skill uses ordinary Markdown frontmatter and relative reference links.
Confirm that your tool discovers `anki-forge` after installation; the loading
paths above do not establish that every tool version was tested here.

For example, from a local checkout of this repository, on macOS/Linux:

```sh
mkdir -p ~/.agents/skills
cp -R skills/anki-forge ~/.agents/skills/
```

Use `.claude/skills` instead for a Claude Code project installation. Copy to an
unused destination, or review your existing Skill before replacing it. These
commands are instructions for you; the website does not install anything.
For Cursor, ChatGPT or another tool without a configured Skill loader, use the
copied brief and the linked public documentation.

Then ask, for example:

> Use Anki Forge to turn my Spanish vocabulary CSV into a deck with audio.

The Skill directs the assistant to inspect the project language and dependency,
read only the relevant language/task references, build a persistent APKG with
stable namespace and note keys, and report the run command and verification.
Only the public documented package APIs should be used.

## Verify generated work

Check the generated source and commands before running them. For a Basic
quickstart expect `spanish.apkg`, one note, one card, and no media. For media,
templates or updates, inspect the resulting counts and the intended client
behavior. Keep the previous original distribution APKG as the update baseline;
an Anki re-export does not preserve the required identity evidence.

The repository's `verify_quickstart.mjs` script is a contributor content check.
Run it in this repository; it is not a deck inspector or an application dependency.
See [documentation maintenance](documentation.md) for generation and validation.
