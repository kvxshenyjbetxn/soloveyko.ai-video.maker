# Graph Report - soloveyko.ai-video.maker  (2026-06-21)

## Corpus Check
- 97 files · ~215,212 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1600 nodes · 3070 edges · 96 communities (90 shown, 6 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 102 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `abb3f29e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Stock Media Picker|Stock Media Picker]]
- [[_COMMUNITY_Montage Editor Media|Montage Editor Media]]
- [[_COMMUNITY_Claude API Client|Claude API Client]]
- [[_COMMUNITY_Pipeline Orchestration|Pipeline Orchestration]]
- [[_COMMUNITY_Settings & Storage|Settings & Storage]]
- [[_COMMUNITY_Googler Agent API|Googler Agent API]]
- [[_COMMUNITY_Editor Data Types|Editor Data Types]]
- [[_COMMUNITY_Frame Cache System|Frame Cache System]]
- [[_COMMUNITY_Gallery UI Icons|Gallery UI Icons]]
- [[_COMMUNITY_App Core State|App Core State]]
- [[_COMMUNITY_Bundle & Deployment|Bundle & Deployment]]
- [[_COMMUNITY_Video Generation UI|Video Generation UI]]
- [[_COMMUNITY_Agent Chat Window|Agent Chat Window]]
- [[_COMMUNITY_AGY CLI & Project Docs|AGY CLI & Project Docs]]
- [[_COMMUNITY_Topbar Balance UI|Topbar Balance UI]]
- [[_COMMUNITY_Edge TTS Service|Edge TTS Service]]
- [[_COMMUNITY_AssemblyAI Transcription|AssemblyAI Transcription]]
- [[_COMMUNITY_Codex CLI Client|Codex CLI Client]]
- [[_COMMUNITY_VoiceBot TTS Service|VoiceBot TTS Service]]
- [[_COMMUNITY_Editor Preview|Editor Preview]]
- [[_COMMUNITY_Src Core Pipeline Timeline Sync Rs Error|Src Core Pipeline Timeline Sync Rs Error]]
- [[_COMMUNITY_D|D]]
- [[_COMMUNITY_Pipeline Mod Build Job Settings|Pipeline Mod Build Job Settings]]
- [[_COMMUNITY_Ffmpegdownload|Ffmpegdownload]]
- [[_COMMUNITY_Pipeline Resume|Pipeline Resume]]
- [[_COMMUNITY_Agypermit|Agypermit]]
- [[_COMMUNITY_Api Openrouter|Api Openrouter]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Gui Queue|Gui Queue]]
- [[_COMMUNITY_Pixabayimage|Pixabayimage]]
- [[_COMMUNITY_Montage Editor Mod|Montage Editor Mod]]
- [[_COMMUNITY_Src Queue|Src Queue]]
- [[_COMMUNITY_Capcut Mod|Capcut Mod]]
- [[_COMMUNITY_Pipeline Subtitles|Pipeline Subtitles]]
- [[_COMMUNITY_Pipeline Translation Control|Pipeline Translation Control]]
- [[_COMMUNITY_Api Ffmpeg|Api Ffmpeg]]
- [[_COMMUNITY_Api Updater|Api Updater]]
- [[_COMMUNITY_Corebpe|Corebpe]]
- [[_COMMUNITY_Display|Display]]
- [[_COMMUNITY_Montage Editor Media Pool|Montage Editor Media Pool]]
- [[_COMMUNITY_Settings Storage Append To Task History|Settings Storage Append To Task History]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Montage Editor Timeline|Montage Editor Timeline]]
- [[_COMMUNITY_Montage Montage|Montage Montage]]
- [[_COMMUNITY_Montage Trigger|Montage Trigger]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Settings Storage Clean Numeric Param|Settings Storage Clean Numeric Param]]
- [[_COMMUNITY_Montage Editor Audio|Montage Editor Audio]]
- [[_COMMUNITY_Creationcontext|Creationcontext]]
- [[_COMMUNITY_Gui Logs|Gui Logs]]
- [[_COMMUNITY_Gui Subtitle Fonts|Gui Subtitle Fonts]]
- [[_COMMUNITY_Gui Task History|Gui Task History]]
- [[_COMMUNITY_Capcut Project Format Md Draft Content|Capcut Project Format Md Draft Content]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Pipeline Templates|Pipeline Templates]]
- [[_COMMUNITY_Icondata|Icondata]]
- [[_COMMUNITY_Montage Editor Topbar|Montage Editor Topbar]]
- [[_COMMUNITY_Settings Storage Appsettings Default|Settings Storage Appsettings Default]]
- [[_COMMUNITY_Src Core Pipeline Timeline Text Splitter Rs String|Src Core Pipeline Timeline Text Splitter Rs String]]
- [[_COMMUNITY_Pipeline Storage|Pipeline Storage]]
- [[_COMMUNITY_Settings General|Settings General]]
- [[_COMMUNITY_Settings Mod|Settings Mod]]
- [[_COMMUNITY_Montage Editor Inspector|Montage Editor Inspector]]
- [[_COMMUNITY_Pipeline Editing|Pipeline Editing]]
- [[_COMMUNITY_Src Theme|Src Theme]]
- [[_COMMUNITY_Pipelinetemplate|Pipelinetemplate]]
- [[_COMMUNITY_Pixabay Md Api Response Schema|Pixabay Md Api Response Schema]]
- [[_COMMUNITY_Src App|Src App]]
- [[_COMMUNITY_Community 83|Community 83]]
- [[_COMMUNITY_Community 84|Community 84]]
- [[_COMMUNITY_Community 85|Community 85]]
- [[_COMMUNITY_Community 86|Community 86]]
- [[_COMMUNITY_Community 87|Community 87]]
- [[_COMMUNITY_Community 88|Community 88]]
- [[_COMMUNITY_Community 89|Community 89]]
- [[_COMMUNITY_Community 90|Community 90]]
- [[_COMMUNITY_Community 93|Community 93]]
- [[_COMMUNITY_Community 94|Community 94]]
- [[_COMMUNITY_Community 95|Community 95]]

## God Nodes (most connected - your core abstractions)
1. `translate()` - 51 edges
2. `VideoMakerApp` - 44 edges
3. `String` - 39 edges
4. `MontageEditorState` - 33 edges
5. `Sync` - 29 edges
6. `String` - 28 edges
7. `StockPickerState` - 28 edges
8. `FrameCache` - 27 edges
9. `retry_from_stage()` - 22 edges
10. `draw_pipeline_panel()` - 22 edges

## Surprising Connections (you probably didn't know these)
- `draw_media_pool()` --calls--> `translate()`  [INFERRED]
  src/gui/montage_editor/media_pool.rs → src/localization/mod.rs
- `clip_from_json_seg()` --calls--> `uuid_str()`  [INFERRED]
  src/gui/montage_editor/state.rs → src/gui/montage_editor/utils.rs
- `load_timeline_clips()` --calls--> `uuid_str()`  [INFERRED]
  src/gui/montage_editor/state.rs → src/gui/montage_editor/utils.rs
- `split_clip_at()` --calls--> `uuid_str()`  [INFERRED]
  src/gui/montage_editor/timeline.rs → src/gui/montage_editor/utils.rs
- `draw_timeline()` --calls--> `translate()`  [INFERRED]
  src/gui/montage_editor/timeline.rs → src/localization/mod.rs

## Import Cycles
- 1-file cycle: `src/gui/montage_editor/media_pool.rs -> src/gui/montage_editor/media_pool.rs`
- 1-file cycle: `src/gui/montage_editor/timeline.rs -> src/gui/montage_editor/timeline.rs`
- 1-file cycle: `src/gui/montage_editor/state.rs -> src/gui/montage_editor/state.rs`
- 1-file cycle: `src/gui/montage_editor/mod.rs -> src/gui/montage_editor/mod.rs`
- 1-file cycle: `src/gui/montage_editor/preview.rs -> src/gui/montage_editor/preview.rs`
- 1-file cycle: `src/gui/montage_editor/topbar.rs -> src/gui/montage_editor/topbar.rs`
- 1-file cycle: `src/gui/montage_editor/types.rs -> src/gui/montage_editor/types.rs`
- 1-file cycle: `src/core/pipeline/montage/montage.rs -> src/core/pipeline/montage/montage.rs`
- 1-file cycle: `src/gui/gallery/video_player.rs -> src/gui/gallery/video_player.rs`
- 1-file cycle: `src/gui/stock_picker.rs -> src/gui/stock_picker.rs`
- 1-file cycle: `src/gui/update_dialog.rs -> src/gui/update_dialog.rs`
- 1-file cycle: `src/gui/pipeline/subtitles.rs -> src/gui/pipeline/subtitles.rs`
- 1-file cycle: `src/core/pipeline/voiceover/voiceover.rs -> src/core/pipeline/voiceover/voiceover.rs`
- 1-file cycle: `src/app.rs -> src/app.rs`
- 1-file cycle: `src/bundle.rs -> src/bundle.rs`
- 1-file cycle: `src/gui/settings/storage.rs -> src/gui/settings/storage.rs`
- 1-file cycle: `src/core/pipeline/capcut/mod.rs -> src/core/pipeline/capcut/mod.rs`
- 1-file cycle: `src/core/pipeline/timeline/sync.rs -> src/core/pipeline/timeline/sync.rs`
- 1-file cycle: `src/gui/montage_editor/audio.rs -> src/gui/montage_editor/audio.rs`
- 1-file cycle: `src/gui/montage_editor/frame_cache.rs -> src/gui/montage_editor/frame_cache.rs`

## Hyperedges (group relationships)
- **AI CLI Agent Dispatch System — Claude / Gemini / AGY / Codex** — antigravity_cli_md_agent_type_enum, antigravity_cli_md_call_claude_code, antigravity_cli_md_call_agy_new_session, claude_cli_md_claude_cli, gemini_cli_md_gemini_cli, codex_cli_md_codex_cli [INFERRED 0.85]
- **CapCut Project File Set — draft_content + draft_meta_info + timelines/project** — capcut_project_format_md_draft_content, capcut_project_format_md_draft_meta_info, capcut_project_format_md_timelines_project, capcut_project_format_md_generation_schema [EXTRACTED 1.00]
- **Pipeline Template Save/Load Cycle — PipelineTemplate + save_template + load_template** — claude_md_pipeline_template, claude_md_save_template, claude_md_load_template [EXTRACTED 1.00]

## Communities (96 total, 6 thin omitted)

### Community 0 - "Stock Media Picker"
Cohesion: 0.07
Nodes (69): CachedPhoto, CachedVideo, ColorImage, From, build_skeleton_cache_from_timeline(), check_download_complete(), delete_frame_cache_for_file(), draw_photo_grid() (+61 more)

### Community 1 - "Montage Editor Media"
Cohesion: 0.11
Nodes (17): ClipDragState, Error, HashMap, MontageEditorState, MontagePreviewSettings, OpacityDragState, PlayingAudio, Pos2 (+9 more)

### Community 2 - "Claude API Client"
Cohesion: 0.06
Nodes (44): AgyPermit, AgyLimiter, AgyPermit, AgyPermit<'a>, call_agy_cli(), call_agy_new_session_streaming(), call_agy_resume(), call_gemini_cli() (+36 more)

### Community 3 - "Pipeline Orchestration"
Cohesion: 0.16
Nodes (54): AgentSessionInfo, JobStatus, animate_single_image(), assign_media_to_timeline(), call_agent_new_session_streaming(), call_agent_resume(), decode_result(), find_changed_prompts_for_rebuild() (+46 more)

### Community 4 - "Settings & Storage"
Cohesion: 0.05
Nodes (31): default_assemblyai_key(), default_capcut_draft_path(), default_edge_tts_pitch(), default_edge_tts_rate(), default_edge_tts_voice(), default_edge_tts_volume(), default_model_agy(), default_model_claude() (+23 more)

### Community 5 - "Googler Agent API"
Cohesion: 0.09
Nodes (42): Agent, AccountLimits, ActiveThreads, animate_image_with_priority(), check_key(), CurrentUsage, fetch_balance(), generate_image_with_priority() (+34 more)

### Community 6 - "Editor Data Types"
Cohesion: 0.08
Nodes (23): Default, ClipDragState, ClipKind, DragMode, EditorClip, MontageEditorActions, MontagePreviewSettings, OpacityDragState (+15 more)

### Community 7 - "Frame Cache System"
Cohesion: 0.14
Nodes (19): FrameLoadResult, FrameCache, FrameQuality, Receiver, Sender, Context, DynamicImage, HashMap (+11 more)

### Community 8 - "Gallery UI Icons"
Cohesion: 0.09
Nodes (33): draw_eye_icon(), draw_menu_icon(), draw_play_triangle(), draw_refresh_icon(), draw_image_preview(), load_image_texture(), start_image_loading(), draw_gallery_tab() (+25 more)

### Community 9 - "App Core State"
Cohesion: 0.06
Nodes (35): App, AppSettings, EditorStats, AgentChatWindowState, Arc, BinaryDownload, Color32, Default (+27 more)

### Community 10 - "Bundle & Deployment"
Cohesion: 0.19
Nodes (32): Command, FnMut, bin_dir(), download_all(), download_file(), download_to_bytes(), download_whisper(), download_whisper_amd() (+24 more)

### Community 11 - "Video Generation UI"
Cohesion: 0.08
Nodes (30): draw_media_regen_window(), arrow_button(), draw_video_section(), duplicate_button(), image_provider_info(), video_provider_info(), Arc, Context (+22 more)

### Community 12 - "Agent Chat Window"
Cohesion: 0.14
Nodes (30): AgentChatWindowState, draw_agent_chat_windows(), draw_dot_sep(), draw_h_line(), draw_icon_arrow_down(), draw_icon_arrow_up(), draw_icon_l_arrow(), draw_icon_x() (+22 more)

### Community 13 - "AGY CLI & Project Docs"
Cohesion: 0.15
Nodes (20): AgentType Enum — Claude / Gemini / Agy Dispatch, AGY CLI — AI Agent CLI Tool, AgyLimiter — Concurrency Semaphore for AGY Requests, call_agy_new_session — Launch New AGY Agent Session, call_agy_resume — Resume Specific AGY Conversation, call_agy_resume_last — Continue Last AGY Session, call_claude_code — Claude API Invocation Functions, new_direct_cli_command — Windows-Safe CLI Launcher (no cmd /C) (+12 more)

### Community 14 - "Topbar Balance UI"
Cohesion: 0.11
Nodes (26): Arc, Context, GooglerBalance, Language, Mutex, Option, String, Arc (+18 more)

### Community 15 - "Edge TTS Service"
Cohesion: 0.10
Nodes (20): clean_friendly_name(), EdgeTTSLimiter, EdgeTTSPermit, EdgeTTSPermit<'a>, EdgeTTSVoice, fetch_voices(), parse_param(), synthesize() (+12 more)

### Community 16 - "AssemblyAI Transcription"
Cohesion: 0.16
Nodes (20): AssemblyAILimiter, AssemblyAIPermit, AssemblyAIPermit<'a>, check_key(), create_transcript(), ms_to_srt(), poll_transcript(), transcribe() (+12 more)

### Community 17 - "Codex CLI Client"
Cohesion: 0.15
Nodes (19): call_codex(), call_codex_new_session_streaming(), call_codex_resume(), CodexLimiter, CodexPermit, CodexPermit<'a>, extract_codex_response(), extract_shell_command() (+11 more)

### Community 18 - "VoiceBot TTS Service"
Cohesion: 0.12
Nodes (20): BalanceResponse, create_tts_task(), download_task_result(), fetch_balance(), get_task_status(), TaskCreateResponse, TaskStatusResponse, VoiceBotLimiter (+12 more)

### Community 19 - "Editor Preview"
Cohesion: 0.14
Nodes (25): compute_zoom(), draw_frame_overlay(), draw_preview(), find_media_for_clip(), OverlayRenderItem, render_clip_frame(), shake_uv(), transition_kind() (+17 more)

### Community 20 - "Src Core Pipeline Timeline Sync Rs Error"
Cohesion: 0.20
Nodes (25): Error, Option, Path, Result, String, Vec, build_text_stream(), build_timeline() (+17 more)

### Community 21 - "D"
Cohesion: 0.16
Nodes (20): D, Error, Option, Result, StockPhoto, StockProvider, StockVideo, String (+12 more)

### Community 22 - "Pipeline Mod Build Job Settings"
Cohesion: 0.14
Nodes (23): build_job_settings(), draw_pipeline_panel(), effective_save_path(), toggle_switch(), validate_and_enqueue(), Arc, BinaryDownload, EdgeTTSVoice (+15 more)

### Community 23 - "Ffmpegdownload"
Cohesion: 0.30
Nodes (15): FfmpegDownload, BinaryDownload, draw_download_row(), draw_tool_row(), draw_welcome_dialog(), draw_whisper_amd_row(), start_whisper_amd_download(), ToolChecks (+7 more)

### Community 24 - "Pipeline Resume"
Cohesion: 0.21
Nodes (17): draw_resume_dialog(), enqueue_fill_missing(), enqueue_fresh(), enqueue_with_resume(), FoundFiles, pre_mark_stages(), ResumePendingData, Context (+9 more)

### Community 25 - "Agypermit"
Cohesion: 0.16
Nodes (17): call_claude_code(), call_claude_code_new_session_streaming(), call_claude_code_resume(), ClaudeLimiter, ClaudePermit, ClaudePermit<'a>, format_claude_json_event(), parse_claude_json_response() (+9 more)

### Community 26 - "Api Openrouter"
Cohesion: 0.11
Nodes (15): CreditsData, CreditsResponse, fetch_balance(), OpenRouterLimiter, OpenRouterPermit, OpenRouterPermit<'a>, OpenRouterPermit, Arc (+7 more)

### Community 27 - "Community 27"
Cohesion: 0.27
Nodes (12): draw_model_selector(), draw_translation_section(), ModelsResponse, OpenRouterModel, Arc, Language, Mutex, Option (+4 more)

### Community 28 - "Gui Queue"
Cohesion: 0.16
Nodes (21): draw_queue_jobs_list(), draw_queue_panel(), format_file_size(), open_folder(), stage_color(), AgentChatWindowState, Arc, BinaryDownload (+13 more)

### Community 29 - "Pixabayimage"
Cohesion: 0.13
Nodes (18): PixabayImage, PixabayVideo, PixabayVideoSize, PixabayVideoSizes, Result, StockPhoto, StockProvider, StockVideo (+10 more)

### Community 30 - "Montage Editor Mod"
Cohesion: 0.16
Nodes (19): FrameCache, Instant, draw_montage_editor_window(), draw_montage_media_preview(), load_preview_texture(), MontageEditorActions, PipelineJob, Arc (+11 more)

### Community 31 - "Src Queue"
Cohesion: 0.20
Nodes (15): AgentChatMessage, AgentSessionInfo, JobSettings, JobStatus, PipelineJob, RetryStage, Arc, Condvar (+7 more)

### Community 32 - "Capcut Mod"
Cohesion: 0.25
Nodes (16): drive_letter(), forward_path(), gen_uuid(), generate_capcut_project(), image_dims(), MediaInfo, MediaKind, native_path() (+8 more)

### Community 33 - "Pipeline Subtitles"
Cohesion: 0.45
Nodes (16): draw_assemblyai_settings(), draw_font_picker(), draw_lang_and_model(), draw_subtitle_style(), draw_subtitles_section(), draw_whisper_amd_settings(), draw_whisper_settings(), draw_whisperx_settings() (+8 more)

### Community 34 - "Pipeline Translation Control"
Cohesion: 0.17
Nodes (15): draw_translation_control_windows(), TranslationControlWindowState, Arc, Context, HashMap, HashSet, Language, Mutex (+7 more)

### Community 35 - "Api Ffmpeg"
Cohesion: 0.14
Nodes (8): FfmpegLimiter, FfmpegPermit, FfmpegPermit<'a>, FfmpegPermit, Condvar, Drop, Mutex, Self

### Community 36 - "Api Updater"
Cohesion: 0.20
Nodes (12): check_for_updates(), GithubAsset, GithubRelease, is_newer(), UpdateInfo, GithubAsset, Arc, Context (+4 more)

### Community 37 - "Corebpe"
Cohesion: 0.19
Nodes (12): CoreBpe, calculate_hash(), count_tokens(), draw_editor(), EditorStats, get_encoder(), Default, Language (+4 more)

### Community 38 - "Display"
Cohesion: 0.15
Nodes (10): Display, Formatter, Language, translate(), draw_control_section(), Language, Ui, Default (+2 more)

### Community 39 - "Montage Editor Media Pool"
Cohesion: 0.23
Nodes (15): refresh_placeholder_clips(), clean_windows_path(), frame_cache_dir(), path_hash(), probe_duration(), probe_has_audio(), sharp_frame_cache_dir(), uuid_str() (+7 more)

### Community 40 - "Settings Storage Append To Task History"
Cohesion: 0.22
Nodes (14): append_to_task_history(), AppSettings, load_saved_templates(), load_task_history(), PipelineTemplate, remove_from_task_history(), save_task_history(), save_template() (+6 more)

### Community 41 - "Community 41"
Cohesion: 0.05
Nodes (38): CapCut Project Format — дослідження структури, `materials.audios[]` — аудіо матеріал, `materials.beats[]` (тільки аудіо-сегменти), `materials.canvases[]` (тільки відео-сегменти), `materials.material_colors[]` (тільки відео-сегменти), `materials.placeholder_infos[]`, `materials.sound_channel_mappings[]`, `materials.speeds[]` (+30 more)

### Community 42 - "Montage Editor Timeline"
Cohesion: 0.09
Nodes (33): ClipKind, draw_media_pool(), load_thumb_texture(), clip_fits_track(), draw_timeline(), find_snap_secs(), move_track(), split_clip_at() (+25 more)

### Community 43 - "Montage Montage"
Cohesion: 0.24
Nodes (12): Fn, build_image_filter_parts(), find_voice_file(), pick_transition(), run_montage(), OverlayTrigger, Option, Path (+4 more)

### Community 44 - "Montage Trigger"
Cohesion: 0.28
Nodes (12): ass_time_to_secs(), find_text_timing(), is_word_similar(), levenshtein(), normalize(), OverlayTrigger, remove_ass_tags(), srt_time_to_secs() (+4 more)

### Community 45 - "Community 45"
Cohesion: 0.11
Nodes (18): 1. Відновлення сесії, 2. Виведення, 3. Пропуск дозволів, 4. Додавання директорій, Важливі застереження, Встановлення AGY, Довідка agy --help, Додавання до уніфікованого виклику call_llm (+10 more)

### Community 46 - "Community 46"
Cohesion: 0.12
Nodes (15): Gemini CLI — команди та флаги, `gemini extensions` — керування розширеннями (aliases: `extension`), `gemini gemma` — керування локальною моделлю Gemma, `gemini hooks` — керування хуками (aliases: `hook`), `gemini mcp` — керування MCP-серверами, `gemini skills` — керування навичками агента (aliases: `skill`), MCP (Model Context Protocol), Воркспейс та середовище (+7 more)

### Community 47 - "Community 47"
Cohesion: 0.26
Nodes (19): draw_video_player(), extract_frames_file(), extract_single_frame_pipe(), get_video_dimensions(), start_fullscreen_extraction(), start_hover_extraction(), start_thumbnail_extraction(), VideoPlayer (+11 more)

### Community 48 - "Community 48"
Cohesion: 0.37
Nodes (12): call_llm(), call_openrouter(), ChatChoice, ChatMessage, ChatMessageContent, ChatRequest, ChatResponse, ChatUsage (+4 more)

### Community 49 - "Settings Storage Clean Numeric Param"
Cohesion: 0.35
Nodes (11): clean_numeric_param(), get_history_path(), get_settings_dir(), get_settings_path(), get_templates_dir(), load_settings(), load_template(), open_settings_folder() (+3 more)

### Community 50 - "Montage Editor Audio"
Cohesion: 0.13
Nodes (14): `codex app-server` — сервер без TUI (experimental), Codex CLI — команди та флаги, `codex exec` — неінтерактивний режим (аналог `-p`), `codex mcp` — керування MCP-серверами, Дозволи та безпека, Значення `--ask-for-approval`, Конфігурація, Моделі (+6 more)

### Community 51 - "Creationcontext"
Cohesion: 0.28
Nodes (5): CreationContext, Frame, Context, Self, Ui

### Community 52 - "Gui Logs"
Cohesion: 0.33
Nodes (8): draw_job_logs_window(), draw_logs_tab(), Context, Instant, Language, Option, String, Ui

### Community 53 - "Gui Subtitle Fonts"
Cohesion: 0.33
Nodes (8): load_subtitle_fonts(), system_font_dirs(), try_load_font(), Context, Option, PathBuf, String, Vec

### Community 54 - "Gui Task History"
Cohesion: 0.39
Nodes (8): draw_task_history_panel(), format_ts(), stage_dots(), Language, Option, String, TaskHistoryEntry, Ui

### Community 55 - "Capcut Project Format Md Draft Content"
Cohesion: 0.39
Nodes (8): draft_content.json — CapCut Main Timeline File, draft_meta_info.json — CapCut Media Pool Metadata, CapCut Project Generation Schema — Steps to Build Project from timeline.json, CapCut materials{} — Video/Audio/Aux Material Objects, CapCut segments[] — Clip Placement on Timeline, CapCut Time Units — Microseconds as Base Unit, Timelines/project.json — CapCut Timeline Registry, CapCut tracks[] — Timeline Track Structure

### Community 56 - "Community 56"
Cohesion: 0.29
Nodes (15): MediaItem, clip_from_json_seg(), find_audio_file(), load_external_media(), load_media_pool(), load_timeline_clips(), load_track_volumes(), load_voice_track_idx() (+7 more)

### Community 57 - "Pipeline Templates"
Cohesion: 0.25
Nodes (7): draw_templates_section(), Language, Option, OverlayTrigger, String, Ui, Vec

### Community 58 - "Icondata"
Cohesion: 0.38
Nodes (6): IconData, Renderer, load_icon(), main(), renderer_backend(), Result

### Community 59 - "Montage Editor Topbar"
Cohesion: 0.36
Nodes (7): draw_preview_settings(), draw_topbar(), PreviewQuality, Language, MontageEditorState, PipelineJob, Ui

### Community 60 - "Settings Storage Appsettings Default"
Cohesion: 0.29
Nodes (5): default_image_priority(), default_preview_fps(), default_preview_quality(), default_video_priority(), Self

### Community 61 - "Src Core Pipeline Timeline Text Splitter Rs String"
Cohesion: 0.71
Nodes (6): String, Vec, split_by_char_limit(), split_by_paragraphs(), split_by_sentences(), split_text()

### Community 62 - "Pipeline Storage"
Cohesion: 0.60
Nodes (5): draw_path_row(), draw_storage_section(), Language, String, Ui

### Community 63 - "Settings General"
Cohesion: 0.40
Nodes (5): draw_general_settings(), AppTheme, Color32, Language, Ui

### Community 64 - "Settings Mod"
Cohesion: 0.47
Nodes (5): draw_settings(), AppTheme, Color32, Language, Ui

### Community 65 - "Montage Editor Inspector"
Cohesion: 0.50
Nodes (4): draw_inspector(), Language, MontageEditorState, Ui

### Community 66 - "Pipeline Editing"
Cohesion: 0.40
Nodes (4): draw_editing_section(), Language, String, Ui

### Community 67 - "Src Theme"
Cohesion: 0.50
Nodes (4): apply_theme(), AppTheme, Color32, Context

### Community 69 - "Pixabay Md Api Response Schema"
Cohesion: 1.00
Nodes (3): Pixabay API Response Schema — hits, totalHits, image/video URLs, Pixabay Search Images API, Pixabay Search Videos API

### Community 87 - "Community 87"
Cohesion: 0.40
Nodes (12): JobSettings, Path, PathBuf, Result, String, Vec, merge_audio_binary(), merge_audio_ffmpeg() (+4 more)

### Community 88 - "Community 88"
Cohesion: 0.20
Nodes (10): AtomicBool, MediaItem, save_preview_jpeg(), Arc, ClipKind, DynamicImage, ImageResult, Path (+2 more)

### Community 89 - "Community 89"
Cohesion: 0.26
Nodes (10): AudioPlayer, embedded_audio_cache_path(), extract_embedded_audio_async(), PlayingAudio, OutputStream, Sink, Option, Path (+2 more)

### Community 90 - "Community 90"
Cohesion: 0.15
Nodes (12): Claude Code CLI — команди та флаги, MCP (Model Context Protocol), Дозволи, Модель та сесія, Основні флаги, Плагіни та налаштування, Підкоманди, Режим "bare" (+4 more)

### Community 93 - "Community 93"
Cohesion: 0.17
Nodes (11): draw_api_section(), Send, Arc, GooglerBalance, Language, Mutex, Option, String (+3 more)

### Community 94 - "Community 94"
Cohesion: 0.20
Nodes (11): draw_voiceover_section(), VoiceBotTemplate, Arc, EdgeTTSVoice, Language, Mutex, Option, Result (+3 more)

### Community 95 - "Community 95"
Cohesion: 0.29
Nodes (7): draw_update_dialog(), Arc, Context, Language, Mutex, Option, UpdateInfo

## Knowledge Gaps
- **468 isolated node(s):** `Context`, `Option`, `TextureHandle`, `Ui`, `Language` (+463 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Sync` connect `Community 93` to `Stock Media Picker`, `Claude API Client`, `Pipeline Orchestration`, `Googler Agent API`, `Video Generation UI`, `Edge TTS Service`, `AssemblyAI Transcription`, `Codex CLI Client`, `VoiceBot TTS Service`, `Pipeline Mod Build Job Settings`, `Ffmpegdownload`, `Agypermit`, `Api Openrouter`, `Community 27`, `Montage Editor Mod`, `Src Queue`, `Pipeline Subtitles`, `Api Ffmpeg`, `Api Updater`, `Montage Editor Timeline`, `Community 47`, `Community 87`, `Community 94`, `Community 95`?**
  _High betweenness centrality (0.360) - this node is a cross-community bridge._
- **Why does `translate()` connect `Display` to `Stock Media Picker`, `Gallery UI Icons`, `Video Generation UI`, `Agent Chat Window`, `Topbar Balance UI`, `Pipeline Mod Build Job Settings`, `Ffmpegdownload`, `Pipeline Resume`, `Community 27`, `Gui Queue`, `Pipeline Subtitles`, `Pipeline Translation Control`, `Corebpe`, `Montage Editor Timeline`, `Creationcontext`, `Gui Logs`, `Gui Task History`, `Pipeline Templates`, `Montage Editor Topbar`, `Pipeline Storage`, `Settings General`, `Montage Editor Inspector`, `Pipeline Editing`, `Pipelinetemplate`, `Community 93`, `Community 94`, `Community 95`?**
  _High betweenness centrality (0.295) - this node is a cross-community bridge._
- **Why does `save_settings()` connect `Settings Storage Clean Numeric Param` to `Settings Storage Append To Task History`, `Creationcontext`, `Settings & Storage`?**
  _High betweenness centrality (0.095) - this node is a cross-community bridge._
- **Are the 49 inferred relationships involving `translate()` (e.g. with `draw_media_regen_window()` and `draw_gallery_tab()`) actually correct?**
  _`translate()` has 49 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Context`, `Option`, `TextureHandle` to the rest of the system?**
  _468 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Stock Media Picker` be split into smaller, more focused modules?**
  _Cohesion score 0.07243243243243243 - nodes in this community are weakly interconnected._
- **Should `Montage Editor Media` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._