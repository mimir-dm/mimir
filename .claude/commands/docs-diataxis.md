---
name: docs-diataxis
description: Document a codebase against the Diátaxis framework in four gated phases (discovery, plan, write, adversarial parallel review). Use when generating or overhauling project documentation. Dispatches the accuracy, completeness, clarity, and diataxis-compliance review agents.
---

# Diátaxis documentation coordinator

You are documenting this codebase against the Diátaxis framework. You work in
four phases and you do not skip or soften any of them. Your output is judged by
adversarial review agents at the end, so do the work properly the first time.

## The four quadrants, and the lines you DO NOT cross

Tutorial — learning by doing. A single guaranteed-to-work path that teaches a
beginner by having them build something real, with a visible result at each
step. NEVER explain the why beyond one sentence, NEVER offer choices or
alternatives, NEVER catalog options or edge cases. If you are explaining, you
have left the tutorial.

How-to — a task for someone already competent. A named real-world goal
("How to X") and the ordered steps to reach it. May branch where the goal
demands it. NEVER teach fundamentals, NEVER give extended rationale, NEVER try
to be complete; a how-to solves one problem, it does not survey the system.

Reference — accurate description of the machinery. Austere, neutral, structured
for lookup, mirroring the code's own structure. Every API, flag, config key,
env var, data model, and error. NEVER instruct, NEVER persuade, NEVER hand-hold,
NEVER editorialize. If a sentence has an opinion, it belongs in explanation.

Explanation — understanding and the why. Design decisions, trade-offs,
alternatives considered, mental models, context. Discursive, bounded by a topic
not a task. NEVER give numbered steps, NEVER exhaustively list machinery; point
to reference for that.

The single most common failure is one quadrant doing another's job. Police it.

## Grounding rule (non-negotiable)

Every factual claim in reference and every step in a how-to must trace to a
specific location in the code: file path plus the symbol, flag, or line. If you
cannot point to where the code does what you are claiming, you may not write the
claim; go read the code until you can, or omit it. No invented flags, no assumed
defaults, no "typically." Verify defaults and signatures against the source, not
memory.

## Phase 1 — Discovery (produces an artifact)

Explore the whole codebase and write an INVENTORY before planning anything:
- every entrypoint, config file, and README
- every user-facing surface: CLI commands and flags, public API, config keys,
  environment variables
- the main workflows traced end to end, with the files they run through
- build, test, and deployment lifecycle
- dependencies and integration points
- implicit/tribal knowledge a new hire would trip on that the code doesn't state

The inventory is a checklist the later phases are measured against. If a surface
is in the code, it is in the inventory.

## Phase 2 — Plan

Produce an outline of every document, grouped by quadrant. For each: title,
target audience, the inventory items it covers, and which other docs it links
to. Justify each document's existence in one line; not every topic needs four
quadrants. Coverage is judged against the inventory, so a flag or command in the
inventory with no reference doc is a planned gap you must either fill or
explicitly defer with a reason.

## Phase 3 — Write

Write every document in full. No placeholders, no TODOs, no "see code for
details." Concrete examples with real values from this codebase, never abstract
ones. Cross-link: tutorials link forward to the how-tos and reference they rely
on; reference links out to explanation for the why. Surface the implicit
knowledge from the inventory.

## Phase 4 — Review (gated loop, not one pass)

Dispatch the four review agents in parallel: accuracy-reviewer,
completeness-reviewer, clarity-reviewer, diataxis-compliance-reviewer. Each
returns findings as a list of
{severity: blocker|major|minor, location: doc + section, issue, required fix}.

Then loop: address every blocker and major, then re-run the reviewer(s) whose
findings you addressed. Repeat until accuracy, completeness, and compliance
return zero blockers and majors, and clarity returns no blockers. Minors are
addressed or explicitly waived with a reason. Do not declare done until that bar
is met. Do not rewrite a reviewer's job into your own opinion; if a finding is
wrong, say why with a code citation.

Definition of done: every inventory surface is covered or deferred-with-reason;
every reference claim is code-traceable; no quadrant violations remain.
