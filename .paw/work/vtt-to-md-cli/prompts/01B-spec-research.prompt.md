---
agent: 'PAW-01B Spec Researcher'
---
# Spec Research Prompt: VTT to MD CLI

Perform research to answer the following questions.

Target Branch: feature/v1
Issue URL: https://github.com/lossyrob/vtt-to-md/issues/1
Additional Inputs: none

## Questions

1. What is the official WebVTT specification for voice tags (`<v>` elements)? What variations exist in how different platforms (Microsoft Teams, Zoom, Google Meet) implement speaker identification in VTT files?

2. What are common edge cases in VTT file formatting that the parser should handle gracefully (malformed tags, missing closing tags, nested structures, special characters in speaker names, empty speaker identifiers)?

3. What are typical file sizes for meeting transcripts in VTT format? Are there performance or memory considerations for processing large transcripts (e.g., multi-hour meetings)?

4. How do operating systems pass file paths to programs registered as file handlers? Are paths typically absolute or relative? Are they quoted when containing spaces?

5. What error handling patterns are considered best practice for CLI tools in Rust? What exit codes should be used for different error categories (file not found, permission denied, parse errors, invalid arguments)?

6. Beyond the `@` symbol replacement, what other speaker name sanitization or normalization might be needed? How should empty or whitespace-only speaker names be handled?

7. What should happen if the output file already exists? Should it be overwritten silently, prompt for confirmation, or provide a flag to control this behavior?

8. What are the expectations for whitespace handling in consolidated speaker text (leading/trailing spaces, multiple consecutive spaces, newlines within segments)?

### Optional External / Context

1. Are there industry standards or accessibility guidelines for transcript formatting that should influence the Markdown output structure?

2. What are common use cases for VTT-to-text conversion beyond reading transcripts (indexing, analysis, archival)? Could these inform optional features?

3. What other CLI transcript tools exist and what features do users commonly request or expect?
