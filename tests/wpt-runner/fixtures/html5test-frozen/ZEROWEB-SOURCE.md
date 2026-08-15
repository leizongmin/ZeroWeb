# Frozen HTML5test fixture

This directory is a source snapshot of `WebPlatformTest/HTML5test` commit
`47687ab6e94f6485c57eff75e8f79bc8cc95cbf1` (2018-05-12), downloaded from
the archived upstream repository on 2026-08-15.

It is covered by the upstream [MIT License](LICENSE).  The ZeroWeb fixture
adapter replaces the discontinued external WhichBrowser branding lookup with a
local equivalent and disables its obsolete HTTPS upgrade probe, so the page can
run offline and deterministically. The release 9 runner also isolates each
feature probe: an exception records that probe as unsupported instead of
aborting the entire report. These adaptations never report an unsupported API
as supported or add points to the score. The archived runner's timer-based
completion gate is also bypassed so the completed synchronous probe results
can be rendered in ZeroWeb's frozen test harness. The corresponding
presentation-only timer is made synchronous so the completed report becomes
visible without waiting for an unavailable timer callback.

The report begins visible rather than using the archived loading-only hidden
state. This only controls presentation in the frozen, synchronous harness; it
does not modify any feature probe, result, or score.

The score panel is assembled through equivalent DOM node creation rather than
one nested `innerHTML` string. Its actual score sentence is a direct text node
on the panel, avoiding the current nested heading paint limitation while
remaining visible through the host-mutation bridge.

The result table's two structural column wrappers are created with
`document.createElement()` rather than `innerHTML`. This is DOM-equivalent to
the archived markup, but ensures their subsequently appended, real result rows
remain on ZeroWeb's host mutation path. It does not modify any feature probe,
result, or score.
