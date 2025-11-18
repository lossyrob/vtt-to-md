NOTE: I did not use the Spec Researcher for this, as this is a brand new project with no prior code to analyze. Instead, I used Chat GPT Pro to generate the following:

Awesome project! I did a broad, source‑driven scan across the WebVTT spec, platform docs, developer forums, and well‑regarded CLI guidelines. Below is a concise, cited brief you can drop into your **feature/v1** planning issue.

---

## 1) WebVTT voice tags (`<v>`) — the official spec & real‑world variations

**What the spec says (official):**

* WebVTT defines a *voice span* using the `<v>` cue‑text tag. A `<v>` start tag **attaches a Voice object** whose value is the tag’s **annotation string** (the text following `v` in the tag); if no annotation is present, the value is the **empty string**. Example from the spec:
  `00:11.000 --> 00:13.000`
  `<v Roger Bingham>We are in New York City` ([w3.org][1])
  The normative rule is in the *cue text parsing rules*: **“Attach a WebVTT Voice Object, and set its value to the token's annotation string, or the empty string if there is no annotation string.”** ([w3.org][1])

**Platform behavior in the wild:**

* **Microsoft Teams** — exported `.vtt` transcripts generally put the speaker name in a `<v>` tag per cue, e.g.
  `00:00:00.800` `<v Lisa Simpson>Knock knock</v>` (example from users working with Teams VTT). ([GitHub][2])
* **Zoom** — cloud recordings yield a separate `.vtt` transcript file. In many files the speaker is embedded via `<v>`; however, some users report **numeric “speaker” placeholders** instead of names (i.e., *“1”, “2”* rather than real names) depending on account/settings. ([Zoom][3])
* **Google Meet** — exported `.vtt` files are **not always spec‑clean** and often **omit `<v>`**, using parenthetical initials and **blank lines** between fragments (which the spec doesn’t allow). A frequently cited example shows `(DF)` / `(DB)` lines and blank lines inside cue payloads. ([GitHub][4])

**Takeaways for your CLI:**

* Expect `<v>` **when it exists** to be the authoritative speaker label; be robust to its **absence** or **empty annotation** (both allowed by the spec). ([w3.org][1])
* Be prepared for **non‑standard Meet files** and **Zoom/Teams variability** in how names are encoded. ([GitHub][4])

---

## 2) Edge cases your parser should handle

From the spec and real files:

* **Missing or empty voice annotations**: `<v>text` (annotation empty) is valid; treat speaker as “Unknown” or similar. ([w3.org][1])
* **Malformed or mismatched tags** in cue text (`<i>`, `<b>`, `<u>`, `<c>`, `<v>`, `<ruby>`, `<rt>`, `<lang>`). Be permissive: ignore unknown/stray end tags and **close open spans at cue end**. (This follows the general design of the cue text tokenizer in §6.4; implement defensively.) ([w3.org][1])
* **HTML character references** inside annotations (speaker names can include `&amp;`, etc.) — the *start‑tag annotation state* integrates HTML character references; decode them. ([w3.org][1])
* **Blank lines in cue payloads** (seen in Google Meet exports) — **not legal** per spec; strip or coalesce. ([GitHub][4])
* **Non‑caption blocks**: `NOTE` comments, `STYLE`, `REGION` sections may appear; skip or preserve as metadata. (They’re part of the WebVTT file structure.) ([w3.org][1])
* **Special characters in speaker names**: quotes, punctuation, RTL marks, emoji; handle via Unicode normalization and Markdown escaping (see Q6). (CommonMark allows **backslash escapes for punctuation**.) ([GitHub][5])

---

## 3) Typical `.vtt` transcript sizes & performance notes

There isn’t a single “official” size table, but we can **estimate** using speaking rate + format overhead:

* **Speaking rate**: ~150–170 words/min in conversation. Using 150 wpm as a baseline from linguistic research. ([Language Log][6])
* **Rough size math** (estimation):

  * text per minute: ~150 words × ~6 bytes/word (avg letters + space/punct) ≈ **900 bytes/min**
  * WebVTT overhead per cue (timestamps, newlines, optional settings) is often **hundreds of bytes/minute** depending on segmentation.
  * **Rule of thumb** seen in practice: ~**80–400 KB per hour** for unstyled transcripts; heavy styling/short cues edge higher. *(This is an estimate derived from the format and public examples, not a normative figure.)*

**Implications for your CLI:**

* Treat these as **small text files**; even multi‑hour meetings are typically **sub‑MB** to a few MB at most. Use a **streaming parser** (`BufRead`/`Lines`) and avoid materializing the entire file if you plan additional processing.
* Be mindful of occasional **edge cases with timestamps/headers** (e.g., large `X‑TIMESTAMP‑MAP` in streaming contexts can trip some parsers; not common for offline meeting exports but worth defensive coding). ([GitHub][7])

---

## 4) How OSes pass file paths to registered handlers (absolute? quoted?)

* **Windows (file associations)**: The shell uses the command in the ProgID’s `shell\open\command`, typically with `%1` placeholder for **the file path**. **Quote paths containing spaces** (use `"%1"`). Microsoft documents quoting long/space‑containing paths and best practices for file associations. Paths are presented to your app as arguments. ([Microsoft Learn][8])
* **macOS (Launch Services)**: Finder sends **open‑document events** (not argv) to GUI apps; your app receives them via `NSApplicationDelegate` methods like `application(_:openFiles:)` / `application(_:openFile:)`, providing file names/URLs (effectively **absolute** file paths). For association, you declare supported document types in `CFBundleDocumentTypes`. For CLIs, users usually invoke via `open -a` or from Terminal. ([Apple Developer][9])
* **Linux/*BSD desktop (freedesktop.org)**: File associations are defined by a `.desktop` entry. The `Exec` key uses **field codes** like `%f`/`%F` (one or many file paths). **Launchers expand these into arguments**; quoting rules are defined by the spec (arguments can be quoted; field codes are replaced with one or more argv tokens). Use `%F` if you can handle multiple files. ([Freedesktop Specifications][10])

---

## 5) Rust CLI error‑handling patterns & exit codes

**Patterns that work well:**

* Use **clap** for argument parsing (robust UX, help, colored errors, suggestions). On parse errors, clap prints a message and exits (you can let `error.exit()` handle process exit). ([Docs.rs][11])
* Use **`std::process::ExitCode`** in `main()` for clean returns from your own code; reserve `process::exit` for rare cases when immediate termination is required. ([Rust Docs][12])
* Adopt a **consistent exit‑code scheme**. Many CLI authors follow **BSD `sysexits.h`** (portable and self‑describing):

  * **EX_USAGE (64)** – invalid CLI usage/args
  * **EX_DATAERR (65)** – bad input data (e.g., parse error in VTT)
  * **EX_NOINPUT (66)** – input file missing/unreadable
  * **EX_NOPERM (77)** – permission denied
  * **EX_SOFTWARE (70)** – internal error
    References: man pages for `sysexits`. ([man7.org][13])
* FYI: a Rust **panic** typically yields **exit code 101**; you should avoid panics for expected errors and map them to explicit codes. ([rust-cli.github.io][14])
* For user‑friendly diagnostics: **`miette`** (fancy, annotated errors) or **`anyhow`** (easy context and error chaining). `anyhow` pairs nicely with `thiserror` for library errors; `miette`’s `Diagnostic` trait makes pretty reports for CLIs. ([Docs.rs][15])

---

## 6) Speaker name sanitization (beyond `@`) & empty names

**What to normalize:**

* **Trim & collapse whitespace**; normalize Unicode to **NFC** (prevents weird combining sequences).
* **Strip/control invisible bidi marks** (e.g., LRM/RLM) if they harm readability.
* For Markdown, **escape punctuation** that can trigger formatting (e.g., `* _ # [ ] ( ) { } ! > | \` and backticks) — CommonMark explicitly allows **backslash‑escaping of punctuation**. ([GitHub][5])

**Recommended behavior for empties:**

* If `<v>` has **no annotation** or only whitespace, set speaker to a neutral label, e.g., `"Unknown"` or `"Speaker"`, and consider adding a stable numeric suffix when switching speakers (`Unknown #2`) so consolidated text remains distinguishable. (Empty annotations are allowed by the spec.) ([w3.org][1])

---

## 7) If the output file already exists: overwrite, prompt, or flag?

* General CLI guidance favors **non‑interactive by default** and **explicit flags for destructive actions**. A common pattern:

  * default: **error** if destination exists
  * `--force`/`-f`: **overwrite**
  * `--no-clobber`/`-n`: **skip** with a note
  * `--stdout` to stream results for piping
    This aligns with widely cited **Command Line Interface Guidelines (CLIG)** principles. ([clig.dev][16])

---

## 8) Whitespace policy when consolidating speaker text

* The WebVTT spec notes that **line breaks in cue text are significant** for captions, but for transcript output (Markdown) you’ll usually want **readable paragraphs**. Suggested rules:

  * **Trim** leading/trailing spaces within each fragment.
  * **Coalesce** multiple spaces to one, except inside inline code spans you generate.
  * **Join adjacent cues** for the same speaker with a **single space**, **unless** the cue ends with strong terminal punctuation (`.?!`) — in which case join with a **space** (or start a new sentence).
  * Consider inserting a **blank line** (new paragraph) after **long gaps** or **speaker changes**.
  * Keep soft line breaks only when the input had an intentional break (e.g., explicit linebreak in cue payload). See spec’s multiple‑line caption example and guidance. ([w3.org][1])

---

## Optional: standards, use‑cases, and comparable tools

**A. Standards / accessibility that can shape Markdown output**

* **WCAG 2.1/2.2 – 1.2.2 Captions (Prerecorded)** — captions must be present, synchronized, and accurate; transcripts should **identify speakers** and convey non‑speech audio where relevant. Your Markdown can reflect this with: speaker labels, timestamps (optional), and bracketed descriptions for sounds. ([w3.org][17])
* **W3C WAI “Making Audio & Video Accessible”** (and related pages) explain expectations for captions/transcripts, including **speaker identification** and **non‑speech cues** — handy for deciding whether to optionally include `[laughter]`, `[applause]`, etc. ([w3.org][18])

**B. Common non‑reading use cases (can inspire flags/features)**

* **Indexing / full‑text search** (export plain text as well as MD).
* **NLP/analytics** (speaker‑turn stats, topics, sentiment; JSON export beside MD).
* **Archival / legal** (stable IDs, checksums; optional timestamps per paragraph).
* **SEO / publishing** (clean, readable transcript for the web).
  (These map well to optional `--json`, `--include-timestamps`, `--no-speakers`, `--wrap N`, etc.)

**C. Other transcript tools & what users expect**

* Open‑source converters/transcribers show common asks: *“strip timestamps,” “preserve speakers,” “combine cues into paragraphs,” “export JSON/HTML”*. See examples focused on turning WebVTT into transcripts, e.g., Devlin’s **webvtt‑transcript**, others in GitHub topic listings, and Python libraries like `webvtt-py`. ([GitHub][19])

---

## Implementation checklist (practical)

* **Parsing**

  * Use an existing WebVTT parser crate if it fits (e.g., `webvtt`, `webvtt_parser`, or your own streaming parser); be permissive around **empty `<v>`**, **unknown tags**, and **Meet‑style blank lines**. ([Docs.rs][20])
* **Speaker resolution**

  * Prefer `<v>`; trim & normalize; Markdown‑escape; default unknowns. ([w3.org][1])
* **Consolidation**

  * Merge adjacent same‑speaker cues; trim, coalesce spaces, paragraphize sensibly (see Q8).
* **Flags (suggestions)**

  * `--force/-f`, `--no-clobber/-n`, `--stdout`
  * `--include-timestamps [none|first|each]`
  * `--wrap <cols>` (text wrapping)
  * `--unknown-speaker "Label"` (custom fallback)
  * `--format {md,txt,json}` (extensibility)
* **Errors & exit codes**

  * Map to `sysexits`: usage=64, parse(data)=65, noinput=66, noperm=77, internal=70; return via `ExitCode`. Consider `anyhow`/`miette` for rich errors. ([Ubuntu Manpages][21])
* **File associations (docs)**

  * Windows: ensure quotes around `%1`; Linux: provide a `.desktop` with `%F`; macOS: associations live with apps, not CLIs. ([Microsoft Learn][8])

---

### Sources (selected)

* **WebVTT spec (W3C Candidate Recommendation & examples)**: voice `<v>` semantics and cue text rules. ([w3.org][1])
* **Platform behaviors**: Teams example (<v Name>), Zoom transcript notes & variability, Meet example with blank lines and no `<v>`. ([GitHub][2])
* **CLI & error codes**: clap docs; `sysexits.h` manpages; Rust `ExitCode`; Rust CLI book on exit codes. ([Docs.rs][11])
* **Error reporting crates**: `anyhow`, `miette`. ([Docs.rs][15])
* **CLI UX**: Command Line Interface Guidelines (clig.dev). ([clig.dev][16])
* **Markdown escaping**: CommonMark allows backslash escapes of punctuation. ([GitHub][5])
* **Accessibility guidance**: WCAG 1.2.2 and WAI resources on captions/transcripts. ([w3.org][17])
* **Desktop file associations**: freedesktop `.desktop` Exec field codes. ([Freedesktop Specifications][10])
* **Windows associations & quoting**: MS docs + quoting for paths with spaces. ([Microsoft Learn][8])
* **macOS file open flow**: `CFBundleDocumentTypes`; `application(_:openFiles:)`. ([Apple Developer][22])



[1]: https://www.w3.org/TR/webvtt1/ "WebVTT: The Web Video Text Tracks Format"
[2]: https://github.com/glut23/webvtt-py/issues/55?utm_source=chatgpt.com "Transcript file metadata missing · Issue #55 · glut23/webvtt-py - GitHub"
[3]: https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0064927&utm_source=chatgpt.com "Using audio transcription for cloud recordings - Zoom"
[4]: https://github.com/danyelf/parse-vtt "GitHub - danyelf/parse-vtt"
[5]: https://github.com/commonmark/commonmark-spec?utm_source=chatgpt.com "GitHub - commonmark/commonmark-spec: CommonMark spec, with reference ..."
[6]: https://languagelog.ldc.upenn.edu/myl/ldc/llog/icslp06_final.pdf?utm_source=chatgpt.com "Towards an Integrated Understanding of Speaking Rate in Conversation"
[7]: https://github.com/videojs/video.js/issues/9111?utm_source=chatgpt.com "WebVTT Parser Fails with Large MPEGTS Timestamp Values"
[8]: https://learn.microsoft.com/en-us/windows/win32/shell/fa-best-practices?utm_source=chatgpt.com "Best Practices for File Associations - Win32 apps | Microsoft Learn"
[9]: https://developer.apple.com/documentation/appkit/nsapplicationdelegate/application%28_%3Aopenfiles%3A%29?utm_source=chatgpt.com "application(_:openFiles:) | Apple Developer Documentation"
[10]: https://specifications.freedesktop.org/desktop-entry/latest/?utm_source=chatgpt.com "Desktop Entry Specification"
[11]: https://docs.rs/clap/latest/clap/?utm_source=chatgpt.com "clap - Rust - Docs.rs"
[12]: https://doc.rust-lang.org/stable/std/process/struct.ExitCode.html?utm_source=chatgpt.com "ExitCode in std::process - Rust"
[13]: https://www.man7.org/linux/man-pages/man3/sysexits.h.3head.html?utm_source=chatgpt.com "sysexits.h (3head) — Linux manual page - man7.org"
[14]: https://rust-cli.github.io/book/in-depth/exit-code.html?utm_source=chatgpt.com "Exit codes - Command Line Applications in Rust"
[15]: https://docs.rs/anyhow/latest/anyhow/?utm_source=chatgpt.com "anyhow - Rust - Docs.rs"
[16]: https://clig.dev/?trk=public_post-text&utm_source=chatgpt.com "Command Line Interface Guidelines - clig.dev"
[17]: https://www.w3.org/WAI/WCAG21/Understanding/captions-prerecorded.html?utm_source=chatgpt.com "Understanding Success Criterion 1.2.2: Captions (Prerecorded) | WAI | W3C"
[18]: https://www.w3.org/WAI/media/av/captions/?utm_source=chatgpt.com "Captions/Subtitles | Web Accessibility Initiative (WAI) | W3C"
[19]: https://github.com/iandevlin/webvtt-transcript?utm_source=chatgpt.com "GitHub - iandevlin/webvtt-transcript: Provides a simple transcript of a ..."
[20]: https://docs.rs/webvtt/latest/webvtt/?utm_source=chatgpt.com "webvtt - Rust - Docs.rs"
[21]: https://manpages.ubuntu.com/manpages/noble/man3/sysexits.h.3head.html?utm_source=chatgpt.com "Ubuntu Manpage: sysexits.h - exit codes for programs"
[22]: https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundledocumenttypes?utm_source=chatgpt.com "CFBundleDocumentTypes | Apple Developer Documentation"
