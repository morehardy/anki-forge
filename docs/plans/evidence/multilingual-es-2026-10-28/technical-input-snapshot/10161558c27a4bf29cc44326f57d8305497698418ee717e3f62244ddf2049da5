# Generate Anki flashcards with an AI assistant

Use an AI assistant to draft question-and-answer pairs from your notes, check
them against the source, then use Anki Forge to export an Anki `.apkg` deck.
Anki Forge is the open-source generation library in this workflow: the assistant
drafts content or writes a script, and your local Python, Node.js or Rust program
builds the cards.

You can [copy a coding-assistant brief](/#ai-quickstart) to get started with a
programming language, or follow the CSV workflow below. The website's brief
builder assembles text locally; it does not call an AI model or upload your notes.

<!-- section: choose-input -->
## Start with source material you can check

Use a short passage with a clear subject, such as your own course notes or a
vocabulary list. For PDFs, scans or lecture slides, extract and check the text
with a separate tool first. Anki Forge accepts the content you provide through
its APIs; PDF extraction and OCR are outside this library.

Keep each proposed question focused on one fact. Include enough context for the
answer to make sense when the card appears outside the original document.

<!-- section: draft-cards -->
## Ask for CSV cards

Paste this example into your assistant. Replace the source passage when making
your own deck, and adjust the requested note count and IDs for your new notes:

```text
Turn the source below into exactly three Basic Anki notes.
Use only facts stated in the source. Do not add facts from memory.
Each question must test one fact and be understandable on its own.
Return only UTF-8 CSV with the header id,front,back, with no Markdown fence.
Use the IDs es:hola, es:gracias and es:adios for these three vocabulary items.
Quote any CSV field containing a comma, quote or newline correctly.

Source:
In Spanish, hola means hello, gracias means thank you, and adiós means goodbye.
```

The expected content is equivalent to:

```csv
id,front,back
es:hola,hola,hello
es:gracias,gracias,thank you
es:adios,adiós,goodbye
```

The assistant may phrase the front differently. Check each row against the source,
remove ambiguous or unsupported answers, then save the reviewed result as
`cards.csv`. Assign each new note a unique ID and retain it during later edits;
do not ask the assistant to regenerate IDs for an existing deck.

<!-- section: export-apkg -->
## Turn reviewed cards into an APKG

Follow [CSV to Anki with Python](csv-to-anki.md) for the complete program,
installation commands and sample input. That script checks the CSV shape,
requires unique IDs and writes a persistent package. The sample creates three
Basic cards with no media.

If your data is already JSON or you need images and audio, use the
[JSON to Anki recipe](json-to-anki.md), with complete Python, Node / TypeScript
and Rust programs. CSV and JSON are parsed by your application before it passes
notes to Anki Forge.

For help writing a custom program, use the
[coding-assistant brief](/#ai-quickstart) and describe the input fields, desired
card type and output filename. The [optional Anki Forge Skill](ai-assistants.md)
provides the public API references and task workflows for assistants that load Skills.

<!-- section: check-result -->
## Check the cards before studying

Open the generated APKG in Anki and inspect the card count, question, answer and
formatting. For the CSV example, expect three notes and three cards in the Spanish
deck. A successful export confirms that a package was built; it does not establish
that an AI's answers are correct or useful for studying.

For fill-in-the-blank questions, use [Cloze cards](cards.md). For diagram labels,
use [Image Occlusion](image-occlusion.md) with the image and mask coordinates you
provide; this does not automatically detect labels in an image.

Keep the original distribution APKG for the next release and follow the
[update workflow](updates.md). Preserve the namespace and note IDs, inspect the
comparison, and account for Anki's import settings and newer local edits.
