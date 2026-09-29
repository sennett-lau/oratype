// Mounts the real app (src/App.tsx) into the capture frame, after main.ts installed the mocks.
import { createRoot } from 'react-dom/client'
import '../../src/i18n'
import './app.css'
import './capture.css'
import App from '../../src/App'
import { SHORTCUT_TOUR_FIRST_STEP, useAppStore } from '../../src/stores/appStore'
import { SHORTCUT_PAGES } from '../../src/components/Onboarding/shortcutConfig'
import { setBaseConfig, type Screen } from './mocks'
import { AskAnswerPanel, type AskPanelContent } from '../../src/components/AskPanel/AskAnswerPanel'
import { CapsuleAskThinking } from '../../src/components/Capsule/CapsuleAskThinking'
import { CapsuleAurora } from '../../src/components/Capsule/CapsuleAurora'
import type { AskDictationResult } from '../../src/lib/tauri'

const LONG = `The Warriors start their preseason with three games in the first week of October [1]:

• Sun 4 Oct, 7:00 PM: vs LA Clippers at Chase Center, San Francisco [1][3]
• Tue 6 Oct, 7:30 PM: at Portland Trail Blazers, Moda Center [1]
• Fri 9 Oct, 7:00 PM: vs Los Angeles Lakers at Chase Center [1]

The Clippers game is the first look at the new rotation. The preview expects Stephen Curry to play limited minutes, around 18 to 20, while the coaching staff tests younger guards in the second unit [2].

Reuters reports the team will also use the opener to try a new starting frontcourt after the summer trade, and that the regular season begins on 21 October at home [3].

Tickets for the opener are still on sale. Preseason games are shown on NBC Sports Bay Area and streamed on the NBA app [2].`
const QUESTION = 'What’s the next match for the Golden State Warriors?'

function askContent(screen: Screen): AskPanelContent | null {
  const base = {
    question: QUESTION,
    usedSelectedText: false,
    fallbackReason: null,
    mayBeOutOfDate: false,
  } as unknown as AskDictationResult
  if (screen === 'ask-answer-sources' || screen === 'ask-long-sources')
    return {
      kind: 'result',
      result: {
        ...base,
        output: 'popupAnswer',
        answer: screen === 'ask-long-sources' ? LONG : 'The Warriors play the LA Clippers on Sunday, 4 October 2026, at 7:00 PM [1].',
        sources: [
          { number: 1, title: 'Golden State Warriors Schedule 2026-27 - ESPN', url: 'https://www.espn.com/nba/team/schedule/_/name/gs/golden-state-warriors', snippet: 'Full 2026-27 schedule with preseason dates, times, TV and results for the Golden State Warriors.' },
          { number: 2, title: 'Warriors vs. Clippers: Preseason game preview | NBA.com', url: 'https://www.nba.com/game/lac-vs-gsw-0012600012', snippet: 'Curry expected to play around 18 to 20 minutes as the Warriors test a new second unit.' },
          { number: 3, title: 'Warriors open preseason against Clippers in San Francisco | Reuters', url: 'https://www.reuters.com/sports/basketball/warriors-open-preseason-against-clippers-2026-10-01/', snippet: 'Golden State will try a new starting frontcourt when it hosts the Clippers on Sunday.' },
        ],
      } as AskDictationResult,
    }
  if (screen === 'ask-not-configured')
    return { kind: 'result', result: { ...base, output: 'needsLiveInfo', answer: '', liveSearch: 'notConfigured' } as AskDictationResult }
  return null
}

function mountAsk(screen: Screen) {
  const stage = document.getElementById('stage') as HTMLElement
  stage.innerHTML = '<div id="app" class="ask-screen"></div>'
  const content = askContent(screen)
  createRoot(document.getElementById('app') as HTMLElement).render(
    <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'flex-end', paddingBottom: 24, gap: 16 }}>
      {content && (
        <AskAnswerPanel content={content} onClose={() => {}} onAnswerAnyway={() => {}} limits={{ maxWidth: 1008, maxHeight: 443 }} />
      )}
      {!content && <div className="pill relative rounded-full" style={{ width: content ? 64 : 140, height: 32, borderRadius: 16 }}>
        {!content && (
          <>
            <div className="absolute inset-0" style={{ borderRadius: 'inherit' }}><CapsuleAurora mode="working" /></div>
            <div className="relative h-full"><CapsuleAskThinking /></div>
          </>
        )}
      </div>}
    </div>,
  )
  setTimeout(() => {
    window.__captureReady = true
  }, 1800)
}

declare global {
  interface Window {
    /** Set once the screen has rendered and settled; the capture script waits for it. */
    __captureReady?: boolean
  }
}

const COPY_TEXT =
  'Thanks for the update! Let’s meet on Thursday at 4 pm to go through the launch checklist.'

export function mount(screen: Screen) {
  const defaults = useAppStore.getState().config
  setBaseConfig(defaults)
  // Both services passed a check (green dots in the sidebar).
  useAppStore.setState({
    speechHealth: { presetId: defaults.speech_presets[0].id, ok: true },
    aiHealth: { presetId: defaults.ai_presets[0].id, ok: true },
  })

  if (screen.startsWith('ask-')) {
    mountAsk(screen)
    return
  }
  if (screen === 'onboarding-websearch') {
    useAppStore.setState({ onboardingStep: 4 })
  }
  if (screen === 'onboarding') {
    // The Translate setup page: key caps, the language list and the pill preview.
    const index = SHORTCUT_PAGES.findIndex((p) => p.role === 'translate' && p.kind === 'setup')
    useAppStore.setState({ onboardingStep: SHORTCUT_TOUR_FIRST_STEP + index })
  }

  const target = document.getElementById('app') as HTMLElement
  createRoot(target).render(<App />)

  if (screen === 'copy-pill') {
    // The Copy pill appears when a result had nowhere to be pasted.
    setTimeout(
      () => useAppStore.getState().setCopyOffer({ text: COPY_TEXT, targetLang: null }),
      1500,
    )
  }

  // Give data loads, fonts and entry animations time to finish.
  setTimeout(() => {
    window.__captureReady = true
  }, 1800)
}
