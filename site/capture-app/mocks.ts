// A fake Tauri backend for the capture page: every command the captured screens call gets a
// realistic answer. Only `import type` from the app here, so no app module runs before the
// mocks are installed (see main.ts).
import type { AppConfig, AiPreset, SpeechPreset } from '../../src/stores/appStore'
import type { RunTiming } from '../../src/lib/speed'
import libraryIndex from '../../presets/languages/index.json'

export type Screen =
  | 'home'
  | 'settings-languages'
  | 'onboarding'
  | 'copy-pill'
  // Promo-video stills (.claude/skills/promo-video): real screens set up for one feature.
  | 'promo-hands-free'
  | 'pill'

export const SCREENS: Record<Screen, { hash: string }> = {
  home: { hash: '#/' },
  'settings-languages': { hash: '#/settings?pane=llm' },
  onboarding: { hash: '#/' },
  'copy-pill': { hash: '#capsule' },
  // Settings → General with Hands-free on (plan `hands-free-mode`).
  'promo-hands-free': { hash: '#/settings?pane=general' },
  // The pill alone, in the state given by `&state=` (recording, transcribing, polishing, …).
  pill: { hash: '#capsule' },
}

/** Screens that render the pill window instead of the main window. */
export const PILL_SCREENS: Screen[] = ['copy-pill', 'pill']

const NOW = Date.UTC(2026, 8, 26, 9, 30)
const OLLAMA_ID = 'capture-ollama'
const SPEECH_SERVER_ID = 'capture-whisper-server'

let baseConfig: AppConfig | null = null
let config: AppConfig | null = null

/** Called by app.ts with the store's own defaults before anything renders. */
export function setBaseConfig(defaults: AppConfig) {
  baseConfig = defaults
}

type LibraryEntry = (typeof libraryIndex.presets)[number]
const libraryEntry = (id: string): LibraryEntry =>
  libraryIndex.presets.find((preset) => preset.id === id) as LibraryEntry

function presetRef(id: string) {
  const entry = libraryEntry(id)
  return { id: entry.id, version: entry.version, sha256: entry.sha256 }
}

/** The store defaults, set up as a finished install: models ready, three languages. */
function capturedConfig(screen: Screen, dark: boolean): AppConfig {
  if (config) return config
  if (!baseConfig) throw new Error('setBaseConfig was not called')
  const base = baseConfig
  const speech: SpeechPreset[] = [
    {
      ...base.speech_presets[0],
      model: 'large-v3-turbo',
      model_file: 'ggml-large-v3-turbo-q5_0.bin',
      verified_at: NOW,
    },
    {
      id: SPEECH_SERVER_ID,
      name: 'whisper.cpp server',
      kind: 'openai_compatible',
      base_url: 'http://localhost:8178/v1',
      model: 'large-v3-turbo',
      model_file: '',
      language: 'auto',
      builtin: false,
      verified_at: NOW,
    } as SpeechPreset,
  ]
  const ai: AiPreset[] = [
    {
      ...base.ai_presets[0],
      model: 'qwen3-4b',
      model_file: 'Qwen3-4B-Instruct-2507-Q4_K_M.gguf',
      verified_at: NOW,
    },
    {
      id: OLLAMA_ID,
      name: 'Ollama',
      kind: 'openai_compatible',
      base_url: 'http://localhost:11434/v1',
      model: 'qwen3:4b',
      model_file: '',
      extra_request_fields: {},
      builtin: false,
      verified_at: NOW,
    } as AiPreset,
  ]
  // The onboarding pill preview shows the first language; a short name keeps it still.
  const targets = screen === 'onboarding' ? ['en', 'zh-Hant-HK', 'ja'] : ['zh-Hant-HK', 'en', 'ja']
  const firstTarget = targets[0]
  config = {
    ...base,
    theme: dark ? 'dark' : 'light',
    ui_language: 'en',
    speech_presets: speech,
    active_speech_preset_id: speech[0].id,
    ai_presets: ai,
    active_ai_preset_id: ai[0].id,
    target_lang: firstTarget,
    translation: {
      targets,
      active_target: firstTarget,
      languages: {
        'zh-Hant-HK': {
          instructions: null,
          library_preset: presetRef('cantonese-hong-kong'),
          enabled: true,
          auto_update: true,
          user_hints: [],
        },
        en: {
          instructions: null,
          library_preset: presetRef('english'),
          enabled: true,
          auto_update: false,
          user_hints: [],
        },
        ja: { instructions: null, library_preset: null, enabled: true },
      },
    },
    shortcut_tour_completed: true,
    shortcut_tour_prompt_dismissed: true,
    hands_free: {
      enabled: screen === 'promo-hands-free',
      wake_name: 'Sam',
      sensitivity: 'normal',
      action: 'dictate',
    },
  }
  return config
}

/** 40 finished runs across two AI and two speech presets, newest last. */
function runTimings(cfg: AppConfig): RunTiming[] {
  const builtinSpeech = cfg.speech_presets[0].id
  const builtinAi = cfg.ai_presets[0].id
  const runs: RunTiming[] = []
  for (let i = 0; i < 40; i++) {
    // A deterministic wobble so the averages look like real use.
    const wobble = Math.sin(i * 1.7) * 0.5 + Math.cos(i * 0.9) * 0.3
    const recordingSecs = 3.2 + ((i * 37) % 50) / 10
    const onOllama = i % 3 === 1
    const onServer = i % 5 === 4
    const speechPerSec = onServer ? 0.31 : 0.27
    const speechMs = Math.round(recordingSecs * speechPerSec * 1000 * (1 + wobble * 0.08))
    const aiMs = Math.round((onOllama ? 420 : 610) * (1 + wobble * 0.12))
    const pasteMs = Math.round(120 + wobble * 20)
    const finishRecordingMs = 90
    runs.push({
      id: i + 1,
      mode: i % 7 === 3 ? 'translate' : 'dictate',
      recordingSecs,
      audioBytes: Math.round(recordingSecs * 32000),
      finishRecordingMs,
      speechMs,
      aiMs,
      pasteMs,
      totalMs: finishRecordingMs + speechMs + aiMs + pasteMs,
      speechPresetId: onServer ? SPEECH_SERVER_ID : builtinSpeech,
      speechModel: 'large-v3-turbo',
      aiPresetId: onOllama ? OLLAMA_ID : builtinAi,
      aiModel: onOllama ? 'qwen3:4b' : 'qwen3-4b',
      language: i % 4 === 0 ? 'zh' : 'en',
      outcome: 'ok',
    } as RunTiming)
  }
  return runs
}

function presetDetail(id: string, code: string) {
  const entry = libraryEntry(id)
  if (!entry) return null
  return {
    id: entry.id,
    name: entry.name,
    tier: entry.tier,
    summary: entry.summary,
    version: entry.version,
    sha256: entry.sha256,
    authors: entry.authors,
    model_hint: (entry as { model_hint?: string }).model_hint ?? null,
    text: `# ${entry.name}\n\n${entry.summary}`,
    variant: code.includes('-') && entry.id === 'english' ? code : null,
    detect_codes: entry.languages,
    hints: entry.id === 'cantonese-hong-kong' ? ['嘅', '咗', '喺', '唔', '聽日'] : [],
    require_hint: entry.id === 'cantonese-hong-kong',
    applies_to: entry.applies_to,
  }
}

const ready = (modelId: string, bytes: number) => ({
  modelId,
  phase: 'ready',
  downloadedBytes: bytes,
  totalBytes: bytes,
  bytesPerSecond: 0,
  error: null,
  testMs: 812,
})

const hardware = (models: [string, number][]) => ({
  hardware: {
    chipKind: 'apple_silicon',
    chipName: 'Apple M1 Pro',
    memoryBytes: 32 * 1024 ** 3,
    freeBytes: 120_000_000_000,
  },
  offer: {
    models: models.map(([id, sizeBytes], index) => ({ id, sizeBytes, recommended: index === 0 })),
    leftOut: null,
    neededBytes: null,
  },
})

const unknown = new Set<string>()

export function handleCommand(
  screen: Screen,
  dark: boolean,
  cmd: string,
  args: Record<string, unknown>,
): unknown {
  const cfg = capturedConfig(screen, dark)
  switch (cmd) {
    // Tauri plugins
    case 'plugin:store|load':
    case 'plugin:store|get_store':
      return 1
    case 'plugin:store|get': {
      if (args.key === 'onboarding_completed') return [screen !== 'onboarding', true]
      return [null, false]
    }
    case 'plugin:store|set':
    case 'plugin:store|save':
      return null
    case 'plugin:app|version':
      return '0.1.0'
    case 'plugin:app|name':
      return 'OraType'

    // Config and data
    case 'get_config':
      return cfg
    case 'update_config':
      return null
    case 'get_dictionary':
    case 'get_correction_rules':
    case 'list_custom_app_mappings':
      return []
    case 'get_platform_capabilities':
      return {
        os: 'macos',
        sessionType: 'unknown',
        globalHotkeyReliable: true,
        keyboardOutputReliable: true,
        clipboardAutoPasteReliable: true,
      }
    case 'get_hotkey_registration_error':
      return null
    case 'check_accessibility_permission':
      return true
    case 'get_microphone_permission':
    case 'get_automation_permission':
      return 'granted'
    case 'get_run_timings':
      return runTimings(cfg)
    case 'get_speed_stats':
      return { speakingWpm: 142, typingWpm: 48, timesFaster: 142 / 48 }
    case 'get_speech_setup_status':
      return ready('large-v3-turbo', 574_041_195)
    case 'get_ai_setup_status':
      return ready('qwen3-4b', 2_497_281_120)
    case 'get_speech_hardware':
      return hardware([
        ['large-v3-turbo', 574_041_195],
        ['small', 190_085_487],
      ])
    case 'get_ai_hardware':
      return hardware([
        ['qwen3-4b', 2_497_281_120],
        ['qwen3-1.7b', 1_107_409_472],
      ])
    case 'list_speech_models':
      return [
        {
          id: 'large-v3-turbo',
          fileName: 'ggml-large-v3-turbo-q5_0.bin',
          sizeBytes: 574_041_195,
          installed: true,
        },
        { id: 'small', fileName: 'ggml-small-q5_1.bin', sizeBytes: 190_085_487, installed: false },
      ]
    case 'list_ai_models':
      return [
        {
          id: 'qwen3-4b',
          fileName: 'Qwen3-4B-Instruct-2507-Q4_K_M.gguf',
          sizeBytes: 2_497_281_120,
          installed: true,
        },
        {
          id: 'qwen3-1.7b',
          fileName: 'Qwen3-1.7B-Q4_K_M.gguf',
          sizeBytes: 1_107_409_472,
          installed: false,
        },
      ]
    case 'get_credential_status':
      return {
        namespace: String(args.namespace),
        provider: String(args.provider),
        hasSecret: false,
        updatedAt: null,
        storage: 'os-vault',
      }
    case 'get_translation_language_defaults':
      return {}
    case 'get_language_library_status':
      return {
        latest: Object.fromEntries(
          libraryIndex.presets.map((p) => [p.id, { version: p.version, sha256: p.sha256 }]),
        ),
        updates: {},
      }
    case 'load_language_preset':
    case 'download_language_preset':
      return presetDetail(String(args.id), String(args.code))
    case 'list_language_presets':
      return { presets: [], related: [], offline: false }
    case 'list_input_devices':
      return [{ name: 'MacBook Pro Microphone', is_default: true }]
    case 'get_stt_recording_capability':
      // `stt::capabilities::ResolvedRecordingLimit`; the old flat shape crashed the Speech pane.
      return {
        capability: {
          registryVersion: 1,
          providerId: cfg.speech_presets[0].id,
          transport: 'fileUpload',
          recommendedMaxSeconds: 600,
          hardMaxSeconds: 720,
          maxUploadBytes: 25_165_824,
          source: 'clientBuffer',
          explanationKey: 'clientBuffer',
        },
        mode: 'auto',
        requestedSeconds: 600,
        effectiveMaxSeconds: 600,
      }
    case 'get_hotkey_status':
      // `HotkeyStatus`: every shortcut installed, no conflict (Settings → General).
      return {
        dictation: { value: 'End', valid: true },
        ask: { value: 'RightControl+End', valid: true },
        conflict: false,
        registration_error: null,
        roles: [],
        capability: {
          platform: 'macos',
          sessionType: 'unknown',
          supportsGlobalHotkey: true,
          supportsHoldMode: true,
          supportsReleasedEdge: true,
          supportsSideSpecificModifiers: true,
          requiresAccessibilityPermission: true,
          statusHint: null,
        },
      }
    case 'get_hands_free_status':
      return {
        listener: cfg.hands_free.enabled ? 'listening' : 'off',
        modelInstalled: true,
        modelSizeBytes: 59_707_625,
        setup: {
          modelId: null,
          phase: 'idle',
          downloadedBytes: 0,
          totalBytes: 0,
          bytesPerSecond: 0,
          error: null,
        },
      }
    case 'get_latest_mapping_candidate':
    case 'take_pending_ask_message':
      return null
    case 'set_shortcut_gate':
    case 'set_shortcut_tour_state':
    case 'start_mic_level_monitor':
    case 'stop_mic_level_monitor':
    case 'dismiss_copy_offer':
    case 'pause_hotkey':
    case 'resume_hotkey':
      return null
  }

  // Window and event plumbing: harmless no-ops.
  if (cmd.startsWith('plugin:window|') || cmd.startsWith('plugin:webview|')) return null
  if (cmd.startsWith('plugin:')) return null

  if (!unknown.has(cmd)) {
    unknown.add(cmd)
    console.warn('[capture] unmocked command', cmd, args)
  }
  return null
}
