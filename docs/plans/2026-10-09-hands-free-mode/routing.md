# Request and routing

What happens after the wake phrase. Part of [hands-free-mode](index.md).

## The request

On a match, the listener starts Ask exactly as the Ask shortcut does
(`hotkey::start_ask_hands_free`). The only difference is a `hands_free` flag on the Ask state.
The pill shows the normal recording state. The onboarding shortcut gate applies, so nothing
starts before onboarding is finished.

While Ask records, the listener feeds the same microphone audio to `AutoStop`
(`hands_free/segment.rs`), with the same voice gate and a 3 s hangover:

- **end phrase:** "Thanks, Sam" (with the user's wake name; also "Thank you, Sam", 多謝 Sam,
  谢谢 Sam, 唔該 Sam) → stop at once. At each pause a second segmenter hands the last 2.5 s of
  audio to the wake model (`WakeChecker::check_end`), and the matcher looks for a closing word
  and the name as the very last words (`WakePhrase::end_phrase_start`). "Thanks Sam for the
  notes" goes on. A Latin name is decoded as English and, when that misses, once more as
  Chinese, so a request in Cantonese can end with 多謝 Sam. The phrase is cut from the
  transcript before routing (`strip_end_phrase`); a request that was only the phrase is no
  request;
- **end of speech:** 3 s quiet after speech → stop. Long enough for a thinking pause while
  dictating; the end phrase is the quick way;
- **no speech:** nothing said within 5 s → stop. The usual voice check then ends the run with
  the calm no-speech fade (plan `quiet-no-speech`), and no audio is sent;
- **max length:** the recording limit from Settings (5–120 s) → stop.

Less than 200 ms of voice (a breath or a click while the user waits for the pill) neither
starts nor ends the request: it ended a real request 2 s after the wake phrase. When Escape or
the pill ends the run, the listener stops watching it and listens again once all runs are idle.

Stopping calls the same `stop_ask_shortcut` as the Ask key, so Escape, errors, the answer panel
and run timings all behave as for a shortcut run.

## Routing

In `stop_ask_dictation`, after transcription and language routing, a hands-free run goes
through `hands_free::routing::route`. Without a routing word, a request does what Settings says
(**After "Hey Sam"**: *Type what I say*, the default, or *Ask anything*). People use hands-free
mode mostly to type without the keyboard, so typing is the default; a question that is meant to
be typed ("Can we meet at six?") is typed, not answered.

When it types by default:

| Request starts with | Route |
| --- | --- |
| "ask", "ask anything", 問下 / 问下, 問一下 / 问一下, 問一問 / 问一问 | **Ask** the rest |
| "translate", 翻譯 / 翻译 | **Ask**, whole (Ask translates) |
| "type …" and the other dictate words below, except a bare 打 / 寫 | **Dictate** the rest |
| anything else, questions included | **Dictate**, whole |

A bare 問 / 问 or 打 is not a routing word here: too many dictated sentences start with them
(問佢幾點到, 打風).

When it asks by default:

| Request starts with | Route |
| --- | --- |
| "type", "type out", "type in", "dictate", "write down", "write out", "note down" | **Dictate** the rest |
| "write" + not a draft object (a, an, the, me, back, to …) | **Dictate** the rest |
| 打 (Cantonese "type"; not 打开 / 打电话 / 打算 …), 幫我打, 打字, 输入 / 輸入, 写下 / 寫下, 寫低, 记低, 听写 | **Dictate** the rest |
| bare 写 / 寫 (not 写一封 / 写个 / 帮我写 …) | **Dictate** the rest |
| anything else | **Ask**, unchanged |

In both, "open", "switch to", "launch", "go to", "bring up", 打开 / 打開, 切换到 / 切換到 and 转去
go to the **voice command** hook.

- **Dictate** runs `PipelineHandle::run_hands_free_dictation`: the same polish, dictionary,
  language rules and paste as the Dictate shortcut. The target is the app that had focus when
  the request started. With AI polish off, the text is pasted as heard.
- **Ask** keeps Ask's own intent detection, so questions, live questions, web search, drafts
  ("write an email to …"), edits of selected text and translations ("translate … into French")
  work as they do with the Ask shortcut.
- **Voice command:** `run_voice_command` is a hook that returns "not handled" until the
  `feature/voice-commands` work is on main. The request then does the default
  (`routing::fallback`).

Every route after the wake phrase uses the user's own speech and AI settings, as a shortcut run
does.
